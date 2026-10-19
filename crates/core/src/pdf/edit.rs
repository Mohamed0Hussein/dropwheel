//! Changing existing PDFs with `lopdf`: cropping pages, recompressing their pictures and
//! removing metadata. Everything else in the file is kept as it is.
//!
//! Encrypted PDFs are refused: saving one would drop its protection.

use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, RgbImage};
use lopdf::{Document, Object, ObjectId, SaveOptions};

use crate::action::{Compress, Crop, Options};
use crate::error::{Error, Result};
use crate::imaging::{self, EncodeOptions};
use crate::kind::FileKind;

/// No stream is decompressed past this, so a crafted file cannot exhaust memory.
const MAX_STREAM: usize = 512 * 1024 * 1024;
/// Pictures smaller than this are left alone; recompressing them gains nothing.
const MIN_PICTURE_BYTES: usize = 8 * 1024;

fn decode_error(path: &Path, reason: impl std::fmt::Display) -> Error {
    Error::Decode {
        path: path.to_path_buf(),
        kind: FileKind::Pdf,
        reason: reason.to_string(),
    }
}

/// Opens `path` for editing.
pub fn load(path: &Path) -> Result<Document> {
    let data = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let options = lopdf::LoadOptions {
        max_decompressed_size: Some(MAX_STREAM),
        ..lopdf::LoadOptions::default()
    };
    let doc = Document::load_mem_with_options(&data, options).map_err(|e| match e {
        lopdf::Error::Decryption(_) => Error::Protected {
            path: path.to_path_buf(),
        },
        e => decode_error(path, e),
    })?;
    if doc.trailer.has(b"Encrypt") || doc.is_encrypted() {
        return Err(Error::Protected {
            path: path.to_path_buf(),
        });
    }
    if doc.get_pages().is_empty() {
        return Err(decode_error(path, "the PDF has no pages"));
    }
    Ok(doc)
}

/// Writes `doc` compactly: unused objects dropped, streams compressed, and objects
/// packed into object streams.
pub fn save(doc: &mut Document) -> Result<Vec<u8>> {
    doc.prune_objects();
    doc.compress();
    if doc.version.as_str() < "1.5" {
        doc.version = "1.5".into();
    }
    let options = SaveOptions::builder()
        .use_object_streams(true)
        .use_xref_streams(true)
        .build();
    let mut out = Vec::new();
    doc.save_with_options(&mut out, options)
        .map_err(|e| Error::Encode {
            kind: FileKind::Pdf,
            reason: e.to_string(),
        })?;
    Ok(out)
}

/// A page attribute, looked up the page tree when the page does not set it.
fn inherited(doc: &Document, page: ObjectId, key: &[u8]) -> Option<Object> {
    let mut id = page;
    for _ in 0..32 {
        let dict = doc.get_dictionary(id).ok()?;
        if let Ok(value) = dict.get(key) {
            return Some(match value {
                Object::Reference(r) => doc.get_object(*r).ok()?.clone(),
                v => v.clone(),
            });
        }
        id = dict.get(b"Parent").ok()?.as_reference().ok()?;
    }
    None
}

fn rect(obj: &Object) -> Option<[f32; 4]> {
    let a = obj.as_array().ok()?;
    if a.len() != 4 {
        return None;
    }
    let mut v = [0.0f32; 4];
    for (slot, o) in v.iter_mut().zip(a) {
        *slot = o.as_float().ok()?;
    }
    Some([
        v[0].min(v[2]),
        v[1].min(v[3]),
        v[0].max(v[2]),
        v[1].max(v[3]),
    ])
}

