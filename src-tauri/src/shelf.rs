//! The small floating square that files are dropped on.

use tauri::{
    AppHandle, DragDropEvent, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};

use crate::radial;
use crate::settings::{Position, SettingsStore};
use crate::tray;

pub const LABEL: &str = "shelf";
/// Logical pixels, so the shelf keeps its size on high-DPI displays.
const SIZE: f64 = 120.0;
/// Gap from the corner of the screen on first run, in logical pixels.
const MARGIN: f64 = 24.0;
/// Where Windows parks a minimised window.
const PARKED: i32 = -32000;

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("shelf.html".into()));
    #[cfg(windows)]
    let builder = match crate::platform::hidden_owner() {
        Some(owner) => builder.owner_raw(owner),
        None => builder,
    };
    let window = builder
        .title(format!("{} shelf", app.package_info().name))
        .inner_size(SIZE, SIZE)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        // Shown once it is in place, so it never flashes at the default position.
        .visible(false)
        .build()?;

    let saved = app.state::<SettingsStore>().get().shelf;
    let position = saved
        .position
        .filter(|p| is_on_screen(&window, *p))
        .or_else(|| default_position(&window));
    if let Some(p) = position {
        window.set_position(PhysicalPosition::new(p.x, p.y))?;
    }

    let handle = app.clone();
    let shelf = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::DragDrop(DragDropEvent::Drop { paths, position }) if !paths.is_empty() => {
            radial::open(&handle, paths.clone(), drop_point(&shelf, *position));
        }
        WindowEvent::Moved(p) => remember_position(&handle, *p),
        // Alt+F4 on the shelf hides it; only Quit in the tray menu ends the app.
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            set_visible(&handle, false);
        }
        _ => {}
    });

    if saved.visible {
        window.show()?;
    }
    tray::set_shelf_visible(app, saved.visible);
    Ok(())
}

pub fn set_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let result = if visible {
        // Re-assert the z-order: other always-on-top windows may have covered it.
        window.show().and_then(|()| window.set_always_on_top(true))
    } else {
        window.hide()
    };
    if let Err(err) = result {
        log::error!("could not change shelf visibility: {err}");
        return;
    }
    if let Some(store) = app.try_state::<SettingsStore>() {
        store.update(|s| s.shelf.visible = visible);
    }
    tray::set_shelf_visible(app, visible);
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let visible = app
        .get_webview_window(LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    set_visible(app, !visible);
}

/// Where the files were dropped, in screen coordinates. The cursor is still there; the
/// event's own position is relative to the window and only used as a fallback.
fn drop_point<R: Runtime>(
    window: &WebviewWindow<R>,
    in_window: PhysicalPosition<f64>,
) -> PhysicalPosition<f64> {
    window.cursor_position().unwrap_or_else(|_| {
        let origin = window.inner_position().unwrap_or_default();
        PhysicalPosition::new(
            f64::from(origin.x) + in_window.x,
            f64::from(origin.y) + in_window.y,
        )
    })
}

fn remember_position<R: Runtime>(app: &AppHandle<R>, p: PhysicalPosition<i32>) {
    if p.x <= PARKED || p.y <= PARKED {
        return;
    }
    if let Some(store) = app.try_state::<SettingsStore>() {
        store.update(|s| s.shelf.position = Some(Position { x: p.x, y: p.y }));
    }
}

/// A saved position can point at a monitor that has since been unplugged.
fn is_on_screen<R: Runtime>(window: &WebviewWindow<R>, p: Position) -> bool {
    let screens: Vec<Screen> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| Screen {
            origin: Position {
                x: m.position().x,
                y: m.position().y,
            },
            width: m.size().width,
            height: m.size().height,
            scale_factor: m.scale_factor(),
        })
        .collect();
    centre_is_on_a_screen(p, &screens)
}

struct Screen {
    origin: Position,
    width: u32,
    height: u32,
    scale_factor: f64,
}

fn centre_is_on_a_screen(p: Position, screens: &[Screen]) -> bool {
    screens.iter().any(|s| {
        let half = physical(SIZE / 2.0, s.scale_factor);
        let (x, y) = (p.x.saturating_add(half), p.y.saturating_add(half));
        let right = s.origin.x.saturating_add_unsigned(s.width);
        let bottom = s.origin.y.saturating_add_unsigned(s.height);
        (s.origin.x..right).contains(&x) && (s.origin.y..bottom).contains(&y)
    })
}

/// Bottom-right corner of the primary display, clear of the taskbar.
fn default_position<R: Runtime>(window: &WebviewWindow<R>) -> Option<Position> {
    let monitor = window.primary_monitor().ok().flatten()?;
    let area = monitor.work_area();
    // Not outer_size(): a window that has never been shown may report zero.
    let inset = physical(SIZE + MARGIN, monitor.scale_factor());
    Some(Position {
        x: area.position.x + area.size.width as i32 - inset,
        y: area.position.y + area.size.height as i32 - inset,
    })
}

fn physical(logical: f64, scale_factor: f64) -> i32 {
    (logical * scale_factor).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn screen(x: i32, y: i32, width: u32, height: u32, scale_factor: f64) -> Screen {
        Screen {
            origin: Position { x, y },
            width,
            height,
            scale_factor,
        }
    }

    #[test]
    fn saved_position_must_be_on_a_connected_screen() {
        // A 4K laptop panel at 150% with a 1080p monitor to its left.
        let screens = [
            screen(0, 0, 3840, 2160, 1.5),
            screen(-1920, 0, 1920, 1080, 1.0),
        ];
        let at = |x, y| centre_is_on_a_screen(Position { x, y }, &screens);
        assert!(at(3600, 1900));
        assert!(at(-1900, 10));
        // Mostly off the right edge, or on a monitor that is no longer there.
        assert!(!at(3800, 100));
        assert!(!at(-3000, 100));
        assert!(!at(100, 2500));
        assert!(!at(i32::MAX, i32::MAX));
    }
}
