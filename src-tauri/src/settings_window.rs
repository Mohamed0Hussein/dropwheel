//! The Settings window, opened from the tray.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "settings";

/// Brings the window forward if it is already open.
pub fn open<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let shown = window
            .unminimize()
            .and_then(|()| window.show())
            .and_then(|()| window.set_focus());
        if let Err(err) = shown {
            log::warn!("could not bring settings forward: {err}");
        }
        return;
    }
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
        .title(format!("{} Settings", app.package_info().name))
        .inner_size(480.0, 560.0)
        .min_inner_size(400.0, 420.0)
        .center()
        .build();
    if let Err(err) = built {
        log::error!("could not open settings: {err}");
    }
}
