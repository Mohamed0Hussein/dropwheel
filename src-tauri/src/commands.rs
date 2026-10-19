//! IPC commands the windows call. Each one is a thin wrapper; the work lives in
//! `dropwheel-core` or the other modules.

use serde::Serialize;
use tauri::State;

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
