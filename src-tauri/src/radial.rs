//! The radial menu: a transparent window that opens centred on the drop point.
//!
//! The window is created hidden at startup and reused, so opening it costs a move and a
//! show rather than starting a new WebView.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use dropwheel_core::{Action, ActionGroup, FileKind, actions};
use serde::Serialize;
use tauri::utils::config::BackgroundThrottlingPolicy;
use tauri::{
    AppHandle, DragDropEvent, Emitter, Manager, PhysicalPosition, Runtime, WebviewUrl,
    WebviewWindowBuilder, WindowEvent,
};

pub const LABEL: &str = "radial";
/// Logical pixels. The UI draws its rings to fit.
const SIZE: f64 = 380.0;
/// Sent to the radial window with the menu to show.
pub const OPEN_EVENT: &str = "radial:open";

/// What the radial window shows. Kept so the page can ask for it again, and so the
/// chosen action runs on the files that were dropped rather than on paths from the page.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Menu {
    pub drop_id: u64,
    pub files: Vec<DroppedFile>,
    pub actions: Vec<MenuAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DroppedFile {
    /// Stays on this side: serde cannot write a path that is not valid Unicode, which
    /// Windows allows, and the page has no use for it beyond the preview.
    #[serde(skip)]
    pub path: PathBuf,
    /// Set when the WebView can show the file itself and the asset protocol allows it.
    pub preview_path: Option<String>,
    pub name: String,
    pub kind: FileKind,
    pub kind_label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuAction {
    pub id: &'static str,
    pub label: &'static str,
    pub group: ActionGroup,
    /// Choosing it opens a settings dialog instead of running straight away.
    pub dialog: bool,
}

impl From<&'static dyn Action> for MenuAction {
    fn from(a: &'static dyn Action) -> Self {
        MenuAction {
            id: a.id(),
            label: a.label(),
            group: a.group(),
            dialog: crate::dialog::has_dialog(a.id()),
        }
    }
}

#[derive(Default)]
pub struct RadialState {
    menu: Mutex<Option<Menu>>,
    next_drop: AtomicU64,
}

impl RadialState {
    pub fn current(&self) -> Option<Menu> {
        self.menu.lock().ok().and_then(|m| m.clone())
    }

    /// The files of drop `drop_id`, if it is still the current one.
    pub fn files(&self, drop_id: u64) -> Option<Vec<DroppedFile>> {
        self.current()
            .filter(|m| m.drop_id == drop_id)
            .map(|m| m.files)
    }

    fn set(&self, menu: Menu) {
        if let Ok(mut current) = self.menu.lock() {
            *current = Some(menu);
        }
    }
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    app.manage(RadialState::default());
    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("radial.html".into()));
    #[cfg(windows)]
    let builder = match crate::platform::hidden_owner() {
        Some(owner) => builder.owner_raw(owner),
        None => builder,
    };
    let window = builder
        .title(format!("{} menu", app.package_info().name))
        .inner_size(SIZE, SIZE)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        // Created visible and focusable, then hidden at once: on Windows a webview created
        // hidden and unfocused never accepts file drops (tauri-apps/tauri#14643). The page
        // is transparent at this point, so nothing shows.
        .visible(true)
        // Keep the hidden page responsive so the open animation starts on the first frame.
        .background_throttling(BackgroundThrottlingPolicy::Disabled)
        .build()?;
    window.hide()?;

    let handle = app.clone();
    let menu_window = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(&handle);
        }
        // The open menu covers the shelf, so a second drop lands here. Treat it the same.
        WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) if !paths.is_empty() => {
            if let Ok(point) = menu_window.cursor_position() {
                open(&handle, paths.clone(), point);
            }
        }
        _ => {}
    });
    Ok(())
}

/// Opens the menu for `paths`, centred on `point` (physical screen coordinates).
pub fn open<R: Runtime>(app: &AppHandle<R>, paths: Vec<PathBuf>, point: PhysicalPosition<f64>) {
    let app = app.clone();
    // Reading file headers can be slow on network drives; keep it off the main thread.
    tauri::async_runtime::spawn_blocking(move || {
        let Some(state) = app.try_state::<RadialState>() else {
            return;
        };
        let drop_id = state.next_drop.fetch_add(1, Ordering::Relaxed) + 1;
        let files: Vec<DroppedFile> = paths.into_iter().map(|p| describe(&app, p)).collect();
        let kinds: Vec<FileKind> = files.iter().map(|f| f.kind).collect();
        let actions = actions::for_kinds(&kinds)
            .into_iter()
            .map(MenuAction::from)
            .collect();
        let menu = Menu {
            drop_id,
            files,
            actions,
        };
        log::debug!(
            "drop {drop_id}: {} file(s), {} action(s)",
            menu.files.len(),
            menu.actions.len()
        );
        state.set(menu.clone());
        if let Err(err) = show(&app, &menu, point) {
            log::error!("could not open the menu: {err}");
        }
    });
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL)
        && let Err(err) = window.hide()
    {
        log::error!("could not hide the menu: {err}");
    }
}