/// The visible area of a page in its own units, and its rotation in degrees.
fn visible(doc: &Document, page: ObjectId) -> Option<([f32; 4], i64)> {
    let media = inherited(doc, page, b"MediaBox").and_then(|o| rect(&o))?;
    let crop = inherited(doc, page, b"CropBox")
        .and_then(|o| rect(&o))
        .map(|c| {
            [
                c[0].max(media[0]),
                c[1].max(media[1]),
                c[2].min(media[2]),
                c[3].min(media[3]),
            ]
        })
        .filter(|c| c[2] > c[0] && c[3] > c[1])
        .unwrap_or(media);
    let rotate = inherited(doc, page, b"Rotate")
        .and_then(|o| o.as_i64().ok())
        .unwrap_or(0)
        .rem_euclid(360);
    Some((crop, rotate - rotate % 90))
}

/// The part of a page `fx0..fx1` across and `fy0..fy1` down, as fractions of the page
/// as displayed, in the page's own coordinates.
fn displayed_to_user(b: [f32; 4], rotate: i64, f: [f32; 4]) -> [f32; 4] {
    let [llx, lly, urx, ury] = b;
    let (w, h) = (urx - llx, ury - lly);
    let [fx0, fy0, fx1, fy1] = f;
    match rotate {
        90 => [llx + fy0 * w, lly + fx0 * h, llx + fy1 * w, lly + fx1 * h],
        180 => [
            llx + (1.0 - fx1) * w,
            lly + fy0 * h,
            llx + (1.0 - fx0) * w,
            lly + fy1 * h,
        ],
        270 => [
            llx + (1.0 - fy1) * w,
            lly + (1.0 - fx1) * h,
            llx + (1.0 - fy0) * w,
            lly + (1.0 - fx0) * h,
        ],
        _ => [llx + fx0 * w, ury - fy1 * h, llx + fx1 * w, ury - fy0 * h],
    }
}

/// Crops every page to the same part of it. `crop` is in the units of a first page
/// `ref_width` × `ref_height`, as the Crop dialog draws it.
pub fn crop(path: &Path, crop: &Crop, opts: &Options) -> Result<Vec<u8>> {
    if crop.width == 0 || crop.height == 0 || crop.ref_width == 0 || crop.ref_height == 0 {
        return Err(Error::InvalidOptions("the crop rectangle is empty".into()));
    }
    let fraction = |v: u32, of: u32| (v as f32 / of as f32).clamp(0.0, 1.0);
    let f = [
        fraction(crop.x, crop.ref_width),
        fraction(crop.y, crop.ref_height),
        fraction(crop.x + crop.width, crop.ref_width),
        fraction(crop.y + crop.height, crop.ref_height),
    ];
    if f[2] - f[0] < 1e-4 || f[3] - f[1] < 1e-4 {
        return Err(Error::InvalidOptions("the crop rectangle is empty".into()));
    }
    let mut doc = load(path)?;
    opts.check_cancelled()?;
    let pages: Vec<ObjectId> = doc.get_pages().into_values().collect();
    for page in pages {
        let Some((b, rotate)) = visible(&doc, page) else {
            continue;
        };
        let r = displayed_to_user(b, rotate, f);
        let boxed = Object::Array(r.iter().map(|v| Object::Real(*v)).collect());
        if let Ok(dict) = doc.get_dictionary_mut(page) {
            dict.set("CropBox", boxed.clone());
            // Printers and some viewers use the trim box when there is one.
            if dict.has(b"TrimBox") {
                dict.set("TrimBox", boxed);
            }
        }
    }
    opts.report(0.8);
    save(&mut doc)
}

/// Removes the document information dictionary, XMP metadata wherever it is attached,
/// and private application data.
pub fn strip_metadata(path: &Path, opts: &Options) -> Result<Vec<u8>> {
    let mut doc = load(path)?;
    opts.check_cancelled()?;
    doc.trailer.remove(b"Info");
    for object in doc.objects.values_mut() {
        let dict = match object {
            Object::Dictionary(d) => d,
            Object::Stream(s) => &mut s.dict,
            _ => continue,
        };
        dict.remove(b"Metadata");
        dict.remove(b"PieceInfo");
        dict.remove(b"LastModified");
    }
    opts.report(0.8);
    save(&mut doc)
}

