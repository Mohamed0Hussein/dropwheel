//! PDFium: loading it, and reading pages through it.
//!
//! PDFium is a C++ library loaded at run time. It is not thread-safe, so every use goes
//! through [`with`], which holds one lock.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, PoisonError};

use image::RgbImage;
use pdfium_render::prelude::{
    PdfDocument, PdfPage, PdfRenderConfig, Pdfium, PdfiumError, PdfiumInternalError,
};
use unicode_normalization::UnicodeNormalization;

use crate::error::{Error, Result};
use crate::kind::FileKind;

static LIBRARY: OnceLock<PathBuf> = OnceLock::new();
static PDFIUM: OnceLock<std::result::Result<Pdfium, String>> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

/// Pages are never rendered larger than this many pixels, whatever their size in points.
pub const MAX_PIXELS: f32 = 60_000_000.0;
/// Nor longer than this on either side.
pub const MAX_SIDE: f32 = 16_000.0;

/// Where to load PDFium from. The app calls this once at startup; later calls, and calls
/// after the first PDF was opened, are ignored.
pub fn set_library(path: PathBuf) {
    let _ = LIBRARY.set(path);
}

fn candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(p) = LIBRARY.get() {
        paths.push(p.clone());
    }
    if let Some(p) = std::env::var_os("DROPWHEEL_PDFIUM") {
        paths.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        paths.push(Pdfium::pdfium_platform_library_name_at_path(dir));
    }
    paths
}

fn load() -> std::result::Result<Pdfium, String> {
    let mut tried = Vec::new();
    for path in candidates() {
        if !path.is_file() {
            continue;
        }
        match Pdfium::bind_to_library(&path) {
            Ok(bindings) => return Ok(Pdfium::new(bindings)),
            Err(e) => tried.push(format!("{}: {e}", path.display())),
        }
    }
    match Pdfium::bind_to_system_library() {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(_) if tried.is_empty() => Err("the PDFium library (pdfium.dll) was not found".into()),
        Err(_) => Err(tried.join("; ")),
    }
}

/// Runs `f` with PDFium, holding the lock for its duration.
pub fn with<T>(f: impl FnOnce(&Pdfium) -> Result<T>) -> Result<T> {
    let _guard = LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let pdfium = PDFIUM
        .get_or_init(load)
        .as_ref()
        .map_err(|reason| Error::MissingLibrary {
            what: "PDF support",
            reason: reason.clone(),
        })?;
    f(pdfium)
}

/// Whether PDFium could be loaded.
pub fn available() -> bool {
    with(|_| Ok(())).is_ok()
}

/// Turns a PDFium failure on `path` into a message for the user.
pub fn error(path: &Path, e: PdfiumError) -> Error {
    use PdfiumInternalError::{FormatError, PasswordError, SecurityError};
    match e {
        PdfiumError::PdfiumLibraryInternalError(PasswordError | SecurityError) => {
            Error::Protected {
                path: path.to_path_buf(),
            }
        }
        PdfiumError::PdfiumLibraryInternalError(FormatError) => Error::Decode {
            path: path.to_path_buf(),
            kind: FileKind::Pdf,
            reason: "the file is damaged or is not a PDF".into(),
        },
        other => Error::Decode {
            path: path.to_path_buf(),
            kind: FileKind::Pdf,
            reason: other.to_string(),
        },
    }
}

/// Opens `data`, the contents of `path`, as a PDF.
pub fn open<'a>(pdfium: &'a Pdfium, path: &Path, data: &'a [u8]) -> Result<PdfDocument<'a>> {
    let document = pdfium
        .load_pdf_from_byte_slice(data, None)
        .map_err(|e| error(path, e))?;
    if document.pages().is_empty() {
        return Err(Error::Decode {
            path: path.to_path_buf(),
            kind: FileKind::Pdf,
            reason: "the PDF has no pages".into(),
        });
    }
    Ok(document)
}

/// Reads a whole PDF into memory, which PDFium then reads from.
pub fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| Error::io(path, e))
}

/// The size of `page` as displayed (after its rotation), in points.
pub fn page_size(page: &PdfPage) -> (f32, f32) {
    (page.width().value.max(1.0), page.height().value.max(1.0))
}

/// The pixels per point that render `page` at `dpi`, within the size limits.
pub fn scale_for(page: &PdfPage, dpi: f32) -> f32 {
    let (w, h) = page_size(page);
    let wanted = dpi / 72.0;
    let by_area = (MAX_PIXELS / (w * h)).sqrt();
    let by_side = MAX_SIDE / w.max(h);
    wanted.min(by_area).min(by_side).max(0.01)
}

/// Renders `page` upright, on white, at `scale` pixels per point.
pub fn render(path: &Path, page: &PdfPage, scale: f32) -> Result<RgbImage> {
    let (w, h) = page_size(page);
    let width = ((w * scale).round() as i32).max(1);
    let height = ((h * scale).round() as i32).max(1);
    let config = PdfRenderConfig::new()
        .set_target_size(width, height)
        .render_form_data(true)
        .render_annotations(true);
    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| error(path, e))?;
    let (bw, bh) = (bitmap.width().max(0) as u32, bitmap.height().max(0) as u32);
    let rgba = bitmap.as_rgba_bytes();
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for px in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
    }
    RgbImage::from_raw(bw, bh, rgb).ok_or_else(|| Error::Decode {
        path: path.to_path_buf(),
        kind: FileKind::Pdf,
        reason: "PDFium returned an incomplete picture".into(),
    })
}

/// The text of `page` in reading order, lines separated by `\n`.
pub fn page_text(path: &Path, page: &PdfPage) -> Result<String> {
    let text = page.text().map_err(|e| error(path, e))?.all();
    Ok(normalise_text(&text))
}

/// Line breaks as `\n`, and Arabic presentation forms and Latin ligatures, which some
/// PDFs store instead of letters, turned back into letters.
fn normalise_text(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{FB00}'..='\u{FB06}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFE}' => {
                out.extend(std::iter::once(c).nfkc());
            }
            // PDFium marks soft hyphens and unknown glyphs this way.
            '\u{FEFF}' | '\u{FFFE}' | '\u{FFFF}' | '\u{0002}' => {}
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_normalised() {
        assert_eq!(normalise_text("a\r\nb\rc"), "a\nb\nc");
        assert_eq!(normalise_text("\u{FB01}ne"), "fine");
        // Isolated, final and initial forms of beh, then the lam-alef ligature.
        assert_eq!(normalise_text("\u{FE8F}\u{FE90}\u{FE91}\u{FEFB}"), "بببلا");
    }
}
