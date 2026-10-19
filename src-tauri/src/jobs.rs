//! Runs an action over the dropped files on a background thread, reporting progress
//! and the outcome as events.

use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use dropwheel_core::{Action, Error, Options, paths};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::settings::SettingsStore;

pub const STARTED_EVENT: &str = "job:started";
pub const PROGRESS_EVENT: &str = "job:progress";
pub const FINISHED_EVENT: &str = "job:finished";

/// Progress events are capped at about 30 a second.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(33);

static NEXT_JOB: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Started {
    job_id: u64,
    action: &'static str,
    total: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    job_id: u64,
    /// Files finished so far.
    done: usize,
    total: usize,
    /// Overall, from 0.0 to 1.0.
    fraction: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    job_id: u64,
    outputs: Vec<String>,
    failures: Vec<Failure>,
    /// A few words for the shelf, such as "Saved 3 files".
    summary: String,
    /// The full story for a tooltip or notification.
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    file: String,
    code: &'static str,
    message: String,
}

/// Starts `action` on `files` and returns the job id its events will carry.
pub fn start<R: Runtime>(
    app: &AppHandle<R>,
    action: &'static dyn Action,
    files: Vec<PathBuf>,
) -> u64 {
    let job_id = NEXT_JOB.fetch_add(1, Ordering::Relaxed);
    let output_dir = app
        .try_state::<SettingsStore>()
        .and_then(|s| s.get().output_dir);
    let worker_app = app.clone();
    let spawned = thread::Builder::new()
        .name(format!("job-{job_id}"))
        .spawn(move || run(&worker_app, job_id, action, &files, output_dir.as_deref()));
    if let Err(err) = spawned {
        log::error!("could not start job {job_id}: {err}");
        let failures = vec![Failure {
            file: String::new(),
            code: "internal",
            message: format!("Could not start: {err}"),
        }];
        emit(app, FINISHED_EVENT, finished(job_id, Vec::new(), failures));
    }
    job_id
}

fn run<R: Runtime>(
    app: &AppHandle<R>,
    job_id: u64,
    action: &'static dyn Action,
    files: &[PathBuf],
    output_dir: Option<&Path>,
) {
    let total = files.len();
    log::info!("job {job_id}: {} on {total} file(s)", action.id());
    emit(
        app,
        STARTED_EVENT,
        Started {
            job_id,
            action: action.label(),
            total,
        },
    );

    let mut outputs = Vec::new();
    let mut failures = Vec::new();
    for (done, input) in files.iter().enumerate() {
        let opts = progress_reporter(app, job_id, done, total);
        match run_one(action, input, &opts, output_dir) {
            Ok(path) => outputs.push(path),
            Err(err) => {
                log::warn!("job {job_id}: {}: {err}", input.display());
                failures.push(Failure {
                    file: paths::display(input),
                    code: err.code(),
                    message: err.to_string(),
                });
            }
        }
        emit(
            app,
            PROGRESS_EVENT,
            Progress {
                job_id,
                done: done + 1,
                total,
                fraction: (done + 1) as f32 / total as f32,
            },
        );
    }

    let report = finished(job_id, outputs, failures);
    log::info!("job {job_id}: {}", report.summary);
    emit(app, FINISHED_EVENT, report);
}

/// Runs the action on one file. A panic inside it is a bug, but it must cost one file,
/// not the app.
fn run_one(
    action: &dyn Action,
    input: &Path,
    opts: &Options,
    output_dir: Option<&Path>,
) -> Result<PathBuf, Error> {
    let out = match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => input
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| Error::NotFound {
                path: input.to_path_buf(),
            })?,
    };
    panic::catch_unwind(AssertUnwindSafe(|| action.run(input, opts, &out))).unwrap_or_else(
        |payload| {
            let what = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            log::error!("{} panicked on {}: {what}", action.id(), input.display());
            Err(Error::Internal)
        },
    )
}

fn progress_reporter<R: Runtime>(
    app: &AppHandle<R>,
    job_id: u64,
    done: usize,
    total: usize,
) -> Options {
    let app = app.clone();
    let last = Mutex::new(Instant::now());
    Options::default().with_progress(move |within| {
        let Ok(mut last) = last.lock() else {
            return;
        };
        if last.elapsed() < PROGRESS_INTERVAL {
            return;
        }
        *last = Instant::now();
        emit(
            &app,
            PROGRESS_EVENT,
            Progress {
                job_id,
                done,
                total,
                fraction: (done as f32 + within) / total as f32,
            },
        );
    })
}

fn finished(job_id: u64, outputs: Vec<PathBuf>, failures: Vec<Failure>) -> Finished {
    let (summary, detail) = describe(&outputs, &failures);
    Finished {
        job_id,
        outputs: outputs.iter().map(|p| paths::display(p)).collect(),
        failures,
        summary,
        detail,
    }
}