/// A picture in the PDF that can be recompressed.
struct Candidate {
    id: ObjectId,
    image: DynamicImage,
    /// Bytes it takes now.
    size: usize,
    /// It has a soft mask whose colours were premultiplied for this exact picture, so
    /// it keeps its size.
    keep_size: bool,
}

fn components(doc: &Document, cs: &Object) -> Option<u8> {
    let cs = match cs {
        Object::Reference(r) => doc.get_object(*r).ok()?,
        o => o,
    };
    match cs {
        Object::Name(n) if n == b"DeviceRGB" => Some(3),
        Object::Name(n) if n == b"DeviceGray" => Some(1),
        Object::Array(a) if a.first().and_then(|o| o.as_name().ok()) == Some(b"ICCBased") => {
            let stream = doc
                .get_object(a.get(1)?.as_reference().ok()?)
                .ok()?
                .as_stream()
                .ok()?;
            match stream.dict.get(b"N").ok()?.as_i64().ok()? {
                3 => Some(3),
                1 => Some(1),
                _ => None,
            }
        }
        _ => None,
    }
}

fn candidates(doc: &Document, opts: &Options) -> Result<Vec<Candidate>> {
    let mut found = Vec::new();
    for (id, object) in &doc.objects {
        opts.check_cancelled()?;
        let Object::Stream(stream) = object else {
            continue;
        };
        let d = &stream.dict;
        if d.get(b"Subtype").and_then(Object::as_name).ok() != Some(b"Image".as_slice())
            || d.get(b"ImageMask")
                .and_then(Object::as_bool)
                .unwrap_or(false)
            || d.has(b"Decode")
            || d.get(b"BitsPerComponent").and_then(Object::as_i64).ok() != Some(8)
            || stream.content.len() < MIN_PICTURE_BYTES
        {
            continue;
        }
        let Some(n) = d.get(b"ColorSpace").ok().and_then(|cs| components(doc, cs)) else {
            continue;
        };
        let (Ok(w), Ok(h)) = (
            d.get(b"Width").and_then(Object::as_i64),
            d.get(b"Height").and_then(Object::as_i64),
        ) else {
            continue;
        };
        let (Ok(w), Ok(h)) = (u32::try_from(w), u32::try_from(h)) else {
            continue;
        };
        if w < 32 || h < 32 || w > imaging::MAX_SIDE || h > imaging::MAX_SIDE {
            continue;
        }
        let filters: Vec<Vec<u8>> = stream
            .filters()
            .map(|f| f.into_iter().map(<[u8]>::to_vec).collect())
            .unwrap_or_default();
        let image = match filters
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [b"DCTDecode"] => {
                let mut reader = image::ImageReader::with_format(
                    std::io::Cursor::new(&stream.content),
                    image::ImageFormat::Jpeg,
                );
                reader.limits(imaging::limits());
                match reader.decode() {
                    Ok(img) if img.width() == w && img.height() == h => img,
                    _ => continue,
                }
            }
            [b"FlateDecode"] | [] => {
                let expected = w as usize * h as usize * usize::from(n);
                let Ok(raw) = stream.decompressed_content_with_limit(expected + 16) else {
                    continue;
                };
                if raw.len() < expected {
                    continue;
                }
                let raw = raw[..expected].to_vec();
                let decoded = if n == 3 {
                    RgbImage::from_raw(w, h, raw).map(DynamicImage::ImageRgb8)
                } else {
                    GrayImage::from_raw(w, h, raw).map(DynamicImage::ImageLuma8)
                };
                match decoded {
                    Some(img) => img,
                    None => continue,
                }
            }
            _ => continue,
        };
        let keep_size = d
            .get(b"SMask")
            .and_then(Object::as_reference)
            .ok()
            .and_then(|r| doc.get_dictionary(r).ok())
            .is_some_and(|m| m.has(b"Matte"));
        found.push(Candidate {
            id: *id,
            image,
            size: stream.content.len(),
            keep_size,
        });
    }
    Ok(found)
}

