//! Conversions that involve documents: images to PDF, PDF pages to images, and text
//! between PDF, DOCX and TXT.

use std::path::{Path, PathBuf};

use image::DynamicImage;

use super::support;
use crate::action::Options;
use crate::doc::{Document, Picture};
use crate::error::{Error, Result};
use crate::imaging::{self, EncodeOptions};
use crate::kind::FileKind::{self, *};
use crate::output;
use crate::pdf::{self, write::PicturePage};
use crate::{docx, txt};

/// Pages rendered from a PDF are this sharp.
const PAGE_DPI: f32 = 200.0;
/// An image is placed on an A4 page with this margin.
const MARGIN: f32 = 36.0;
const A4: (f32, f32) = (595.28, 841.89);

/// Reads any document Dropwheel can convert from.
fn read_document(input: &Path, kind: FileKind, opts: &Options) -> Result<Document> {
    let doc = match kind {
        Docx => docx::read::read(input)?,
        Txt => txt::to_document(&txt::read_text(input)?),
        Pdf => pdf::read::document(input, opts)?,
        _ => {
            return Err(Error::Unsupported {
                action: "Converting",
                kind,
            });
        }
    };
    opts.check_cancelled()?;
    opts.report(0.5);
    Ok(doc)
}

/// Converts the document `input` of `kind` to `target`: PDF, DOCX or TXT.
pub fn document(
    input: &Path,
    kind: FileKind,
    target: FileKind,
    opts: &Options,
    out: &Path,
) -> Result<PathBuf> {
    let bytes = match (kind, target) {
        (Pdf, Txt) => {
            let text = pdf::read::text(input, opts)?;
            text.into_bytes()
        }
        (_, Txt) => read_document(input, kind, opts)?.to_text().into_bytes(),
        (_, Docx) => docx::write::write(&read_document(input, kind, opts)?)?,
        (_, Pdf) => pdf::write::document(&read_document(input, kind, opts)?, opts)?,
        _ => {
            return Err(Error::Unsupported {
                action: "This conversion",
                kind,
            });
        }
    };
    opts.check_cancelled()?;
    opts.report(0.95);
    output::write_bytes(input, out, "converted", target.extension(), &bytes)
}

/// The picture of an image file to place in a PDF: a JPEG as it is when it needs no
/// turning, photos as JPEG, and anything with transparency or sharp edges as PNG.
fn picture_of(input: &Path, kind: FileKind, opts: &Options) -> Result<Picture> {
    let (_, decoded) = support::load(input, opts)?;
    let image = &decoded.image;
    let size = Some((image.width() as f32, image.height() as f32));
    if kind == Jpeg && !decoded.reoriented {
        let data = std::fs::read(input).map_err(|e| Error::io(input, e))?;
        if let Some(p) = Picture::from_bytes(data, size) {
            return Ok(p);
        }
    }
    let photo = matches!(kind, Jpeg | Webp | Avif | Heic) && !image.color().has_alpha();
    let (target, encode) = if photo {
        (
            Jpeg,
            EncodeOptions {
                quality: 92,
                icc: decoded.icc.as_deref(),
                ..EncodeOptions::default()
            },
        )
    } else {
        (
            Png,
            EncodeOptions {
                lossless: true,
                icc: decoded.icc.as_deref(),
                ..EncodeOptions::default()
            },
        )
    };
    let bytes = imaging::encode(image, target, &encode)?;
    Picture::from_bytes(bytes, size).ok_or(Error::Internal)
}

/// Puts an image on an A4 page, turned to suit its shape, within the margins and no
/// larger than its own size at 96 dpi.
pub fn image_to_pdf(input: &Path, kind: FileKind, opts: &Options, out: &Path) -> Result<PathBuf> {
    let picture = picture_of(input, kind, opts)?;
    let (pw, ph) = (picture.pixel_width as f32, picture.pixel_height as f32);
    let (page_w, page_h) = if pw > ph { (A4.1, A4.0) } else { A4 };
    let natural = 0.75;
    let scale = ((page_w - 2.0 * MARGIN) / pw)
        .min((page_h - 2.0 * MARGIN) / ph)
        .min(natural);
    let (w, h) = (pw * scale, ph * scale);
    let page = PicturePage {
        picture,
        width: page_w,
        height: page_h,
        x: (page_w - w) / 2.0,
        y: (page_h - h) / 2.0,
        picture_width: w,
        picture_height: h,
    };
    opts.report(0.6);
    let bytes = pdf::write::pictures(&[page], opts)?;
    opts.check_cancelled()?;
    output::write_bytes(input, out, "converted", "pdf", &bytes)
}

/// Renders a PDF's pages as images: one file for a single page, otherwise a folder
/// with a file per page.
pub fn pdf_to_images(
    input: &Path,
    target: FileKind,
    opts: &Options,
    out: &Path,
) -> Result<PathBuf> {
    let count = pdf::page_sizes(input)?.len();
    let (target, mut encode) = support::rewrite_as(target);
    encode.lossless = !matches!(target, Jpeg | Webp | Avif | Heic);
    encode.quality = 90;
    let encode_page = |image: image::RgbImage| -> Result<Vec<u8>> {
        imaging::encode(&DynamicImage::ImageRgb8(image), target, &encode)
    };
    if count == 1 {
        let mut bytes = None;
        pdf::render_pages(input, PAGE_DPI, opts, |_, _, image| {
            bytes = Some(encode_page(image)?);
            Ok(())
        })?;
        let bytes = bytes.ok_or(Error::Internal)?;
        return output::write_bytes(input, out, "converted", target.extension(), &bytes);
    }
    let folder = output::new_folder(input, out, "pages")?;
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let digits = count.to_string().len();
    pdf::render_pages(input, PAGE_DPI, opts, |i, _, image| {
        let bytes = encode_page(image)?;
        let name = format!("{stem}_page{:0digits$}.{}", i + 1, target.extension());
        folder.write(&name, &bytes)
    })?;
    opts.check_cancelled()?;
    folder.finish()
}

/// A PDF made of the pictures of `pages`, each the size of its page, as the redacted
/// copy of a PDF is.
pub fn pages_to_pdf(pages: Vec<(image::RgbImage, f32, f32)>, opts: &Options) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(pages.len());
    for (image, w, h) in pages {
        let bytes = imaging::encode(
            &DynamicImage::ImageRgb8(image),
            Jpeg,
            &EncodeOptions {
                quality: 88,
                ..EncodeOptions::default()
            },
        )?;
        let picture = Picture::from_bytes(bytes, Some((w, h))).ok_or(Error::Internal)?;
        out.push(PicturePage::full(picture, w, h));
    }
    pdf::write::pictures(&out, opts)
}