fn show<R: Runtime>(
    app: &AppHandle<R>,
    menu: &Menu,
    point: PhysicalPosition<f64>,
) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    let monitor = window
        .monitor_from_point(point.x, point.y)?
        .or(window.primary_monitor()?);
    let area = monitor.as_ref().map(|m| {
        let a = m.work_area();
        Area {
            x: a.position.x,
            y: a.position.y,
            width: a.size.width,
            height: a.size.height,
        }
    });
    let scale = monitor.as_ref().map_or(1.0, |m| m.scale_factor());
    let size = (SIZE * scale).round() as i32;
    let at = place(point, size, area);
    window.set_position(at)?;
    // Moving to a display with a different scale resizes the window; centre again on the
    // size it ended up with.
    let actual = window.outer_size()?.width as i32;
    if actual > 0 && actual != size {
        window.set_position(place(point, actual, area))?;
    }

    window.emit_to(LABEL, OPEN_EVENT, menu)?;
    window.show()?;
    window.set_always_on_top(true)?;
    window.set_focus()?;
    Ok(())
}

fn describe<R: Runtime>(app: &AppHandle<R>, path: PathBuf) -> DroppedFile {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dropwheel_core::paths::display(&path));
    let kind = if path.is_dir() {
        FileKind::Unknown
    } else {
        FileKind::detect(&path).unwrap_or_else(|err| {
            log::warn!("could not read {}: {err}", path.display());
            FileKind::Unknown
        })
    };
    let preview_path = path
        .to_str()
        .filter(|_| browser_can_show(kind))
        .filter(|_| {
            app.asset_protocol_scope()
                .allow_file(&path)
                .map_err(|err| log::warn!("no preview for {}: {err}", path.display()))
                .is_ok()
        })
        .map(str::to_string);
    DroppedFile {
        path,
        preview_path,
        name,
        kind,
        kind_label: kind.label(),
    }
}

/// Formats WebView2 decodes itself. TIFF and HEIC are not among them.
fn browser_can_show(kind: FileKind) -> bool {
    matches!(
        kind,
        FileKind::Jpeg
            | FileKind::Png
            | FileKind::Webp
            | FileKind::Bmp
            | FileKind::Gif
            | FileKind::Ico
            | FileKind::Avif
    )
}

#[derive(Debug, Clone, Copy)]
struct Area {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Top-left corner for a `size`-square window centred on `point`, moved as little as
/// needed to stay inside `area`.
fn place(point: PhysicalPosition<f64>, size: i32, area: Option<Area>) -> PhysicalPosition<i32> {
    let half = f64::from(size) / 2.0;
    let x = (point.x - half).round() as i32;
    let y = (point.y - half).round() as i32;
    let Some(a) = area else {
        return PhysicalPosition::new(x, y);
    };
    let clamp = |v: i32, start: i32, extent: u32| {
        let end = start.saturating_add_unsigned(extent).saturating_sub(size);
        v.min(end).max(start)
    };
    PhysicalPosition::new(clamp(x, a.x, a.width), clamp(y, a.y, a.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Option<Area> = Some(Area {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    });

    fn at(x: f64, y: f64, size: i32, area: Option<Area>) -> (i32, i32) {
        let p = place(PhysicalPosition::new(x, y), size, area);
        (p.x, p.y)
    }

    #[test]
    fn centres_on_the_drop_point() {
        assert_eq!(at(960.0, 520.0, 380, AREA), (770, 330));
        assert_eq!(at(960.0, 520.0, 380, None), (770, 330));
    }

    #[test]
    fn stays_on_screen_near_edges() {
        // The shelf's default spot is the bottom-right corner.
        assert_eq!(at(1850.0, 1000.0, 380, AREA), (1540, 660));
        assert_eq!(at(10.0, 10.0, 380, AREA), (0, 0));
    }

    #[test]
    fn works_on_monitors_left_of_the_primary() {
        let left = Some(Area {
            x: -2560,
            y: 0,
            width: 2560,
            height: 1400,
        });
        assert_eq!(at(-2540.0, 700.0, 570, left), (-2560, 415));
    }

    #[test]
    fn small_areas_pin_to_the_top_left() {
        let tiny = Some(Area {
            x: 100,
            y: 100,
            width: 200,
            height: 200,
        });
        assert_eq!(at(200.0, 200.0, 380, tiny), (100, 100));
    }

    #[test]
    fn previews_only_for_formats_the_webview_decodes() {
        assert!(browser_can_show(FileKind::Avif));
        assert!(!browser_can_show(FileKind::Tiff));
        assert!(!browser_can_show(FileKind::Heic));
        assert!(!browser_can_show(FileKind::Pdf));
    }
}