/// How hard to squeeze pictures: JPEG quality and the longest side allowed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Setting {
    quality: u8,
    max_side: Option<u32>,
}

impl Setting {
    /// The setting for a percentage: quality follows it, and below 90% large pictures
    /// are also scaled down, to 1120 px on the longest side at 10% and 3680 px at 90%.
    fn for_percent(percent: u8) -> Setting {
        let p = percent.clamp(1, 100);
        Setting {
            quality: p.clamp(10, 95),
            max_side: (p < 90).then(|| 800 + u32::from(p) * 32),
        }
    }
}

fn reencode(c: &Candidate, s: Setting) -> Option<(Vec<u8>, u32, u32)> {
    let scaled;
    let image = match s.max_side {
        Some(max) if !c.keep_size && c.image.width().max(c.image.height()) > max => {
            scaled = c.image.resize(max, max, FilterType::Triangle);
            &scaled
        }
        _ => &c.image,
    };
    let opts = EncodeOptions {
        quality: s.quality,
        ..EncodeOptions::default()
    };
    let bytes = imaging::encode(image, FileKind::Jpeg, &opts).ok()?;
    Some((bytes, image.width(), image.height()))
}

/// Replaces each picture that comes out smaller.
fn apply(doc: &mut Document, candidates: &[Candidate], s: Setting, opts: &Options) -> Result<()> {
    for (i, c) in candidates.iter().enumerate() {
        opts.check_cancelled()?;
        opts.report(0.2 + 0.6 * i as f32 / candidates.len().max(1) as f32);
        let Some((bytes, w, h)) = reencode(c, s) else {
            continue;
        };
        if bytes.len() >= c.size {
            continue;
        }
        if let Ok(Object::Stream(stream)) = doc.get_object_mut(c.id) {
            stream
                .dict
                .set("Filter", Object::Name(b"DCTDecode".to_vec()));
            stream.dict.remove(b"DecodeParms");
            stream.dict.set("Width", i64::from(w));
            stream.dict.set("Height", i64::from(h));
            stream.dict.set("BitsPerComponent", 8);
            stream.set_content(bytes);
            stream.allows_compression = false;
        }
    }
    // Thumbnails some editors embed are redrawn by every viewer anyway.
    for page in doc.get_pages().into_values().collect::<Vec<_>>() {
        if let Ok(dict) = doc.get_dictionary_mut(page) {
            dict.remove(b"Thumb");
        }
    }
    Ok(())
}

fn compressed(
    doc: &Document,
    candidates: &[Candidate],
    s: Setting,
    opts: &Options,
) -> Result<Vec<u8>> {
    let mut copy = doc.clone();
    apply(&mut copy, candidates, s, opts)?;
    save(&mut copy)
}

/// Compresses the PDF at `path`, by quality or towards a size in bytes.
pub fn compress(path: &Path, target: Compress, opts: &Options) -> Result<Vec<u8>> {
    let doc = load(path)?;
    let found = candidates(&doc, opts)?;
    opts.report(0.2);
    match target {
        Compress::Quality { percent } => {
            compressed(&doc, &found, Setting::for_percent(percent), opts)
        }
        Compress::Size { bytes } => {
            let attempt = |s: Setting| compressed(&doc, &found, s, opts);
            let first = attempt(Setting::for_percent(95))?;
            if first.len() as u64 <= bytes {
                return Ok(first);
            }
            let low = attempt(Setting::for_percent(10))?;
            if low.len() as u64 > bytes {
                // Even the lowest quality is too big: smaller pictures, then the
                // smallest file found.
                let mut smallest = low;
                for side in [900u32, 640, 480] {
                    let out = attempt(Setting {
                        quality: 10,
                        max_side: Some(side),
                    })?;
                    if out.len() < smallest.len() {
                        smallest = out;
                    }
                    if smallest.len() as u64 <= bytes {
                        break;
                    }
                }
                return Ok(smallest);
            }
            // The highest quality that fits, by halving.
            let mut best = low;
            let (mut lo, mut hi) = (10u8, 95u8);
            while hi - lo > 5 {
                let mid = lo + (hi - lo) / 2;
                let out = attempt(Setting::for_percent(mid))?;
                if out.len() as u64 <= bytes {
                    lo = mid;
                    best = out;
                } else {
                    hi = mid;
                }
            }
            Ok(best)
        }
    }
}

