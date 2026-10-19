//! Reading and writing images: every format the menu offers, behind [`decode`] and
//! [`encode`].

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use image::metadata::Orientation;
use image::{
    DynamicImage, GrayImage, ImageDecoder, ImageError, ImageFormat, ImageReader, Limits, RgbImage,
    RgbaImage,
};

use crate::error::{Error, Result};
use crate::kind::FileKind;

/// No side of an image may be longer than this…
pub const MAX_SIDE: u32 = 30_000;
/// …and decoding one may not allocate more than this. Together they stop a small file
/// that claims to be enormous from exhausting memory.
pub const MAX_ALLOC: u64 = 1 << 30;

const WHITE: [u8; 3] = [255, 255, 255];

/// A decoded image, already turned upright.
pub struct Decoded {
    pub image: DynamicImage,
    /// The colour profile, carried over when writing JPEG or PNG.
    pub icc: Option<Vec<u8>>,
    /// The pixels were turned or flipped to stand upright, so they differ from the file's.
    pub reoriented: bool,
}

/// Whether [`decode`] can read `kind` on this platform. HEIC and AVIF go through Windows'
/// own codecs, which need Microsoft Store extensions; whether those are installed is only
/// known on decode.
pub fn can_decode(kind: FileKind) -> bool {
    match kind {
        FileKind::Avif | FileKind::Heic => cfg!(windows),
        _ => image_format(kind).is_some(),
    }
}

/// Whether [`encode`] can write `kind` on this machine.
pub fn can_encode(kind: FileKind) -> bool {
    match kind {
        FileKind::Jpeg
        | FileKind::Png
        | FileKind::Webp
        | FileKind::Bmp
        | FileKind::Tiff
        | FileKind::Gif
        | FileKind::Avif => true,
        FileKind::Heic => heic_encoder_available(),
        _ => false,
    }
}

#[cfg(windows)]
fn heic_encoder_available() -> bool {
    dropwheel_wic::can_encode_heic()
}

#[cfg(not(windows))]
fn heic_encoder_available() -> bool {
    false
}

/// Reads the image at `path`, which has been detected as `kind`, and applies its EXIF
/// orientation so the pixels are the right way up.
pub fn decode(path: &Path, kind: FileKind) -> Result<Decoded> {
    match kind {
        FileKind::Avif | FileKind::Heic => decode_with_windows(path, kind),
        _ => decode_with_image(path, kind),
    }
}

fn image_format(kind: FileKind) -> Option<ImageFormat> {
    Some(match kind {
        FileKind::Jpeg => ImageFormat::Jpeg,
        FileKind::Png => ImageFormat::Png,
        FileKind::Webp => ImageFormat::WebP,
        FileKind::Bmp => ImageFormat::Bmp,
        FileKind::Tiff => ImageFormat::Tiff,
        FileKind::Gif => ImageFormat::Gif,
        FileKind::Ico => ImageFormat::Ico,
        _ => return None,
    })
}

pub(crate) fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

fn decode_with_image(path: &Path, kind: FileKind) -> Result<Decoded> {
    let Some(format) = image_format(kind) else {
        return Err(not_an_image(path, kind));
    };
    let file = File::open(path).map_err(|e| Error::io(path, e))?;
    let mut reader = ImageReader::new(BufReader::new(file));
    reader.set_format(format);
    reader.limits(limits());
    let failed = |e| decode_error(path, kind, e);

    let mut decoder = reader.into_decoder().map_err(failed)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let icc = decoder.icc_profile().ok().flatten();
    let mut image = DynamicImage::from_decoder(decoder).map_err(failed)?;
    image.apply_orientation(orientation);
    Ok(Decoded {
        image,
        icc,
        reoriented: orientation != Orientation::NoTransforms,
    })
}

