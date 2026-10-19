//! Shared by the integration tests.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

/// Copies a fixture into a fresh temporary folder, optionally under another name, so
/// actions write their output there and never into the repository.
pub fn staged(name: &str, as_name: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join(as_name.unwrap_or(name));
    std::fs::copy(fixture(name), &target).unwrap();
    (dir, target)
}

pub fn decode(path: &Path) -> image::DynamicImage {
    image::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Mean colour of a 6x6 block, which is robust to lossy encoding.
pub fn colour_at(img: &image::DynamicImage, x: u32, y: u32) -> [u8; 3] {
    let rgb = img.to_rgb8();
    let mut sum = [0u32; 3];
    let mut n = 0;
    for dy in 0..6 {
        for dx in 0..6 {
            let p = rgb.get_pixel(
                (x + dx).min(rgb.width() - 1),
                (y + dy).min(rgb.height() - 1),
            );
            for (total, channel) in sum.iter_mut().zip(p.0) {
                *total += u32::from(channel);
            }
            n += 1;
        }
    }
    sum.map(|s| (s / n) as u8)
}

/// Which fixture quadrant colour `c` is closest to: red, green, blue or white.
pub fn quadrant(c: [u8; 3]) -> &'static str {
    let named = [
        ("red", [220, 30, 30]),
        ("green", [30, 200, 60]),
        ("blue", [40, 60, 220]),
        ("white", [250, 250, 250]),
    ];
    named
        .iter()
        .min_by_key(|(_, q)| (0..3).map(|i| (i32::from(c[i]) - q[i]).pow(2)).sum::<i32>())
        .map(|(name, _)| *name)
        .unwrap()
}

/// Points the PDF engine at the library `scripts/fetch-pdfium.mjs` downloads. Tests that
/// need it return early without it, unless `DROPWHEEL_REQUIRE_PDFIUM` is set, as in CI.
pub fn pdfium() -> bool {
    let name = if cfg!(windows) {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(name);
    dropwheel_core::pdf::engine::set_library(path);
    let ok = dropwheel_core::pdf::engine::available();
    if !ok && std::env::var_os("DROPWHEEL_REQUIRE_PDFIUM").is_some() {
        panic!("PDFium is required but missing; run scripts/fetch-pdfium.mjs");
    }
    ok
}

/// The text of a PDF, as Dropwheel reads it.
pub fn pdf_text(path: &Path) -> String {
    dropwheel_core::pdf::read::text(path, &dropwheel_core::Options::default()).unwrap()
}