/// Predicts the size of [`compress`] with `percent`, from small copies of the pictures,
/// without writing the PDF.
pub fn estimate(path: &Path, target: Compress) -> Result<u64> {
    let total = std::fs::metadata(path)
        .map_err(|e| Error::io(path, e))?
        .len();
    let doc = load(path)?;
    let found = candidates(&doc, &Options::default())?;
    let pictures: u64 = found.iter().map(|c| c.size as u64).sum();
    let rest = total.saturating_sub(pictures);
    let estimate_at = |s: Setting| -> u64 {
        found
            .iter()
            .map(|c| {
                let (w, h) = (c.image.width(), c.image.height());
                let (tw, th) = match s.max_side {
                    Some(max) if !c.keep_size && w.max(h) > max => {
                        let k = max as f32 / w.max(h) as f32;
                        ((w as f32 * k) as u32, (h as f32 * k) as u32)
                    }
                    _ => (w, h),
                };
                let proxy = c.image.thumbnail(512.min(tw).max(1), 512.min(th).max(1));
                let opts = EncodeOptions {
                    quality: s.quality,
                    ..EncodeOptions::default()
                };
                let bytes = imaging::encode(&proxy, FileKind::Jpeg, &opts)
                    .map(|b| b.len() as f64)
                    .unwrap_or(c.size as f64);
                let ratio = (f64::from(tw) * f64::from(th))
                    / (f64::from(proxy.width()) * f64::from(proxy.height())).max(1.0);
                ((bytes * ratio) as u64).min(c.size as u64)
            })
            .sum()
    };
    let setting = match target {
        Compress::Quality { percent } => Setting::for_percent(percent),
        Compress::Size { bytes } => {
            let mut chosen = Setting::for_percent(10);
            for p in (10..=95).rev().step_by(5) {
                let s = Setting::for_percent(p);
                if rest + estimate_at(s) <= bytes {
                    chosen = s;
                    break;
                }
            }
            chosen
        }
    };
    Ok(rest + estimate_at(setting))
}

/// Text from a PDF string: UTF-16 or UTF-8 with a byte order mark, or PDFDocEncoding.
pub fn text_string(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    const HIGH: [char; 33] = [
        '•', '†', '‡', '…', '—', '–', 'ƒ', '⁄', '‹', '›', '−', '‰', '„', '“', '”', '‘', '’', '‚',
        '™', 'ﬁ', 'ﬂ', 'Ł', 'Œ', 'Š', 'Ÿ', 'Ž', 'ı', 'ł', 'œ', 'š', 'ž', '\u{FFFD}', '€',
    ];
    const LOW: [char; 8] = ['˘', 'ˇ', 'ˆ', '˙', '˝', '˛', '˚', '˜'];
    bytes
        .iter()
        .map(|&b| match b {
            0x18..=0x1F => LOW[usize::from(b - 0x18)],
            0x80..=0xA0 => HIGH[usize::from(b - 0x80)],
            b => char::from(b),
        })
        .collect()
}