fn describe(outputs: &[PathBuf], failures: &[Failure]) -> (String, String) {
    let saved = outputs.len();
    let failed = failures.len();
    let summary = match (saved, failed) {
        (0, 0) => "Nothing to do".to_string(),
        (1, 0) => "Saved".to_string(),
        (n, 0) => format!("Saved {n} files"),
        (0, _) if failures.iter().all(|f| f.code == "not_available") => {
            "Not available yet".to_string()
        }
        (0, _) if failures.iter().all(|f| f.code == "cancelled") => "Cancelled".to_string(),
        (0, 1) => "Failed".to_string(),
        (0, n) => format!("{n} files failed"),
        (ok, bad) => format!("{ok} saved, {bad} failed"),
    };

    let mut lines: Vec<String> = outputs
        .iter()
        .map(|p| format!("Saved {}", paths::display(p)))
        .collect();
    let mut messages: Vec<&str> = failures.iter().map(|f| f.message.as_str()).collect();
    messages.dedup();
    lines.extend(messages.into_iter().map(str::to_string));
    (summary, lines.join("\n"))
}

fn emit<R: Runtime, S: Serialize + Clone>(app: &AppHandle<R>, event: &str, payload: S) {
    if let Err(err) = app.emit(event, payload) {
        log::warn!("could not send {event}: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(code: &'static str, message: &str) -> Failure {
        Failure {
            file: "x".into(),
            code,
            message: message.into(),
        }
    }

    #[test]
    fn summaries() {
        let out =
            |n: usize| -> Vec<PathBuf> { (0..n).map(|i| PathBuf::from(format!("{i}"))).collect() };
        let na = || failure("not_available", "Compress is not available yet.");
        let bad = || failure("decode", "broken");

        assert_eq!(describe(&out(0), &[]).0, "Nothing to do");
        assert_eq!(describe(&out(1), &[]).0, "Saved");
        assert_eq!(describe(&out(3), &[]).0, "Saved 3 files");
        assert_eq!(describe(&out(0), &[na(), na()]).0, "Not available yet");
        assert_eq!(describe(&out(0), &[bad()]).0, "Failed");
        assert_eq!(describe(&out(0), &[bad(), na()]).0, "2 files failed");
        assert_eq!(describe(&out(2), &[bad()]).0, "2 saved, 1 failed");
    }

    #[test]
    fn detail_lists_outputs_and_distinct_errors() {
        let na = failure("not_available", "Compress is not available yet.");
        let (_, detail) = describe(&[], &[na.clone(), na]);
        assert_eq!(detail, "Compress is not available yet.");

        let (_, detail) = describe(
            &[PathBuf::from("a_small.jpg")],
            &[failure("locked", "busy")],
        );
        assert_eq!(detail, "Saved a_small.jpg\nbusy");
    }

    #[test]
    fn panics_cost_one_file() {
        struct Boom;
        impl Action for Boom {
            fn id(&self) -> &'static str {
                "boom"
            }
            fn label(&self) -> &'static str {
                "BOOM"
            }
            fn group(&self) -> dropwheel_core::ActionGroup {
                dropwheel_core::ActionGroup::Tools
            }
            fn applicable(&self, _: dropwheel_core::FileKind) -> bool {
                true
            }
            fn run(&self, _: &Path, _: &Options, _: &Path) -> dropwheel_core::Result<PathBuf> {
                panic!("malformed input");
            }
        }
        let err = run_one(&Boom, Path::new("/tmp/a.png"), &Options::default(), None).unwrap_err();
        assert_eq!(err.code(), "internal");
    }

    #[test]
    fn output_dir_defaults_to_the_source_folder() {
        struct Echo;
        impl Action for Echo {
            fn id(&self) -> &'static str {
                "echo"
            }
            fn label(&self) -> &'static str {
                "ECHO"
            }
            fn group(&self) -> dropwheel_core::ActionGroup {
                dropwheel_core::ActionGroup::Tools
            }
            fn applicable(&self, _: dropwheel_core::FileKind) -> bool {
                true
            }
            fn run(&self, _: &Path, _: &Options, out: &Path) -> dropwheel_core::Result<PathBuf> {
                Ok(out.to_path_buf())
            }
        }
        let input = Path::new("/photos/a.png");
        let opts = Options::default();
        assert_eq!(
            run_one(&Echo, input, &opts, None).unwrap(),
            Path::new("/photos")
        );
        let custom = Path::new("/results");
        assert_eq!(run_one(&Echo, input, &opts, Some(custom)).unwrap(), custom);
    }
}