/// HEIF keeps rotation in container properties that the codec applies, so EXIF
/// orientation is not applied again here.
#[cfg(windows)]
fn decode_with_windows(path: &Path, kind: FileKind) -> Result<Decoded> {
    let data = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let bitmap = dropwheel_wic::decode(&data, MAX_SIDE, MAX_ALLOC / 4).map_err(|e| {
        let reason = if e.missing_codec {
            missing_codec_hint(kind)
        } else {
            e.to_string()
        };
        Error::Decode {
            path: path.to_path_buf(),
            kind,
            reason,
        }
    })?;
    let image = RgbaImage::from_raw(bitmap.width, bitmap.height, bitmap.rgba)
        .ok_or_else(|| not_an_image(path, kind))?;
    Ok(Decoded {
        image: DynamicImage::ImageRgba8(image),
        icc: None,
        reoriented: false,
    })
}

#[cfg(not(windows))]
fn decode_with_windows(path: &Path, kind: FileKind) -> Result<Decoded> {
    Err(Error::Decode {
        path: path.to_path_buf(),
        kind,
        reason: "this format is opened through Windows' own codecs".into(),
    })
}

#[cfg_attr(not(windows), allow(dead_code))]
fn missing_codec_hint(kind: FileKind) -> String {
    let extension = match kind {
        FileKind::Heic => "HEIF Image Extensions",
        FileKind::Avif => "AV1 Video Extension",
        _ => "the matching image extension",
    };
    format!("Windows needs the free \"{extension}\" from the Microsoft Store to open this file")
}

fn not_an_image(path: &Path, kind: FileKind) -> Error {
    Error::Decode {
        path: path.to_path_buf(),
        kind,
        reason: "it is not an image Dropwheel can open".into(),
    }
}

fn decode_error(path: &Path, kind: FileKind, e: ImageError) -> Error {
    let reason = match e {
        ImageError::IoError(io) => return Error::io(path, io),
        ImageError::Limits(_) => {
            format!("it is larger than {MAX_SIDE} pixels on a side, or too large to fit in memory")
        }
        other => format!("the file is damaged or incomplete ({other})"),
    };
    Error::Decode {
        path: path.to_path_buf(),
        kind,
        reason,
    }
}

/// How to write an image. Each field only matters to the formats that use it.
#[derive(Debug, Clone)]
pub struct EncodeOptions<'a> {
    /// 1 to 100, for JPEG, lossy WebP and AVIF.
    pub quality: u8,
    /// WebP only: write it lossless and ignore `quality`.
    pub lossless: bool,
    /// PNG only: reduce to this many colours (64 to 256) for a smaller file.
    pub palette: Option<u16>,
    /// Colour profile to embed in JPEG and PNG.
    pub icc: Option<&'a [u8]>,
}

impl Default for EncodeOptions<'_> {
    fn default() -> Self {
        EncodeOptions {
            quality: 90,
            lossless: false,
            palette: None,
            icc: None,
        }
    }
}

/// Encodes `image` as `target`. Formats without transparency (JPEG, and BMP as most
/// programs read it) get it flattened onto white.
pub fn encode(image: &DynamicImage, target: FileKind, opts: &EncodeOptions) -> Result<Vec<u8>> {
    let encode_error = |reason: String| Error::Encode {
        kind: target,
        reason,
    };
    match target {
        FileKind::Jpeg => encode_jpeg(image, opts).map_err(encode_error),
        FileKind::Png => encode_png(image, opts).map_err(encode_error),
        FileKind::Webp => encode_webp(image, opts).map_err(encode_error),
        FileKind::Bmp => {
            let rgb = DynamicImage::ImageRgb8(flatten(image, WHITE));
            write_with_image(&rgb, ImageFormat::Bmp).map_err(encode_error)
        }
        FileKind::Gif => {
            let rgba = DynamicImage::ImageRgba8(image.to_rgba8());
            write_with_image(&rgba, ImageFormat::Gif).map_err(encode_error)
        }
        FileKind::Tiff => encode_tiff(image).map_err(encode_error),
        FileKind::Avif => encode_avif(image, opts).map_err(encode_error),
        FileKind::Heic => encode_heic(image).map_err(encode_error),
        _ => Err(encode_error("this format cannot be written".into())),
    }
}

