//! Upright, screen-sized copies of the images a dialog works on, decoded once each.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use dropwheel_core::image::DynamicImage;
use dropwheel_core::imaging::{self, EncodeOptions};
use dropwheel_core::{Error, FileKind, Result};

/// Longest side of a preview, enough for a large dialog on a high-DPI screen.
pub const PREVIEW_SIDE: u32 = 1600;
/// A few recent files; decoding a 50-megapixel photo again would be noticeable.
const CACHE_ENTRIES: usize = 6;

pub struct Preview {
    pub kind: FileKind,
    /// Size of the full, upright image.
    pub width: u32,
    pub height: u32,
    pub file_bytes: u64,
    /// At most [`PREVIEW_SIDE`] on either side.
    pub image: DynamicImage,
    pub encoded: Vec<u8>,
    pub mime: &'static str,
}

#[derive(Default)]
pub struct Previews {
    recent: Mutex<VecDeque<(PathBuf, Arc<Preview>)>>,
}

impl Previews {
    /// Decodes on first use, so call it off the main thread.
    pub fn get(&self, path: &Path) -> Result<Arc<Preview>> {
        if let Some(hit) = self.lock().iter().find(|(p, _)| p == path) {
            return Ok(hit.1.clone());
        }
        let preview = Arc::new(build(path)?);
        let mut recent = self.lock();
        recent.push_front((path.to_path_buf(), preview.clone()));
        recent.truncate(CACHE_ENTRIES);
        Ok(preview)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<(PathBuf, Arc<Preview>)>> {
        self.recent.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn build(path: &Path) -> Result<Preview> {
    let kind = FileKind::detect(path)?;
    if !kind.is_image() {
        return Err(Error::NotAvailable {
            action: format!("Previewing {} files", kind.label()),
        });
    }
    let file_bytes = std::fs::metadata(path)
        .map_err(|e| Error::io(path, e))?
        .len();
    let decoded = imaging::decode(path, kind)?;
    let (width, height) = (decoded.image.width(), decoded.image.height());
    let image = if width > PREVIEW_SIDE || height > PREVIEW_SIDE {
        decoded.image.thumbnail(PREVIEW_SIDE, PREVIEW_SIDE)
    } else {
        decoded.image
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
        file_bytes,
        image,
        encoded,
        mime,
    })
}
