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
    /// The action takes no input from the user, as with conversions.
    #[default]
    None,
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