/// Composites `image` over a solid `background`.
pub fn flatten(image: &DynamicImage, background: [u8; 3]) -> RgbImage {
    if !image.color().has_alpha() {
        return image.to_rgb8();
    }
    let rgba = image.to_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0;
        let blend = |c: u8, bg: u8| {
            let (c, bg, a) = (u32::from(c), u32::from(bg), u32::from(a));
            ((c * a + bg * (255 - a) + 127) / 255) as u8
        };
        image::Rgb([
            blend(r, background[0]),
            blend(g, background[1]),
            blend(b, background[2]),
        ])
    })
}

/// 8-bit pixels in the smallest layout that keeps the image intact.
enum Pixels {
    Gray(GrayImage),
    Rgb(RgbImage),
    Rgba(RgbaImage),
}

fn pixels(image: &DynamicImage) -> Pixels {
    let color = image.color();
    if color.has_alpha() {
        let rgba = image.to_rgba8();
        if rgba.pixels().all(|p| p.0[3] == u8::MAX) {
            return if color.has_color() {
                Pixels::Rgb(image.to_rgb8())
            } else {
                Pixels::Gray(image.to_luma8())
            };
        }
        Pixels::Rgba(rgba)
    } else if color.has_color() {
        Pixels::Rgb(image.to_rgb8())
    } else {
        Pixels::Gray(image.to_luma8())
    }
}

