//! IPC commands the windows call. Each one is a thin wrapper; the work lives in
//! `dropwheel-core` or the other modules.
//!
//! Pages never send file paths. The radial menu names a drop and a dialog is tied to its
//! window, and the paths come from what was actually dropped.

use std::sync::Arc;

use dropwheel_core::actions::{self, compress, metadata};
use dropwheel_core::{Compress, FileKind, Params, pdf};
use serde::Serialize;
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State, WebviewWindow};

use crate::dialog::{self, DialogContext, Dialogs};
use crate::jobs::{self, Jobs};
use crate::previews::{Preview, Previews};
use crate::radial::{self, DroppedFile, Menu, MenuAction, RadialState};
use crate::settings::{Settings, SettingsStore};

type CommandResult<T> = Result<T, String>;

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

fn drop_and_action(
    state: &RadialState,
    drop_id: u64,
    action_id: &str,
) -> CommandResult<(Vec<DroppedFile>, &'static dyn dropwheel_core::Action)> {
    let files = state
        .files(drop_id)
        .ok_or("This menu is out of date. Drop the files again.")?;
    let action = actions::find(action_id).ok_or_else(|| format!("Unknown action {action_id}."))?;
    if !files.iter().all(|f| action.applicable(f.kind)) {
        return Err(format!("{} does not apply to these files.", action.label()));
    }
    Ok((files, action))
}

/// Runs `action_id` on the files of drop `drop_id` and returns the job id.
#[tauri::command]
pub fn run_action(
    app: AppHandle,
    state: State<'_, RadialState>,
    drop_id: u64,
    action_id: String,
    params: Option<Params>,
) -> CommandResult<u64> {
    let (files, action) = drop_and_action(&state, drop_id, &action_id)?;
    let paths = files.into_iter().map(|f| f.path).collect();
    Ok(jobs::start(&app, action, paths, params.unwrap_or_default()))
}

/// Opens the settings dialog for an action. Async, because creating a window from a
/// synchronous command can deadlock on Windows.
#[tauri::command]
pub async fn open_dialog(
    app: AppHandle,
    state: State<'_, RadialState>,
    drop_id: u64,
    action_id: String,
) -> CommandResult<()> {
    let (files, action) = drop_and_action(&state, drop_id, &action_id)?;
    dialog::open(&app, action, files).map_err(|e| e.to_string())
}

fn context(window: &WebviewWindow, dialogs: &Dialogs) -> CommandResult<DialogContext> {
    dialogs
        .context(window.label())
        .ok_or_else(|| "This window is not a dialog.".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogInfo {
    action: MenuAction,
    title: String,
    files: Vec<DroppedFile>,
}

#[tauri::command]
pub fn dialog_info(
    window: WebviewWindow,
    dialogs: State<'_, Dialogs>,
) -> CommandResult<DialogInfo> {
    let ctx = context(&window, &dialogs)?;
    Ok(DialogInfo {
        action: MenuAction::from(ctx.action),
        title: dialog::title(ctx.action),
        files: ctx.files,
    })
}

/// Decodes (or reuses) the preview of page `page` of file `index` of this dialog, off
/// the main thread.
async fn preview(window: &WebviewWindow, index: usize, page: u32) -> CommandResult<Arc<Preview>> {
    let ctx = context(window, &window.state::<Dialogs>())?;
    let file = ctx.files.get(index).ok_or("No such file.")?;
    let path = file.path.clone();
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || app.state::<Previews>().get(&path, page))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    kind: FileKind,
    kind_label: &'static str,
    /// The full image, upright; for a PDF, the page in points.
    width: u32,
    height: u32,
    pages: u32,
    /// "px" for images, "pt" for PDF pages.
    unit: &'static str,
    bytes: u64,
    preview_width: u32,
    preview_height: u32,
    preview_mime: &'static str,
}

#[tauri::command]
pub async fn image_info(
    window: WebviewWindow,
    index: usize,
    page: Option<u32>,
) -> CommandResult<ImageInfo> {
    let p = preview(&window, index, page.unwrap_or(0)).await?;
    Ok(ImageInfo {
        kind: p.kind,
        kind_label: p.kind.label(),
        width: p.width,
        height: p.height,
        pages: p.pages,
        unit: if p.kind == FileKind::Pdf { "pt" } else { "px" },
        bytes: p.file_bytes,
        preview_width: p.image.width(),
        preview_height: p.image.height(),
        preview_mime: p.mime,
    })
}

/// The preview's encoded bytes, sent as binary rather than JSON.
#[tauri::command]
pub async fn image_preview(
    window: WebviewWindow,
    index: usize,
    page: Option<u32>,
) -> CommandResult<Response> {
    let p = preview(&window, index, page.unwrap_or(0)).await?;
    Ok(Response::new(p.encoded.clone()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateInfo {
    original: u64,
    estimate: compress::Estimate,
}

/// Predicts the compressed size of the dialog's first file.
#[tauri::command]
pub async fn estimate_compress(
    window: WebviewWindow,
    target: Compress,
) -> CommandResult<EstimateInfo> {
    let ctx = context(&window, &window.state::<Dialogs>())?;
    let first = ctx.files.first().ok_or("No such file.")?;
    if first.kind == FileKind::Pdf {
        let path = first.path.clone();
        return tauri::async_runtime::spawn_blocking(move || {
            let original = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            pdf::edit::estimate(&path, target).map(|bytes| EstimateInfo {
                original,
                estimate: compress::Estimate {
                    bytes,
                    percent: match target {
                        Compress::Quality { percent } => percent,
                        Compress::Size { .. } => 0,
                    },
                    scale: 1.0,
                },
            })
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string());
    }
    let p = preview(&window, 0, 0).await?;
    let estimate = tauri::async_runtime::spawn_blocking(move || {
        compress::estimate(&p.image, p.width, p.height, p.kind, target).map(|estimate| {
            EstimateInfo {
                original: p.file_bytes,
                estimate,
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    estimate.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_metadata(
    window: WebviewWindow,
    index: usize,
) -> CommandResult<Vec<metadata::Field>> {
    let ctx = context(&window, &window.state::<Dialogs>())?;
    let path = ctx.files.get(index).ok_or("No such file.")?.path.clone();
    tauri::async_runtime::spawn_blocking(move || metadata::read(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Runs the dialog's action on all of its files with the chosen settings.
#[tauri::command]
pub fn dialog_run(
    app: AppHandle,
    window: WebviewWindow,
    dialogs: State<'_, Dialogs>,
    params: Params,
) -> CommandResult<u64> {
    let ctx = context(&window, &dialogs)?;
    let paths = ctx.files.into_iter().map(|f| f.path).collect();
    Ok(jobs::start(&app, ctx.action, paths, params))
}

#[tauri::command]
pub fn cancel_job(jobs: State<'_, Jobs>, job_id: u64) -> bool {
    jobs.cancel(job_id)
}
