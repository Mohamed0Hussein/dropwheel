// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod platform;
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            shelf::set_visible(app, true);
        }))
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
        .invoke_handler(tauri::generate_handler![commands::settings_info])
        .setup(|app| {
            let path = settings::default_path(app.handle())?;
            log::info!("settings: {}", path.display());
            app.manage(SettingsStore::load(path));
            tray::create(app.handle())?;
            shelf::create(app.handle())?;
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