fn encode_jpeg(image: &DynamicImage, opts: &EncodeOptions) -> Result<Vec<u8>, String> {
    use jpeg_encoder::{ColorType, Encoder, SamplingFactor};

    let (data, color, width, height) = match pixels(image) {
        Pixels::Gray(g) => (g.as_raw().clone(), ColorType::Luma, g.width(), g.height()),
        Pixels::Rgb(rgb) => (
            rgb.as_raw().clone(),
            ColorType::Rgb,
            rgb.width(),
            rgb.height(),
        ),
        Pixels::Rgba(_) => {
            let rgb = flatten(image, WHITE);
            (
                rgb.as_raw().clone(),
                ColorType::Rgb,
                rgb.width(),
                rgb.height(),
            )
        }
    };
    let too_big = |_| "JPEG images are limited to 65,535 pixels on a side".to_string();
    let width = u16::try_from(width).map_err(too_big)?;
    let height = u16::try_from(height).map_err(too_big)?;

    let mut out = Vec::new();
    let quality = opts.quality.clamp(1, 100);
    let mut encoder = Encoder::new(&mut out, quality);
    encoder.set_progressive(true);
    encoder.set_optimized_huffman_tables(true);
    // Full-resolution colour at high quality; halved colour resolution saves a lot below.
    encoder.set_sampling_factor(if quality >= 90 {
        SamplingFactor::R_4_4_4
    } else {
        SamplingFactor::R_4_2_0
    });
    if let Some(icc) = opts.icc {
        encoder.add_icc_profile(icc).map_err(|e| e.to_string())?;
    }
    encoder
        .encode(&data, width, height, color)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

fn encode_png(image: &DynamicImage, opts: &EncodeOptions) -> Result<Vec<u8>, String> {
    let (width, height) = (image.width(), image.height());
    let mut info = png::Info::with_size(width, height);
    info.bit_depth = png::BitDepth::Eight;
    info.icc_profile = opts.icc.map(Cow::Borrowed);

    let px = pixels(image);
    let quantized = opts.palette.and_then(|n| {
        let rgba = match &px {
            Pixels::Rgba(rgba) => Cow::Borrowed(rgba),
            _ => Cow::Owned(image.to_rgba8()),
        };
        quantize_rgba(&rgba, usize::from(n.clamp(64, 256)))
    });

    let data = match (quantized, px) {
        (Some(q), _) => {
            info.color_type = png::ColorType::Indexed;
            info.palette = Some(Cow::Owned(q.palette));
            info.trns = q.alpha.map(Cow::Owned);
            q.indices
        }
        (None, Pixels::Gray(g)) => {
            info.color_type = png::ColorType::Grayscale;
            g.into_raw()
        }
        (None, Pixels::Rgb(rgb)) => {
            info.color_type = png::ColorType::Rgb;
            rgb.into_raw()
        }
        (None, Pixels::Rgba(rgba)) => {
            info.color_type = png::ColorType::Rgba;
            rgba.into_raw()
        }
    };

    let mut out = Vec::new();
    let mut encoder = png::Encoder::with_info(&mut out, info).map_err(|e| e.to_string())?;
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&data).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

struct Quantized {
    indices: Vec<u8>,
    /// RGB triplets.
    palette: Vec<u8>,
    /// Alpha per palette entry, up to the last one that is not opaque.
    alpha: Option<Vec<u8>>,
}

/// Reduces `rgba` to at most `colors` palette entries with NeuQuant. Fully transparent
/// pixels get an entry of their own: NeuQuant treats alpha as one more channel and would
/// otherwise blend them with visible colours. Returns `None` when too few pixels are
/// visible to train on, and the image stays in full colour.
fn quantize_rgba(rgba: &RgbaImage, colors: usize) -> Option<Quantized> {
    const MIN_TRAINING_PIXELS: usize = 1024;
    let has_clear = rgba.pixels().any(|p| p.0[3] == 0);
    let visible: Vec<u8> = if has_clear {
        rgba.pixels()
            .filter(|p| p.0[3] != 0)
            .flat_map(|p| p.0)
            .collect()
    } else {
        rgba.as_raw().clone()
    };
    if visible.len() < 4 * MIN_TRAINING_PIXELS {
        return None;
    }
    let trained = if has_clear { colors - 1 } else { colors };
    let nq = color_quant::NeuQuant::new(10, trained, &visible);

    let mut map = nq.color_map_rgba();
    let clear = u8::try_from(map.len() / 4).unwrap_or(u8::MAX);
    if has_clear {
        map.extend_from_slice(&[0, 0, 0, 0]);
    }
    let indices = rgba
        .pixels()
        .map(|p| {
            if p.0[3] == 0 {
                clear
            } else {
                u8::try_from(nq.index_of(&p.0)).unwrap_or(u8::MAX)
            }
        })
        .collect();

    let mut palette = Vec::with_capacity(map.len() / 4 * 3);
    let mut alpha = Vec::with_capacity(map.len() / 4);
    for entry in map.chunks_exact(4) {
        palette.extend_from_slice(&entry[..3]);
        alpha.push(entry[3]);
    }
    while alpha.last() == Some(&u8::MAX) {
        alpha.pop();
    }
    Some(Quantized {
        indices,
        palette,
        alpha: (!alpha.is_empty()).then_some(alpha),
    })
}

/// libwebp limits each side to 16,383 pixels.
const WEBP_MAX_SIDE: u32 = 16_383;

fn encode_webp(image: &DynamicImage, opts: &EncodeOptions) -> Result<Vec<u8>, String> {
    let (width, height) = (image.width(), image.height());
    if width > WEBP_MAX_SIDE || height > WEBP_MAX_SIDE {
        return Err(format!(
            "WebP images are limited to {WEBP_MAX_SIDE} pixels on a side"
        ));
    }
    let quality = f32::from(opts.quality.clamp(1, 100));
    let memory = match pixels(image) {
        Pixels::Rgba(rgba) => webp::Encoder::from_rgba(rgba.as_raw(), width, height)
            .encode_simple(opts.lossless, quality),
        _ => {
            let rgb = image.to_rgb8();
            webp::Encoder::from_rgb(rgb.as_raw(), width, height)
                .encode_simple(opts.lossless, quality)
        }
    }
    .map_err(|e| format!("{e:?}"))?;
    Ok(memory.to_vec())
}

fn encode_tiff(image: &DynamicImage) -> Result<Vec<u8>, String> {
    use tiff::encoder::{Compression, DeflateLevel, Predictor, TiffEncoder, colortype};

    let mut out = Cursor::new(Vec::new());
    let (width, height) = (image.width(), image.height());
    let mut encoder = TiffEncoder::new(&mut out)
        .map_err(|e| e.to_string())?
        .with_compression(Compression::Deflate(DeflateLevel::Balanced))
        .with_predictor(Predictor::Horizontal);
    let written = match pixels(image) {
        Pixels::Gray(g) => encoder.write_image::<colortype::Gray8>(width, height, g.as_raw()),
        Pixels::Rgb(rgb) => encoder.write_image::<colortype::RGB8>(width, height, rgb.as_raw()),
        Pixels::Rgba(rgba) => encoder.write_image::<colortype::RGBA8>(width, height, rgba.as_raw()),
    };
    written.map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}

/// rav1e's speed 6 balances size against time; AVIF encoding is slow at any setting.
const AVIF_SPEED: u8 = 6;

fn encode_avif(image: &DynamicImage, opts: &EncodeOptions) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    use image::codecs::avif::AvifEncoder;

    let mut out = Vec::new();
    let encoder =
        AvifEncoder::new_with_speed_quality(&mut out, AVIF_SPEED, opts.quality.clamp(1, 100));
    let (width, height) = (image.width(), image.height());
    match pixels(image) {
        Pixels::Rgba(rgba) => encoder.write_image(
            rgba.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        ),
        _ => {
            let rgb = image.to_rgb8();
            encoder.write_image(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
        }
    }
    .map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(windows)]
fn encode_heic(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = image.to_rgba8();
    dropwheel_wic::encode_heic(rgba.as_raw(), rgba.width(), rgba.height()).map_err(|e| {
        if e.missing_codec {
            "Windows needs Microsoft's \"HEVC Video Extensions\" to write HEIC files".to_string()
        } else {
            e.to_string()
        }
    })
}

#[cfg(not(windows))]
fn encode_heic(_image: &DynamicImage) -> Result<Vec<u8>, String> {
    Err("HEIC is written through Windows' own codecs".into())
}

fn write_with_image(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, format)
        .map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;

    fn gradient(width: u32, height: u32, alpha: bool) -> DynamicImage {
        let img = RgbaImage::from_fn(width, height, |x, y| {
            let a = if alpha {
                ((x * 255) / width.max(1)) as u8
            } else {
                255
            };
            Rgba([(x * 7) as u8, (y * 5) as u8, ((x + y) * 3) as u8, a])
        });
        DynamicImage::ImageRgba8(img)
    }

    /// Photo-like content: neighbouring pixels differ, so PNG cannot compress it away.
    fn noise(width: u32, height: u32, alpha: bool) -> DynamicImage {
        let mut state = 0x2545_f491_u32;
        let img = RgbaImage::from_fn(width, height, |x, _| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let [r, g, b, _] = state.to_le_bytes();
            let a = if alpha {
                ((x * 255) / width.max(1)) as u8
            } else {
                255
            };
            Rgba([r, g, b, a])
        });
        DynamicImage::ImageRgba8(img)
    }

    fn decode_bytes(bytes: &[u8]) -> DynamicImage {
        image::load_from_memory(bytes).unwrap()
    }

    #[test]
    fn round_trips_every_writable_format() {
        let img = gradient(48, 32, false);
        for kind in [
            FileKind::Jpeg,
            FileKind::Png,
            FileKind::Webp,
            FileKind::Bmp,
            FileKind::Tiff,
            FileKind::Gif,
            FileKind::Avif,
        ] {
            let bytes = encode(&img, kind, &EncodeOptions::default()).unwrap();
            assert_eq!(FileKind::classify(&bytes, ""), kind, "{kind:?}");
            if kind != FileKind::Avif {
                let back = decode_bytes(&bytes);
                assert_eq!((back.width(), back.height()), (48, 32), "{kind:?}");
            }
        }
    }

    #[test]
    fn jpeg_flattens_alpha_onto_white() {
        let clear = DynamicImage::ImageRgba8(RgbaImage::from_pixel(16, 16, Rgba([0, 0, 0, 0])));
        let back =
            decode_bytes(&encode(&clear, FileKind::Jpeg, &EncodeOptions::default()).unwrap());
        let p = back.to_rgb8().get_pixel(8, 8).0;
        assert!(p.iter().all(|c| *c > 245), "{p:?}");
    }

    #[test]
    fn png_keeps_alpha_and_drops_it_when_opaque() {
        let with = encode(
            &gradient(40, 40, true),
            FileKind::Png,
            &EncodeOptions::default(),
        )
        .unwrap();
        assert!(decode_bytes(&with).color().has_alpha());
        let without = encode(
            &gradient(40, 40, false),
            FileKind::Png,
            &EncodeOptions::default(),
        )
        .unwrap();
        assert!(!decode_bytes(&without).color().has_alpha());
    }

    #[test]
    fn png_palette_is_smaller_and_keeps_transparency() {
        let img = noise(128, 96, true);
        let full = encode(&img, FileKind::Png, &EncodeOptions::default()).unwrap();
        let opts = EncodeOptions {
            palette: Some(64),
            ..EncodeOptions::default()
        };
        let small = encode(&img, FileKind::Png, &opts).unwrap();
        assert!(
            small.len() < full.len(),
            "{} vs {}",
            small.len(),
            full.len()
        );
        let back = decode_bytes(&small).to_rgba8();
        assert!(back.get_pixel(0, 10).0[3] < 32);
    }

    #[test]
    fn lower_jpeg_quality_is_smaller() {
        let img = noise(200, 150, false);
        let size = |q| {
            let opts = EncodeOptions {
                quality: q,
                ..EncodeOptions::default()
            };
            encode(&img, FileKind::Jpeg, &opts).unwrap().len()
        };
        assert!(size(30) < size(90));
    }

    #[test]
    fn icc_profile_is_embedded() {
        let icc = vec![7u8; 200];
        let opts = EncodeOptions {
            icc: Some(&icc),
            ..EncodeOptions::default()
        };
        for kind in [FileKind::Jpeg, FileKind::Png] {
            let bytes = encode(&gradient(16, 16, false), kind, &opts).unwrap();
            let mut decoder = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .unwrap()
                .into_decoder()
                .unwrap();
            assert_eq!(
                decoder.icc_profile().unwrap(),
                Some(icc.clone()),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn unwritable_targets_are_errors() {
        let img = gradient(4, 4, false);
        for kind in [FileKind::Ico, FileKind::Pdf, FileKind::Unknown] {
            assert_eq!(
                encode(&img, kind, &EncodeOptions::default())
                    .unwrap_err()
                    .code(),
                "encode"
            );
        }
        assert!(!can_encode(FileKind::Pdf));
    }

    #[test]
    fn webp_rejects_oversized_images_without_panicking() {
        let wide = DynamicImage::ImageRgb8(RgbImage::new(WEBP_MAX_SIDE + 1, 1));
        assert!(encode(&wide, FileKind::Webp, &EncodeOptions::default()).is_err());
    }

    #[test]
    fn flatten_blends() {
        let half = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 128])));
        let p = flatten(&half, [255, 255, 255]).get_pixel(0, 0).0;
        assert_eq!(p, [127, 127, 127]);
    }
}
