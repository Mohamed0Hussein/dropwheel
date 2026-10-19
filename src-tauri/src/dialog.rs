//! Windows for the actions that need settings: compress, crop, edit, annotate, redact,
//! metadata and add background. Each one remembers the files it was opened for.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use dropwheel_core::{Action, ActionGroup};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::radial::DroppedFile;

/// Whether choosing this action opens a dialog rather than running straight away.
pub fn has_dialog(action_id: &str) -> bool {
    matches!(
        action_id,
        "compress" | "crop" | "edit" | "annotate" | "redact" | "metadata" | "add_background"
    )
}

/// "Convert to PNG", "Add background".
pub fn title(action: &dyn Action) -> String {
    match action.group() {
        ActionGroup::Convert => format!("Convert to {}", action.label()),
        ActionGroup::Tools => {
            let lower = action.label().to_lowercase();
            let mut chars = lower.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        }
    }
}

#[derive(Clone)]
pub struct DialogContext {
    pub action: &'static dyn Action,
    pub files: Vec<DroppedFile>,
}

#[derive(Default)]
pub struct Dialogs {
    next: AtomicU64,
    open: Mutex<HashMap<String, DialogContext>>,
}

impl Dialogs {
    pub fn context(&self, label: &str) -> Option<DialogContext> {
        self.open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(label)
            .cloned()
    }

    fn insert(&self, label: String, context: DialogContext) {
        self.open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(label, context);
    }

    fn remove(&self, label: &str) {
        self.open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(label);
    }
}

/// Opens a dialog for `action` on `files`. Several may be open at once.
pub fn open<R: Runtime>(
    app: &AppHandle<R>,
    action: &'static dyn Action,
    files: Vec<DroppedFile>,
) -> tauri::Result<()> {
    let Some(dialogs) = app.try_state::<Dialogs>() else {
        return Ok(());
    };
    let n = dialogs.next.fetch_add(1, Ordering::Relaxed) + 1;
    let label = format!("dialog-{n}");
    let subject = match files.as_slice() {
        [] => String::new(),
        [one] => one.name.clone(),
        [first, rest @ ..] => format!("{} and {} more", first.name, rest.len()),
    };
    dialogs.insert(label.clone(), DialogContext { action, files });

    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("dialog.html".into()))
        .title(format!("{} · {subject}", title(action)))
        .inner_size(900.0, 640.0)
        .min_inner_size(640.0, 480.0)
        .center()
        .build()?;
    let handle = app.clone();
    let closing = label.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event
            && let Some(dialogs) = handle.try_state::<Dialogs>()
        {
            dialogs.remove(&closing);
        }
    });
    // Comes up from the radial menu, which is always on top; bring it forward.
    window.set_focus()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles() {
        let find = |id| dropwheel_core::actions::find(id).unwrap();
        assert_eq!(title(find("convert.png")), "Convert to PNG");
        assert_eq!(title(find("add_background")), "Add background");
        assert_eq!(title(find("compress")), "Compress");
    }
}
