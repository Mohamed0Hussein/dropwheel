//! `%APPDATA%\<product name>\settings.json`.

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

/// Moving the shelf fires a stream of events; write once it has settled.
const SAVE_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub shelf: ShelfSettings,
    /// Where results go. `None` writes them next to each source file.
    pub output_dir: Option<PathBuf>,
    pub start_with_windows: bool,
    /// A Windows notification when a job finishes.
    pub notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shelf: ShelfSettings::default(),
            output_dir: None,
            start_with_windows: false,
            notifications: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShelfSettings {
    pub visible: bool,
    /// Outer position in physical pixels on the virtual desktop. `None` until first moved.
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Default for ShelfSettings {
    fn default() -> Self {
        ShelfSettings {
            visible: true,
            position: None,
        }
    }
}

/// Roaming AppData on Windows, so the folder is named after the product as people see it.
pub fn default_path<R: Runtime>(app: &AppHandle<R>) -> anyhow::Result<PathBuf> {
    let base = app
        .path()
        .config_dir()
        .context("could not find the AppData folder")?;
    Ok(base.join(&app.package_info().name).join("settings.json"))
}

/// The settings in memory, written back to disk in the background shortly after each
/// change.
pub struct SettingsStore {
    shared: Arc<Shared>,
    changed: mpsc::Sender<()>,
}

struct Shared {
    path: PathBuf,
    settings: Mutex<Settings>,
}

impl SettingsStore {
    /// Never fails: a missing file means first run, and an unreadable one is set aside so
    /// a bad edit cannot stop the app from starting.
    pub fn load(path: PathBuf) -> Self {
        let settings = read(&path);
        let shared = Arc::new(Shared {
            path,
            settings: Mutex::new(settings),
        });
        let (changed, rx) = mpsc::channel();
        let saver = shared.clone();
        let spawned = thread::Builder::new()
            .name("settings-saver".into())
            .spawn(move || save_when_settled(&saver, &rx));
        if let Err(err) = spawned {
            log::error!("settings will only be saved on quit: {err}");
        }
        SettingsStore { shared, changed }
    }

    pub fn path(&self) -> &Path {
        &self.shared.path
    }

    pub fn get(&self) -> Settings {
        self.shared.lock().clone()
    }

    pub fn update(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = self.shared.lock();
        let before = settings.clone();
        change(&mut settings);
        if *settings != before {
            // The saver only stops when the store is dropped.
            let _ = self.changed.send(());
        }
    }

    pub fn save_now(&self) -> anyhow::Result<()> {
        self.shared.save()
    }
}

impl Shared {
    /// A panic elsewhere while holding the lock leaves plain data behind, which is still
    /// fine to use.
    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.settings.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Writes to a temporary file and renames it over the old one, so a crash mid-write
    /// cannot leave a truncated file.
    fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_vec_pretty(&*self.lock())?;
        let dir = self
            .path
            .parent()
            .context("settings path has no parent folder")?;
        fs::create_dir_all(dir).with_context(|| format!("could not create {}", dir.display()))?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(&json)?;
        tmp.persist(&self.path)
            .with_context(|| format!("could not write {}", self.path.display()))?;
        Ok(())
    }
}

fn save_when_settled(shared: &Shared, changed: &mpsc::Receiver<()>) {
    while changed.recv().is_ok() {
        // Wait for a quiet spell, which also ends when the store is dropped.
        while changed.recv_timeout(SAVE_DELAY).is_ok() {}
        if let Err(err) = shared.save() {
            log::error!("could not save settings: {err:#}");
        }
    }
}

fn read(path: &Path) -> Settings {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Settings::default(),
        Err(err) => {
            log::error!("could not read {}: {err}; using defaults", path.display());
            return Settings::default();
        }
    };
    match serde_json::from_slice(&bytes) {
        Ok(settings) => settings,
        Err(err) => {
            let backup = path.with_extension("json.bad");
            log::error!(
                "{} is not valid ({err}); moved it to {} and using defaults",
                path.display(),
                backup.display()
            );
            if let Err(err) = fs::rename(path, &backup) {
                log::error!("could not move it aside: {err}");
            }
            Settings::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &tempfile::TempDir) -> SettingsStore {
        SettingsStore::load(dir.path().join("Dropwheel").join("settings.json"))
    }

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir).get();
        assert_eq!(s, Settings::default());
        assert!(s.shelf.visible);
    }

    #[test]
    fn round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let first = store(&dir);
        first.update(|s| {
            s.shelf.position = Some(Position { x: -1200, y: 340 });
            s.output_dir = Some(PathBuf::from(r"D:\نتائج"));
        });
        first.save_now().unwrap();
        assert_eq!(store(&dir).get(), first.get());
    }

    #[test]
    fn saves_in_the_background() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir);
        s.update(|s| s.shelf.visible = false);
        for _ in 0..50 {
            if s.path().exists() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        assert!(!store(&dir).get().shelf.visible);
    }

    #[test]
    fn partial_and_unknown_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            r#"{"shelf":{"position":{"x":5,"y":6}},"fromTheFuture":1}"#,
        )
        .unwrap();
        let s = SettingsStore::load(path).get();
        assert_eq!(s.shelf.position, Some(Position { x: 5, y: 6 }));
        assert!(s.shelf.visible);
        assert!(!s.start_with_windows);
    }

    #[test]
    fn corrupt_file_is_set_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"{ not json").unwrap();
        let s = SettingsStore::load(path.clone());
        assert_eq!(s.get(), Settings::default());
        assert!(!path.exists());
        assert_eq!(
            fs::read(dir.path().join("settings.json.bad")).unwrap(),
            b"{ not json"
        );
    }
}
