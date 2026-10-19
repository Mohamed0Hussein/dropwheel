//! Making image files smaller, by quality or towards a target size.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use image::DynamicImage;
use image::imageops::FilterType;
use serde::Serialize;

use super::support;
use crate::action::{Action, ActionGroup, Compress, Options, Params};
use crate::error::{Error, Result};
use crate::imaging::{self, EncodeOptions};
use crate::kind::FileKind::{self, *};
use crate::output;

pub struct CompressImage;

pub static COMPRESS: CompressImage = CompressImage;

/// Lower than this, JPEG and WebP turn to mush; shrink the image instead.
const MIN_QUALITY: u8 = 10;
const MAX_QUALITY: u8 = 95;
/// Size estimates and target-size planning work on a copy this big.
pub const PROXY_SIDE: u32 = 1024;
/// Never shrink below this, whatever the target.
const MIN_SIDE: u32 = 16;

/// What compressing will produce, or did.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub bytes: u64,
    /// Encoder quality used.
    pub percent: u8,
    /// Factor applied to width and height: below 1 when the target needed a smaller
    /// image.
    pub scale: f32,
}

/// How each format turns `percent` into encoder settings. PNG is lossless, so below 90
/// it reduces colours (256 at 89, down to 64). TIFF is always lossless Deflate.
fn settings(kind: FileKind, percent: u8, icc: Option<&[u8]>) -> EncodeOptions<'_> {
    let percent = percent.clamp(1, 100);
    let palette = (kind == Png && percent < 90).then(|| 64 + (u16::from(percent) - 1) * 192 / 88);
    EncodeOptions {
        quality: percent,
        lossless: false,
        palette,
        icc,
    }
}

fn encode_at(
    image: &DynamicImage,
    kind: FileKind,
    percent: u8,
    icc: Option<&[u8]>,
) -> Result<Vec<u8>> {
    imaging::encode(image, kind, &settings(kind, percent, icc))
}

/// A copy no larger than `side` on either edge, for fast trial encodes.
pub fn proxy(image: &DynamicImage, side: u32) -> Cow<'_, DynamicImage> {
    if image.width() <= side && image.height() <= side {
        Cow::Borrowed(image)
    } else {
        Cow::Owned(image.thumbnail(side, side))
    }
}

fn scaled(image: &DynamicImage, scale: f32) -> Cow<'_, DynamicImage> {
    if scale >= 1.0 {
        return Cow::Borrowed(image);
    }
    let side = |n: u32| ((n as f32 * scale).round() as u32).max(MIN_SIDE.min(n));
    Cow::Owned(image.resize_exact(
        side(image.width()),
        side(image.height()),
        FilterType::Triangle,
    ))
}

/// Predicts the output size for an image of `full_width` × `full_height`, by encoding
/// `proxy` (a smaller copy of it) and scaling by the pixel count.
pub fn estimate(
    proxy: &DynamicImage,
    full_width: u32,
    full_height: u32,
    kind: FileKind,
    target: Compress,
) -> Result<Estimate> {
    let proxy_pixels = (u64::from(proxy.width()) * u64::from(proxy.height())).max(1);
    let ratio = (u64::from(full_width) * u64::from(full_height)) as f64 / proxy_pixels as f64;
    match target {
        Compress::Quality { percent } => {
            let bytes = encode_at(proxy, kind, percent, None)?.len() as f64 * ratio;
            Ok(Estimate {
                bytes: bytes as u64,
                percent,
                scale: 1.0,
            })
        }
        Compress::Size { bytes } => plan(proxy, ratio, kind, bytes, &Options::default()),
    }
}

/// Finds the highest quality whose output fits in `target` bytes, measuring on `image`
/// and multiplying by `ratio`. When even the lowest quality is too big, the image is
/// shrunk and the search repeated.
fn plan(
    image: &DynamicImage,
    ratio: f64,
    kind: FileKind,
    target: u64,
    opts: &Options,
) -> Result<Estimate> {
    let target = target.max(1);
    let mut scale = 1.0f32;
    let mut best = None;
    for _ in 0..6 {
        opts.check_cancelled()?;
        let candidate = scaled(image, scale);
        let size = |percent| -> Result<u64> {
            Ok((encode_at(&candidate, kind, percent, None)?.len() as f64 * ratio) as u64)
        };
        let at_max = size(MAX_QUALITY)?;
        if at_max <= target {
            return Ok(Estimate {
                bytes: at_max,
                percent: MAX_QUALITY,
                scale,
            });
        }
        let at_min = size(MIN_QUALITY)?;
        best = Some(Estimate {
            bytes: at_min,
            percent: MIN_QUALITY,
            scale,
        });
        if at_min > target {
            // Bytes scale roughly with pixel count, so shrink each side by the square root
            // of the overshoot, with a margin.
            let shrink = (target as f64 / at_min as f64).sqrt() as f32 * 0.9;
            scale *= shrink.clamp(0.1, 0.95);
            continue;
        }
        let (mut lo, mut hi) = (MIN_QUALITY, MAX_QUALITY);
        let mut fit = at_min;
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let s = size(mid)?;
            if s <= target {
                lo = mid;
                fit = s;
            } else {
                hi = mid;
            }
        }
        return Ok(Estimate {
            bytes: fit,
            percent: lo,
            scale,
        });
    }
    best.ok_or_else(|| Error::InvalidOptions("the target size could not be planned".into()))
}

