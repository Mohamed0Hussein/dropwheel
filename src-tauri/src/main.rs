// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod dialog;
mod integration;
mod jobs;
mod pdfium;
mod platform;
mod previews;
mod radial;
mod settings;
mod settings_window;
mod shelf;
mod tray;

use anyhow::Context;
use tauri::{Manager, RunEvent};

use crate::settings::SettingsStore;

fn main() {
    if let Err(err) = run() {
        log::error!("{err:#}");
        eprintln!("Dropwheel failed to start: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let app = tauri::Builder::default()
        // Must come first so a second launch is turned away before anything else starts.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A notification's buttons launch a second copy with a URL; anything else
            // is someone starting Dropwheel again.
            if args.iter().any(|a| a.starts_with(integration::SCHEME)) {
                integration::handle_args(app, &args);
            } else {
                shelf::set_visible(app, true);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for(
                    env!("CARGO_CRATE_NAME"),
                    if cfg!(debug_assertions) {
                        log::LevelFilter::Debug
                    } else {
                        log::LevelFilter::Info
                    },
                )
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::settings_info,
            commands::update_settings,
            commands::open_notices,
            commands::radial_menu,
            commands::radial_hide,
            commands::run_action,
            commands::open_dialog,
            commands::dialog_info,
            commands::image_info,
            commands::image_preview,
            commands::estimate_compress,
            commands::read_metadata,
            commands::dialog_run,
            commands::cancel_job,
        ])
        .setup(|app| {
            let path = settings::default_path(app.handle())?;
            log::info!("settings: {}", path.display());
            let store = SettingsStore::load(path);
            let start_with_windows = store.get().start_with_windows;
            app.manage(store);
            integration::register(app.handle());
            // Keeps the Run entry pointing at this exe, wherever it now is.
            if let Err(err) = integration::set_start_with_windows(app.handle(), start_with_windows)
            {
                log::warn!("could not update starting with Windows: {err:#}");
            }
            app.manage(jobs::Jobs::default());
            app.manage(dialog::Dialogs::default());
            app.manage(previews::Previews::default());
            pdfium::install(app.handle());
            tray::create(app.handle())?;
            radial::create(app.handle())?;
            shelf::create(app.handle())?;
            #[cfg(debug_assertions)]
            dev_drop(app.handle());
            // Started by Windows from a notification's button.
            let args: Vec<String> = std::env::args().skip(1).collect();
            integration::handle_args(app.handle(), &args);
            Ok(())
        })
        .build(tauri::generate_context!())
        .context("could not initialise the app")?;

    app.run(|app, event| match event {
        // Closing the last window must not quit: the app lives in the tray. Quit from the
        // tray menu passes an exit code and goes through.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        RunEvent::Exit => {
            let saved = app.try_state::<SettingsStore>().map(|s| s.save_now());
            if let Some(Err(err)) = saved {
                log::error!("could not save settings: {err:#}");
            }
        }
        _ => {}
    });
    Ok(())
}

/// Development aid: `DROPWHEEL_DEV_DROP="C:\a.png;C:\b.pdf"` opens the menu for those
/// files at startup, as if they had been dropped in the middle of the primary screen.
#[cfg(debug_assertions)]
fn dev_drop(app: &tauri::AppHandle) {
    let Some(list) = std::env::var_os("DROPWHEEL_DEV_DROP") else {
        return;
    };
    let paths: Vec<_> = std::env::split_paths(&list).collect();
    let centre = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let (p, s) = (m.position(), m.size());
            tauri::PhysicalPosition::new(
                f64::from(p.x) + f64::from(s.width) / 2.0,
                f64::from(p.y) + f64::from(s.height) / 2.0,
            )
        })
        .unwrap_or_default();
    radial::open(app, paths, centre);
}
