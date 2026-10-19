use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::kind::FileKind;

/// The two rings of wedges in the radial menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionGroup {
    Convert,
    Tools,
}

/// One thing the radial menu can do to a file.
pub trait Action: Send + Sync {
    /// Stable identifier shared with the UI, such as `convert.png` or `compress`.
    fn id(&self) -> &'static str;

    /// Wedge label, such as `PNG` or `COMPRESS`.
    fn label(&self) -> &'static str;

    fn group(&self) -> ActionGroup;

    /// Whether the menu should offer this action for a file of `kind`.
    fn applicable(&self, kind: FileKind) -> bool;

    /// Processes `input` and writes the result as a new file in the directory `out`,
    /// returning the path written.
    ///
    /// Implementations must leave `input` untouched and must not replace existing files;
    /// [`crate::output::write_new`] takes care of both.
    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf>;
}

/// Parameters chosen in an action's dialog. Each action reads its own variant and
/// rejects the others with [`Error::InvalidOptions`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Params {
    /// The action takes no input from the user, as with conversions and removing metadata.
    #[default]
    None,
    Compress(Compress),
    Crop(Crop),
    Edit(Edit),
    Background(Background),
}

/// How hard to compress.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Compress {
    /// Encoder quality from 1 to 100. For PNG, below 90 means fewer colours.
    Quality { percent: u8 },
    /// Aim for files no larger than this, lowering quality and then size as needed.
    Size { bytes: u64 },
}

/// A rectangle in the pixels of an upright image `ref_width` × `ref_height`. Applied to
/// an image of another size, it is scaled to match, so one crop serves a whole batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub ref_width: u32,
    pub ref_height: u32,
}

/// Mirrors the Edit dialog. Brightness and contrast run from -100 to 100, with 0 leaving
/// the image unchanged, and follow the CSS filters the dialog previews with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Edit {
    /// Clockwise: 0, 90, 180 or 270.
    pub rotate: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub grayscale: bool,
    pub brightness: i8,
    pub contrast: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background {
    /// RGB.
    pub color: [u8; 3],
}

type ProgressFn = dyn Fn(f32) + Send + Sync;

/// Everything an action needs besides its input and output locations.
#[derive(Clone, Default)]
pub struct Options {
    pub params: Params,
    pub cancel: CancelFlag,
    progress: Option<Arc<ProgressFn>>,
}

impl Options {
    pub fn new(params: Params) -> Self {
        Options {
            params,
            ..Options::default()
        }
    }

    /// Calls `f` with progress through the current file, from 0.0 to 1.0.
    pub fn with_progress(mut self, f: impl Fn(f32) + Send + Sync + 'static) -> Self {
        self.progress = Some(Arc::new(f));
        self
    }

    pub fn report(&self, fraction: f32) {
        if let Some(f) = &self.progress {
            f(fraction.clamp(0.0, 1.0));
        }
    }

    /// Long-running actions call this between steps so a cancelled job stops promptly.
    pub fn check_cancelled(&self) -> Result<()> {
        if self.cancel.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("params", &self.params)
            .field("cancel", &self.cancel)
            .field("progress", &self.progress.is_some())
            .finish()
    }
}

/// Shared between a running job and whoever may cancel it.
#[derive(Debug, Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn params_round_trip_through_json() {
        let json = serde_json::to_string(&Params::None).unwrap();
        assert_eq!(json, r#"{"type":"none"}"#);
        assert_eq!(serde_json::from_str::<Params>(&json).unwrap(), Params::None);
    }

    #[test]
    fn params_from_the_dialogs() {
        let parse = |s: &str| serde_json::from_str::<Params>(s).unwrap();
        assert_eq!(
            parse(r#"{"type":"compress","mode":"quality","percent":70}"#),
            Params::Compress(Compress::Quality { percent: 70 })
        );
        assert_eq!(
            parse(r#"{"type":"compress","mode":"size","bytes":500000}"#),
            Params::Compress(Compress::Size { bytes: 500_000 })
        );
        assert_eq!(
            parse(r#"{"type":"crop","x":1,"y":2,"width":3,"height":4,"refWidth":5,"refHeight":6}"#),
            Params::Crop(Crop {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
                ref_width: 5,
                ref_height: 6
            })
        );
        assert_eq!(
            parse(r#"{"type":"edit","rotate":90,"grayscale":true,"brightness":-20}"#),
            Params::Edit(Edit {
                rotate: 90,
                grayscale: true,
                brightness: -20,
                ..Edit::default()
            })
        );
        assert_eq!(
            parse(r#"{"type":"background","color":[255,128,0]}"#),
            Params::Background(Background {
                color: [255, 128, 0]
            })
        );
        assert!(serde_json::from_str::<Params>(r#"{"type":"edit","brightness":300}"#).is_err());
    }

    #[test]
    fn cancel_is_seen_by_clones() {
        let opts = Options::default();
        let job = opts.clone();
        assert!(job.check_cancelled().is_ok());
        opts.cancel.cancel();
        assert!(matches!(job.check_cancelled(), Err(Error::Cancelled)));
    }

    #[test]
    fn progress_is_clamped() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        let opts = Options::default().with_progress(move |p| sink.lock().unwrap().push(p));
        opts.report(-1.0);
        opts.report(0.5);
        opts.report(7.0);
        assert_eq!(*seen.lock().unwrap(), vec![0.0, 0.5, 1.0]);
    }
}
