//! The Settings window, opened from the tray.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "settings";
/// Tall enough for the whole page; on a short screen it scrolls instead.
const SIZE: (f64, f64) = (480.0, 700.0);

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
    // Leaves room for the title bar and the taskbar.
    let height = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| f64::from(m.size().height) / m.scale_factor() - 120.0)
        .map_or(SIZE.1, |room| SIZE.1.min(room.max(420.0)));
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
        .title(format!("{} Settings", app.package_info().name))
        .inner_size(SIZE.0, height)
        .min_inner_size(400.0, 420.0)
        .center()
        .build();
    if let Err(err) = built {
        log::error!("could not open settings: {err}");
    }
}
