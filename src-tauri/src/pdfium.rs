//! PDFium ships inside the executable. The first time Dropwheel starts, it writes the
//! library to its local data folder and points the PDF engine at it, so the portable
//! exe needs nothing beside it.

use tauri::AppHandle;

#[cfg(pdfium_embedded)]
static LIBRARY: &[u8] = include_bytes!(env!("DROPWHEEL_PDFIUM_FILE"));

pub fn install(app: &AppHandle) {
    #[cfg(pdfium_embedded)]
    match extract(app) {
        Ok(path) => {
            log::info!("PDFium: {}", path.display());
            dropwheel_core::pdf::engine::set_library(path);
        }
        Err(err) => log::error!("could not install PDFium: {err:#}"),
    }
    #[cfg(not(pdfium_embedded))]
    {
        let _ = app;
        log::warn!("this build has no PDFium; PDF actions are unavailable");
    }
}

#[cfg(pdfium_embedded)]
fn extract(app: &AppHandle) -> anyhow::Result<std::path::PathBuf> {
    use std::io::Write;

    use anyhow::Context;
    use tauri::Manager;

    let dir = app
        .path()
        .app_local_data_dir()?
        .join("pdfium")
        .join(env!("DROPWHEEL_PDFIUM_ID"));
    let path = dir.join(env!("DROPWHEEL_PDFIUM_NAME"));
    // An existing copy is used only if it is exactly ours.
    if std::fs::read(&path).is_ok_and(|bytes| bytes == LIBRARY) {
        return Ok(path);
    }
    std::fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(&dir)?;
    tmp.write_all(LIBRARY)?;
    tmp.flush()?;
    tmp.persist(&path)
        .with_context(|| path.display().to_string())?;
    Ok(path)
}
