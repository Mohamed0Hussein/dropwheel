//! Upright, screen-sized copies of the images and PDF pages a dialog works on, made once
//! each.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use dropwheel_core::image::DynamicImage;
use dropwheel_core::imaging::{self, EncodeOptions};
use dropwheel_core::{Error, FileKind, Result, pdf};

/// Longest side of a preview, enough for a large dialog on a high-DPI screen.
pub const PREVIEW_SIDE: u32 = 1600;
/// A few recent files; decoding a 50-megapixel photo again would be noticeable.
const CACHE_ENTRIES: usize = 6;

pub struct Preview {
    pub kind: FileKind,
    /// Size of the full, upright image, or of the PDF page in points.
    pub width: u32,
    pub height: u32,
    /// Pages in the file: 1 for images.
    pub pages: u32,
    pub file_bytes: u64,
    /// At most [`PREVIEW_SIDE`] on either side.
    pub image: DynamicImage,
    pub encoded: Vec<u8>,
    pub mime: &'static str,
}

type Entry = ((PathBuf, u32), Arc<Preview>);

#[derive(Default)]
pub struct Previews {
    recent: Mutex<VecDeque<Entry>>,
}

impl Previews {
    /// Decodes or renders on first use, so call it off the main thread. `page` counts
    /// from 0 and only matters for PDFs.
    pub fn get(&self, path: &Path, page: u32) -> Result<Arc<Preview>> {
        let key = (path.to_path_buf(), page);
        if let Some(hit) = self.lock().iter().find(|(k, _)| *k == key) {
            return Ok(hit.1.clone());
        }
        let preview = Arc::new(build(path, page)?);
        let mut recent = self.lock();
        recent.push_front((key, preview.clone()));
        recent.truncate(CACHE_ENTRIES);
        Ok(preview)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<Entry>> {
        self.recent.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn build(path: &Path, page: u32) -> Result<Preview> {
    let kind = FileKind::detect(path)?;
    let file_bytes = std::fs::metadata(path)
        .map_err(|e| Error::io(path, e))?
        .len();
    let (image, width, height, pages) = match kind {
        FileKind::Pdf => {
            let sizes = pdf::page_sizes(path)?;
            let (w, h) = sizes
                .get(page as usize)
                .copied()
                .ok_or_else(|| Error::InvalidOptions(format!("there is no page {}", page + 1)))?;
            let image = DynamicImage::ImageRgb8(pdf::render_page(path, page, PREVIEW_SIDE)?);
            let points = |v: f32| (v.round() as u32).max(1);
            let pages = u32::try_from(sizes.len()).unwrap_or(u32::MAX);
            (image, points(w), points(h), pages)
        }
        k if k.is_image() => {
            let decoded = imaging::decode(path, kind)?;
            let (width, height) = (decoded.image.width(), decoded.image.height());
            let image = if width > PREVIEW_SIDE || height > PREVIEW_SIDE {
                decoded.image.thumbnail(PREVIEW_SIDE, PREVIEW_SIDE)
            } else {
                decoded.image
            };
            (image, width, height, 1)
        }
        _ => {
            return Err(Error::Unsupported {
                action: "Previewing",
                kind,
            });
        }
    };
    // PNG keeps transparency visible; JPEG keeps photos small enough to move quickly.
    let (encoded, mime) = if image.color().has_alpha() {
        (
            imaging::encode(&image, FileKind::Png, &EncodeOptions::default())?,
            "image/png",
        )
    } else {
        let opts = EncodeOptions {
            quality: 85,
            ..EncodeOptions::default()
        };
        (
            imaging::encode(&image, FileKind::Jpeg, &opts)?,
            "image/jpeg",
        )
    };
    Ok(Preview {
        kind,
        width,
        height,
        pages,
        file_bytes,
        image,
        encoded,
        mime,
    })
}
