//! IPC commands the windows call. Each one is a thin wrapper; the work lives in
//! `dropwheel-core` or the other modules.

use dropwheel_core::actions;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::jobs;
use crate::radial::{self, Menu, RadialState};
use crate::settings::{Settings, SettingsStore};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInfo {
    path: String,
    settings: Settings,
}

#[tauri::command]
pub fn settings_info(store: State<'_, SettingsStore>) -> SettingsInfo {
    SettingsInfo {
        path: store.path().display().to_string(),
        settings: store.get(),
    }
}

/// The menu the radial window should show, for when it loads after the drop.
#[tauri::command]
pub fn radial_menu(state: State<'_, RadialState>) -> Option<Menu> {
    state.current()
}

/// Called by the radial window once its close animation has finished.
#[tauri::command]
pub fn radial_hide(app: AppHandle) {
    radial::hide(&app);
}

/// Runs `action_id` on the files of drop `drop_id` and returns the job id. The page only
/// names the drop; the paths come from what was actually dropped.
#[tauri::command]
pub fn run_action(
    app: AppHandle,
    state: State<'_, RadialState>,
    drop_id: u64,
    action_id: String,
) -> Result<u64, String> {
    let files = state
        .files(drop_id)
        .ok_or("This menu is out of date. Drop the files again.")?;
    let action = actions::find(&action_id).ok_or_else(|| format!("Unknown action {action_id}."))?;
    if !files.iter().all(|f| action.applicable(f.kind)) {
        return Err(format!("{} does not apply to these files.", action.label()));
    }
    Ok(jobs::start(
        &app,
        action,
        files.into_iter().map(|f| f.path).collect(),
    ))
}
