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
    let saved = app.state::<SettingsStore>().get().shelf;
    let screens = screens(app);
    let placed = saved
        .position
        .and_then(|p| screen_under(p, &screens).map(|s| (keep_inside(p, s), s.scale_factor)))
        .or_else(|| default_position(app));

    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("shelf.html".into()));
    #[cfg(windows)]
    let builder = match crate::platform::hidden_owner() {
        Some(owner) => builder.owner_raw(owner),
        None => builder,
    };
    let builder = builder
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
        // Created visible and focusable, already in place. On Windows a webview created
        // hidden and unfocused never accepts file drops, even once shown: the cursor shows
        // "not allowed" (tauri-apps/tauri#14643).
        .visible(true);
    // The builder takes logical pixels, converted with the scale of the display they land
    // on.
    let builder = match placed {
        Some((p, scale)) => builder.position(f64::from(p.x) / scale, f64::from(p.y) / scale),
        None => builder,
    };
    let window = builder.build()?;
    if let Some((p, _)) = placed {
        // Exact placement, in case the conversion settled on a neighbouring display.
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

    if !saved.visible {
        window.hide()?;
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

struct Screen {
    origin: Position,
    width: u32,
    height: u32,
    scale_factor: f64,
}

fn screens<R: Runtime>(app: &AppHandle<R>) -> Vec<Screen> {
    app.available_monitors()
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
        .collect()
}

/// The display under the shelf's centre at `p`. None when a saved position points at a
/// monitor that has since been unplugged.
fn screen_under(p: Position, screens: &[Screen]) -> Option<&Screen> {
    screens.iter().find(|s| {
        let half = physical(SIZE / 2.0, s.scale_factor);
        let (x, y) = (p.x.saturating_add(half), p.y.saturating_add(half));
        let right = s.origin.x.saturating_add_unsigned(s.width);
        let bottom = s.origin.y.saturating_add_unsigned(s.height);
        (s.origin.x..right).contains(&x) && (s.origin.y..bottom).contains(&y)
    })
}

/// Moves `p` as little as needed for the whole shelf to be on `screen`.
fn keep_inside(p: Position, screen: &Screen) -> Position {
    let size = physical(SIZE, screen.scale_factor);
    let clamp = |v: i32, start: i32, extent: u32| {
        let end = start.saturating_add_unsigned(extent).saturating_sub(size);
        v.min(end).max(start)
    };
    Position {
        x: clamp(p.x, screen.origin.x, screen.width),
        y: clamp(p.y, screen.origin.y, screen.height),
    }
}

/// Bottom-right corner of the primary display, clear of the taskbar, with that display's
/// scale factor.
fn default_position<R: Runtime>(app: &AppHandle<R>) -> Option<(Position, f64)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let area = monitor.work_area();
    let inset = physical(SIZE + MARGIN, monitor.scale_factor());
    let p = Position {
        x: area.position.x + area.size.width as i32 - inset,
        y: area.position.y + area.size.height as i32 - inset,
    };
    Some((p, monitor.scale_factor()))
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
        let at = |x, y| screen_under(Position { x, y }, &screens).map(|s| s.scale_factor);
        assert_eq!(at(3600, 1900), Some(1.5));
        assert_eq!(at(-1900, 10), Some(1.0));
        // Mostly off the right edge, or on a monitor that is no longer there.
        assert_eq!(at(3800, 100), None);
        assert_eq!(at(-3000, 100), None);
        assert_eq!(at(100, 2500), None);
        assert_eq!(at(i32::MAX, i32::MAX), None);
    }

    #[test]
    fn restored_shelf_is_pulled_fully_on_screen() {
        let laptop = screen(0, 0, 3840, 2160, 1.5);
        let inside = |x, y| {
            let p = keep_inside(Position { x, y }, &laptop);
            (p.x, p.y)
        };
        // 120 logical pixels are 180 physical at 150%.
        assert_eq!(inside(3700, 2100), (3660, 1980));
        assert_eq!(inside(-50, -10), (0, 0));
        assert_eq!(inside(500, 600), (500, 600));
    }
}
