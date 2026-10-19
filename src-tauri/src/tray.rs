//! The notification-area icon and its menu.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::{settings_window, shelf};

const TOGGLE_SHELF: &str = "toggle-shelf";
const SETTINGS: &str = "settings";
const QUIT: &str = "quit";

/// Kept so the Show/Hide item can follow the shelf's state.
struct TrayMenu<R: Runtime> {
    toggle_shelf: MenuItem<R>,
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let toggle_shelf = MenuItem::with_id(app, TOGGLE_SHELF, "Hide shelf", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &toggle_shelf,
            &MenuItem::with_id(app, SETTINGS, "Settings…", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(tauri::include_image!("icons/tray.png"))
        .tooltip(&app.package_info().name)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TOGGLE_SHELF => shelf::toggle(app),
            SETTINGS => settings_window::open(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                shelf::toggle(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayMenu { toggle_shelf });
    Ok(())
}

pub fn set_shelf_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    let Some(menu) = app.try_state::<TrayMenu<R>>() else {
        return;
    };
    let text = if visible { "Hide shelf" } else { "Show shelf" };
    if let Err(err) = menu.toggle_shelf.set_text(text) {
        log::warn!("could not update the tray menu: {err}");
    }
}
