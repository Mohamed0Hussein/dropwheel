//! PDF support. PDFium renders pages and reads their text, `krilla` writes new PDFs and
//! `lopdf` changes existing ones.

pub mod engine;
pub mod extract;
pub mod layout;
pub mod read;
pub mod write;

use std::path::Path;

use image::RgbImage;

use crate::action::Options;
use crate::error::{Error, Result};
use crate::kind::FileKind;

/// The pages of a PDF, as displayed: width and height in points.
pub fn page_sizes(path: &Path) -> Result<Vec<(f32, f32)>> {
    let data = engine::read(path)?;
    engine::with(|pdfium| {
        let document = engine::open(pdfium, path, &data)?;
        Ok(document
            .pages()
            .iter()
            .map(|page| engine::page_size(&page))
            .collect())
    })
}

/// Renders page `index` of `path` (from 0) to fit within `max_side` pixels.
pub fn render_page(path: &Path, index: u32, max_side: u32) -> Result<RgbImage> {
    let data = engine::read(path)?;
    engine::with(|pdfium| {
        let document = engine::open(pdfium, path, &data)?;
        let page = document
            .pages()
            .get(i32::try_from(index).unwrap_or(i32::MAX))
            .map_err(|_| Error::InvalidOptions(format!("the PDF has no page {}", index + 1)))?;
        let (w, h) = engine::page_size(&page);
        let scale = (max_side as f32 / w.max(h)).min(engine::scale_for(&page, 600.0));
        engine::render(path, &page, scale)
    })
}

/// Renders every page of `path` at `dpi`, calling `each` with the page index, the page
/// count and the picture. Progress is reported between pages and cancellation checked.
pub fn render_pages(
    path: &Path,
    dpi: f32,
    opts: &Options,
    mut each: impl FnMut(usize, usize, RgbImage) -> Result<()>,
) -> Result<()> {
    let data = engine::read(path)?;
    engine::with(|pdfium| {
        let document = engine::open(pdfium, path, &data)?;
        let count = usize::try_from(document.pages().len()).unwrap_or(0);
        for (i, page) in document.pages().iter().enumerate() {
            opts.check_cancelled()?;
            let image = engine::render(path, &page, engine::scale_for(&page, dpi))?;
            each(i, count, image)?;
            opts.report(0.1 + 0.85 * (i + 1) as f32 / count.max(1) as f32);
        }
        Ok(())
    })
}

/// The text of each page.
pub fn page_texts(path: &Path, opts: &Options) -> Result<Vec<String>> {
    let data = engine::read(path)?;
    engine::with(|pdfium| {
        let document = engine::open(pdfium, path, &data)?;
        let mut texts = Vec::new();
        for page in document.pages().iter() {
            opts.check_cancelled()?;
            texts.push(engine::page_text(path, &page)?);
        }
        Ok(texts)
    })
}

/// Checks that `path` is a PDF this build can open.
pub fn require_pdf(path: &Path) -> Result<()> {
    match FileKind::detect(path)? {
        FileKind::Pdf => Ok(()),
        kind => Err(Error::Unsupported {
            action: "This PDF action",
            kind,
        }),
    }
}

/// Points the engine at the library the fetch script downloads, for tests.
#[cfg(test)]
pub(crate) fn test_library() -> bool {
    let name = if cfg!(windows) {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(name);
    engine::set_library(path);
    let ok = engine::available();
    if !ok && std::env::var_os("DROPWHEEL_REQUIRE_PDFIUM").is_some() {
        panic!("PDFium is required but missing; run scripts/fetch-pdfium.mjs");
    }
    ok
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
    fn pages_render_upright() {
        if !test_library() {
            return;
        }
        let sizes = page_sizes(&fixture("text.pdf")).unwrap();
        assert_eq!(sizes.len(), 2);
        assert!((sizes[0].0 - 595.3).abs() < 1.0, "{sizes:?}");

        // A 200x300 page turned a quarter: landscape, red square top left.
        let sizes = page_sizes(&fixture("rotated.pdf")).unwrap();
        assert_eq!(sizes, [(300.0, 200.0)]);
        let img = render_page(&fixture("rotated.pdf"), 0, 300).unwrap();
        assert_eq!((img.width(), img.height()), (300, 200));
        assert_eq!(img.get_pixel(10, 10).0, [255, 0, 0]);
        assert_eq!(img.get_pixel(290, 190).0, [255, 255, 255]);
    }

    #[test]
    fn text_comes_out_in_reading_order() {
        if !test_library() {
            return;
        }
        let texts = page_texts(&fixture("text.pdf"), &Options::default()).unwrap();
        assert!(texts[0].contains("Quarterly report"), "{texts:?}");
        assert!(texts[0].contains("Dropwheel turns this page into text."));
        assert!(texts[1].contains("Page two"));
        let arabic = page_texts(&fixture("arabic.pdf"), &Options::default()).unwrap();
        assert!(arabic[0].contains("مرحبا بالعالم"), "{arabic:?}");
        assert!(arabic[0].contains("Hello world"));
    }

    #[test]
    fn protected_and_broken_files_are_explained() {
        if !test_library() {
            return;
        }
        let err = page_sizes(&fixture("protected.pdf")).unwrap_err();
        assert_eq!(err.code(), "protected");
        let err = page_sizes(&fixture("broken.pdf")).unwrap_err();
        assert_eq!(err.code(), "decode", "{err}");
    }
}
