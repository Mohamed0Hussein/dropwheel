//! Builds the Tauri context, and embeds PDFium in the executable when
//! `scripts/fetch-pdfium.mjs` has downloaded it into `vendor/pdfium`.

use std::path::PathBuf;

fn main() {
    embed_pdfium();
    tauri_build::build()
}

fn embed_pdfium() {
    println!("cargo::rustc-check-cfg=cfg(pdfium_embedded)");
    let Some(manifest) = std::env::var_os("CARGO_MANIFEST_DIR") else {
        return;
    };
    let dir = PathBuf::from(manifest).join("../vendor/pdfium");
    // Watching the folder picks up a later download; it has to exist to be watched.
    let _ = std::fs::create_dir_all(&dir);
    println!("cargo::rerun-if-changed={}", dir.display());

    let name = match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => "pdfium.dll",
        Ok("macos") => "libpdfium.dylib",
        _ => "libpdfium.so",
    };
    let file = dir.join(name);
    if !file.is_file() {
        println!(
            "cargo::warning=PDFium is not in vendor/pdfium, so PDF actions will be unavailable. \
             Run `node scripts/fetch-pdfium.mjs` (npm run dev and npm run build do this)."
        );
        return;
    }
    // Names the extracted copy, so a new PDFium never collides with an old one.
    let id = std::fs::read_to_string(dir.join(format!("{name}.sha256")))
        .ok()
        .and_then(|s| s.get(..16).map(str::to_string))
        .unwrap_or_else(|| {
            std::fs::metadata(&file)
                .map(|m| m.len().to_string())
                .unwrap_or_default()
        });
    println!("cargo::rustc-cfg=pdfium_embedded");
    println!("cargo::rustc-env=DROPWHEEL_PDFIUM_FILE={}", file.display());
    println!("cargo::rustc-env=DROPWHEEL_PDFIUM_NAME={name}");
    println!("cargo::rustc-env=DROPWHEEL_PDFIUM_ID={id}");
}