/// "D:20260119143000+02'00'" as "2026-01-19 14:30:00 +02:00"; other text as it is.
fn date(text: &str) -> String {
    let Some(d) = text.strip_prefix("D:") else {
        return text.to_string();
    };
    let digits: String = d.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() < 4 {
        return text.to_string();
    }
    let part = |a: usize, b: usize, default: &str| digits.get(a..b).unwrap_or(default).to_string();
    let mut out = format!(
        "{}-{}-{} {}:{}:{}",
        part(0, 4, "0000"),
        part(4, 6, "01"),
        part(6, 8, "01"),
        part(8, 10, "00"),
        part(10, 12, "00"),
        part(12, 14, "00")
    );
    let zone = &d[digits.len()..];
    if zone.starts_with('Z') {
        out.push_str(" UTC");
    } else if let Some(sign) = zone.chars().next().filter(|c| *c == '+' || *c == '-') {
        let z: String = zone[1..].chars().filter(char::is_ascii_digit).collect();
        if z.len() >= 2 {
            out.push_str(&format!(
                " {sign}{}:{}",
                &z[..2],
                z.get(2..4).unwrap_or("00")
            ));
        }
    }
    out
}

/// The metadata of the PDF at `path`: its document information, XMP packet and basic
/// facts, as group, name and value.
pub fn metadata(path: &Path) -> Result<Vec<(&'static str, String, String)>> {
    let doc = load(path)?;
    let mut rows = Vec::new();
    let info = doc.trailer.get(b"Info").ok().and_then(|o| match o {
        Object::Reference(r) => doc.get_dictionary(*r).ok(),
        Object::Dictionary(d) => Some(d),
        _ => None,
    });
    if let Some(info) = info {
        for (key, value) in info.iter() {
            let value = match value {
                Object::Reference(r) => doc.get_object(*r).ok(),
                v => Some(v),
            };
            let text = match value {
                Some(Object::String(bytes, _)) => text_string(bytes),
                Some(Object::Name(n)) => String::from_utf8_lossy(n).into_owned(),
                Some(Object::Integer(i)) => i.to_string(),
                Some(Object::Boolean(b)) => b.to_string(),
                _ => continue,
            };
            if text.trim().is_empty() {
                continue;
            }
            let key = String::from_utf8_lossy(key).into_owned();
            let (name, text) = match key.as_str() {
                "Author" => ("Author".to_string(), text),
                "Creator" => ("Created with".to_string(), text),
                "Producer" => ("PDF made by".to_string(), text),
                "CreationDate" => ("Created".to_string(), date(&text)),
                "ModDate" => ("Modified".to_string(), date(&text)),
                _ => (key, text),
            };
            rows.push(("Document", name, text));
        }
    }
    let xmp = doc
        .catalog()
        .ok()
        .and_then(|c| c.get(b"Metadata").ok())
        .and_then(|o| o.as_reference().ok())
        .and_then(|r| doc.get_object(r).ok())
        .and_then(|o| o.as_stream().ok());
    if let Some(stream) = xmp {
        rows.push((
            "XMP",
            "XMP packet".into(),
            format!("{} bytes", stream.content.len()),
        ));
    }
    let attached = doc
        .objects
        .values()
        .filter(|o| {
            o.as_dict()
                .ok()
                .or_else(|| o.as_stream().ok().map(|s| &s.dict))
                .is_some_and(|d| d.has(b"PieceInfo"))
        })
        .count();
    if attached > 0 {
        rows.push((
            "Application",
            "Private application data".into(),
            format!("{attached} entries"),
        ));
    }
    rows.push(("File", "PDF version".into(), doc.version.clone()));
    rows.push(("File", "Pages".into(), doc.get_pages().len().to_string()));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn rotation_maps_the_displayed_rectangle() {
        let b = [0.0, 0.0, 200.0, 300.0];
        // The left half of the displayed page.
        let half = [0.0, 0.0, 0.5, 1.0];
        assert_eq!(displayed_to_user(b, 0, half), [0.0, 0.0, 100.0, 300.0]);
        // Turned a quarter clockwise, the displayed left half is the bottom of the page.
        assert_eq!(displayed_to_user(b, 90, half), [0.0, 0.0, 200.0, 150.0]);
        assert_eq!(displayed_to_user(b, 180, half), [100.0, 0.0, 200.0, 300.0]);
        assert_eq!(displayed_to_user(b, 270, half), [0.0, 150.0, 200.0, 300.0]);
    }

    #[test]
    fn crops_every_page() {
        let crop = Crop {
            x: 0,
            y: 0,
            width: 50,
            height: 50,
            ref_width: 100,
            ref_height: 100,
        };
        let out = super::crop(&fixture("text.pdf"), &crop, &Options::default()).unwrap();
        let doc = Document::load_mem(&out).unwrap();
        for page in doc.get_pages().into_values() {
            let b = rect(&inherited(&doc, page, b"CropBox").unwrap()).unwrap();
            assert!((b[2] - b[0] - 297.6).abs() < 1.0, "{b:?}");
            // The top half of the page: y from the middle up.
            assert!((b[3] - 841.9).abs() < 1.0, "{b:?}");
        }
    }

    #[test]
    fn metadata_is_removed() {
        let out = strip_metadata(&fixture("text.pdf"), &Options::default()).unwrap();
        let doc = Document::load_mem(&out).unwrap();
        assert!(!doc.trailer.has(b"Info"));
        assert!(!doc.catalog().unwrap().has(b"Metadata"));
        assert_eq!(doc.get_pages().len(), 2);
    }

    #[test]
    fn metadata_is_listed() {
        let rows = metadata(&fixture("text.pdf")).unwrap();
        let has = |n: &str, v: &str| rows.iter().any(|(_, name, value)| name == n && value == v);
        assert!(has("Title", "Quarterly report"), "{rows:#?}");
        assert!(has("Author", "Test Author"));
        assert!(has("Pages", "2"));
        assert_eq!(
            date("D:20260119143000+02'00'"),
            "2026-01-19 14:30:00 +02:00"
        );
        assert_eq!(text_string(b"\xFE\xFF\x06\x45"), "م");
        assert_eq!(text_string(b"caf\xe9 \x80"), "café •");
    }

    #[test]
    fn encrypted_files_are_refused() {
        let err = strip_metadata(&fixture("protected.pdf"), &Options::default()).unwrap_err();
        assert_eq!(err.code(), "protected");
        let err = load(&fixture("broken.pdf")).unwrap_err();
        assert_eq!(err.code(), "decode", "{err}");
    }

    #[test]
    fn pictures_get_smaller() {
        // A PDF with a large, noisy picture stored losslessly.
        let mut state = 1u32;
        let img = RgbImage::from_fn(900, 700, |x, y| {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let n = (state >> 24) as u8 / 4;
            image::Rgb([
                ((x / 4) as u8).wrapping_add(n),
                ((y / 3) as u8).wrapping_add(n),
                (((x + y) / 7) as u8).wrapping_add(n),
            ])
        });
        let mut png = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let picture = crate::doc::Picture::from_bytes(png, Some((450.0, 350.0))).unwrap();
        let pages = [super::super::write::PicturePage::full(
            picture, 450.0, 350.0,
        )];
        let pdf = super::super::write::pictures(&pages, &Options::default()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.pdf");
        std::fs::write(&path, &pdf).unwrap();

        let small = compress(
            &path,
            Compress::Quality { percent: 40 },
            &Options::default(),
        )
        .unwrap();
        assert!(
            small.len() * 3 < pdf.len(),
            "{} vs {}",
            small.len(),
            pdf.len()
        );
        let estimate = estimate(&path, Compress::Quality { percent: 40 }).unwrap();
        assert!(
            estimate > small.len() as u64 / 3 && estimate < small.len() as u64 * 3,
            "{estimate} vs {}",
            small.len()
        );
        let target = pdf.len() as u64 / 4;
        let sized = compress(&path, Compress::Size { bytes: target }, &Options::default()).unwrap();
        assert!(sized.len() as u64 <= target, "{} > {target}", sized.len());
        assert!(Document::load_mem(&sized).is_ok());
    }
}