impl Action for CompressImage {
    fn id(&self) -> &'static str {
        "compress"
    }

    fn label(&self) -> &'static str {
        "COMPRESS"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        matches!(kind, Jpeg | Png | Webp | Avif | Tiff | Pdf)
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        let Params::Compress(target) = opts.params else {
            return Err(Error::InvalidOptions(
                "compress needs a quality or a size".into(),
            ));
        };
        match target {
            Compress::Quality { percent } if !(1..=100).contains(&percent) => {
                return Err(Error::InvalidOptions(format!(
                    "quality must be 1 to 100, not {percent}"
                )));
            }
            Compress::Size { bytes: 0 } => {
                return Err(Error::InvalidOptions(
                    "the target size must be above zero".into(),
                ));
            }
            _ => {}
        }
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Compress",
                kind,
            });
        }
        support::require_image(kind, "Compressing")?;
        let (kind, decoded) = support::load(input, opts)?;
        let icc = decoded.icc.as_deref();

        let bytes = match target {
            Compress::Quality { percent } => encode_at(&decoded.image, kind, percent, icc)?,
            Compress::Size { bytes } => fit_to_size(&decoded.image, kind, bytes, icc, opts)?,
        };
        opts.check_cancelled()?;
        opts.report(0.95);
        output::write_bytes(input, out, "compressed", kind.extension(), &bytes)
    }
}

/// Plans on a proxy, which is fast even for AVIF, then encodes at full size and steps
/// down until the real file fits.
fn fit_to_size(
    image: &DynamicImage,
    kind: FileKind,
    target: u64,
    icc: Option<&[u8]>,
    opts: &Options,
) -> Result<Vec<u8>> {
    let small = proxy(image, PROXY_SIDE);
    let ratio = (u64::from(image.width()) * u64::from(image.height())) as f64
        / (u64::from(small.width()) * u64::from(small.height())).max(1) as f64;
    let plan = plan(&small, ratio, kind, target, opts)?;
    opts.report(0.55);

    let mut percent = plan.percent;
    let mut scale = plan.scale;
    let mut bytes = Vec::new();
    for attempt in 0..5u8 {
        opts.check_cancelled()?;
        bytes = encode_at(&scaled(image, scale), kind, percent, icc)?;
        opts.report(0.6 + f32::from(attempt) * 0.07);
        if bytes.len() as u64 <= target {
            break;
        }
        if percent > MIN_QUALITY {
            percent = percent.saturating_sub(10).max(MIN_QUALITY);
        } else {
            let shrink = (target as f64 / bytes.len() as f64).sqrt() as f32 * 0.95;
            scale *= shrink.clamp(0.1, 0.95);
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage};

    use super::*;

    fn photo(width: u32, height: u32) -> DynamicImage {
        let mut state = 0x9e37_79b9_u32;
        DynamicImage::ImageRgb8(RgbImage::from_fn(width, height, |x, y| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let n = (state & 0x3f) as u8;
            Rgb([
                ((x % 200) as u8).wrapping_add(n),
                ((y % 180) as u8).wrapping_add(n),
                (((x + y) % 150) as u8).wrapping_add(n),
            ])
        }))
    }

    #[test]
    fn png_quality_maps_to_colours() {
        assert_eq!(settings(Png, 95, None).palette, None);
        assert_eq!(settings(Png, 89, None).palette, Some(256));
        assert_eq!(settings(Png, 1, None).palette, Some(64));
        assert_eq!(settings(Jpeg, 50, None).palette, None);
    }

    #[test]
    fn estimate_tracks_quality() {
        let img = photo(300, 200);
        let low = estimate(&img, 300, 200, Jpeg, Compress::Quality { percent: 30 }).unwrap();
        let high = estimate(&img, 300, 200, Jpeg, Compress::Quality { percent: 90 }).unwrap();
        assert!(low.bytes < high.bytes);
        // Measured on a quarter-size proxy, scaled by pixel count.
        let small = img.thumbnail(150, 100);
        let scaled = estimate(&small, 300, 200, Jpeg, Compress::Quality { percent: 90 }).unwrap();
        assert!(scaled.bytes > high.bytes / 3 && scaled.bytes < high.bytes * 3);
    }

    #[test]
    fn plan_meets_the_target() {
        let img = photo(400, 300);
        let full = encode_at(&img, Jpeg, 95, None).unwrap().len() as u64;
        for target in [full / 2, full / 5, full / 50] {
            let est = plan(&img, 1.0, Jpeg, target, &Options::default()).unwrap();
            assert!(est.bytes <= target, "{target}: {est:?}");
        }
        // A generous target keeps full quality and size.
        let est = plan(&img, 1.0, Jpeg, full * 2, &Options::default()).unwrap();
        assert_eq!((est.percent, est.scale), (MAX_QUALITY, 1.0));
    }

    #[test]
    fn lossless_formats_shrink_to_fit() {
        let img = photo(200, 150);
        let full = encode_at(&img, Tiff, 100, None).unwrap().len() as u64;
        let est = plan(&img, 1.0, Tiff, full / 3, &Options::default()).unwrap();
        assert!(est.scale < 1.0 && est.bytes <= full / 3, "{est:?}");
    }

    #[test]
    fn fit_to_size_hits_the_target_at_full_size() {
        let img = photo(640, 480);
        let target = 12_000;
        let bytes = fit_to_size(&img, Jpeg, target, None, &Options::default()).unwrap();
        assert!(bytes.len() as u64 <= target, "{}", bytes.len());
        assert_eq!(FileKind::classify(&bytes, ""), Jpeg);
    }

    #[test]
    fn rejects_bad_settings() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.jpg");
        std::fs::write(&input, encode_at(&photo(8, 8), Jpeg, 90, None).unwrap()).unwrap();
        for params in [
            Params::None,
            Params::Compress(Compress::Quality { percent: 0 }),
            Params::Compress(Compress::Size { bytes: 0 }),
        ] {
            let err = COMPRESS
                .run(&input, &Options::new(params), dir.path())
                .unwrap_err();
            assert_eq!(err.code(), "invalid_options");
        }
    }
}
