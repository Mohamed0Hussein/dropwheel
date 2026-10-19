//! Runs an action over files on a background thread, reporting progress and the outcome
//! as events.

use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use dropwheel_core::{Action, CancelFlag, Error, Options, Params, paths};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::integration;
use crate::settings::SettingsStore;

pub const STARTED_EVENT: &str = "job:started";
pub const PROGRESS_EVENT: &str = "job:progress";
pub const FINISHED_EVENT: &str = "job:finished";

/// Progress events are capped at about 30 a second.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(33);

/// Cancel flags of the jobs still running.
#[derive(Default)]
pub struct Jobs {
    next: AtomicU64,
    running: Mutex<HashMap<u64, CancelFlag>>,
}

impl Jobs {
    pub fn cancel(&self, job_id: u64) -> bool {
        let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running.get(&job_id).map(CancelFlag::cancel).is_some()
    }

    fn begin(&self) -> (u64, CancelFlag) {
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let flag = CancelFlag::default();
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, flag.clone());
        (id, flag)
    }

    fn end(&self, job_id: u64) {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&job_id);
    }
}

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
    pub job_id: u64,
    pub outputs: Vec<Output>,
    pub failures: Vec<Failure>,
    pub cancelled: bool,
    /// A few words for the shelf, such as "Saved 3 files".
    pub summary: String,
    /// The full story for a tooltip or notification.
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub path: String,
    pub bytes: u64,
    pub input_bytes: u64,
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
    params: Params,
) -> u64 {
    let Some(jobs) = app.try_state::<Jobs>() else {
        log::error!("job registry missing");
        return 0;
    };
    let (job_id, cancel) = jobs.begin();
    let output_dir = app
        .try_state::<SettingsStore>()
        .and_then(|s| s.get().output_dir);
    let worker_app = app.clone();
    let spawned = thread::Builder::new()
        .name(format!("job-{job_id}"))
        .spawn(move || {
            let job = Job {
                id: job_id,
                action,
                params,
                cancel,
            };
            run(&worker_app, &job, &files, output_dir.as_deref());
            if let Some(jobs) = worker_app.try_state::<Jobs>() {
                jobs.end(job_id);
            }
        });
    if let Err(err) = spawned {
        log::error!("could not start job {job_id}: {err}");
        jobs.end(job_id);
        let failures = vec![Failure {
            file: String::new(),
            code: "internal",
            message: format!("Could not start: {err}"),
        }];
        emit(
            app,
            FINISHED_EVENT,
            finished(job_id, Vec::new(), failures, false),
        );
    }
    job_id
}

struct Job {
    id: u64,
    action: &'static dyn Action,
    params: Params,
    cancel: CancelFlag,
}

fn run<R: Runtime>(app: &AppHandle<R>, job: &Job, files: &[PathBuf], output_dir: Option<&Path>) {
    let total = files.len();
    log::info!("job {}: {} on {total} file(s)", job.id, job.action.id());
    emit(
        app,
        STARTED_EVENT,
        Started {
            job_id: job.id,
            action: job.action.label(),
            total,
        },
    );

    let mut outputs = Vec::new();
    let mut failures = Vec::new();
    for (done, input) in files.iter().enumerate() {
        if job.cancel.is_cancelled() {
            break;
        }
        let opts = options_for(app, job, done, total);
        match run_one(job.action, input, &opts, output_dir) {
            Ok(path) => outputs.push(Output {
                bytes: file_size(&path),
                input_bytes: file_size(input),
                path: paths::display(&path),
            }),
            Err(Error::Cancelled) => break,
            Err(err) => {
                log::warn!("job {}: {}: {err}", job.id, input.display());
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
                job_id: job.id,
                done: done + 1,
                total,
                fraction: (done + 1) as f32 / total as f32,
            },
        );
    }

    let report = finished(job.id, outputs, failures, job.cancel.is_cancelled());
    log::info!("job {}: {}", job.id, report.summary);
    notify(app, job.action, &report);
    emit(app, FINISHED_EVENT, report);
}

/// A Windows notification for a finished job, unless it was cancelled or notifications
/// are off.
fn notify<R: Runtime>(app: &AppHandle<R>, action: &dyn Action, report: &Finished) {
    let wanted = app
        .try_state::<SettingsStore>()
        .is_none_or(|s| s.get().notifications);
    if !wanted || report.cancelled {
        return;
    }
    let target = report.outputs.first().map(|o| PathBuf::from(&o.path));
    let body = match (report.outputs.as_slice(), report.failures.first()) {
        ([only], None) => {
            let name = Path::new(&only.path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            format!("Saved {name}")
        }
        (_, Some(failure)) if report.outputs.is_empty() => failure.message.clone(),
        _ => report.summary.clone(),
    };
    integration::notify(
        app,
        integration::Notice {
            title: crate::dialog::title(action),
            body,
            several: report.outputs.len() > 1,
            target,
        },
    );
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
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

fn options_for<R: Runtime>(app: &AppHandle<R>, job: &Job, done: usize, total: usize) -> Options {
    let app = app.clone();
    let job_id = job.id;
    let last = Mutex::new(Instant::now());
    let mut opts = Options::new(job.params.clone()).with_progress(move |within| {
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
    });
    opts.cancel = job.cancel.clone();
    opts
}

fn finished(
    job_id: u64,
    outputs: Vec<Output>,
    failures: Vec<Failure>,
    cancelled: bool,
) -> Finished {
    let paths: Vec<&str> = outputs.iter().map(|o| o.path.as_str()).collect();
    let (mut summary, detail) = describe(&paths, &failures);
    if cancelled {
        summary = if outputs.is_empty() {
            "Cancelled".to_string()
        } else {
            format!("Cancelled after {}", plural(outputs.len(), "file"))
        };
    }
    Finished {
        job_id,
        outputs,
        failures,
        cancelled,
        summary,
        detail,
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn describe(outputs: &[&str], failures: &[Failure]) -> (String, String) {
    let saved = outputs.len();
    let failed = failures.len();
    let summary = match (saved, failed) {
        (0, 0) => "Nothing to do".to_string(),
        (1, 0) => "Saved".to_string(),
        (n, 0) => format!("Saved {n} files"),
        (0, _) if failures.iter().all(|f| f.code == "not_available") => {
            "Not available yet".to_string()
        }
        (0, 1) => "Failed".to_string(),
        (0, n) => format!("{n} files failed"),
        (ok, bad) => format!("{ok} saved, {bad} failed"),
    };

    let mut lines: Vec<String> = outputs.iter().map(|p| format!("Saved {p}")).collect();
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
        let names = ["0", "1", "2"];
        let out = |n: usize| -> Vec<&str> { names[..n].to_vec() };
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

        let (_, detail) = describe(&["a_small.jpg"], &[failure("locked", "busy")]);
        assert_eq!(detail, "Saved a_small.jpg\nbusy");
    }

    #[test]
    fn cancelled_summaries() {
        let output = Output {
            path: "a".into(),
            bytes: 1,
            input_bytes: 2,
        };
        assert_eq!(
            finished(1, Vec::new(), Vec::new(), true).summary,
            "Cancelled"
        );
        assert_eq!(
            finished(1, vec![output], Vec::new(), true).summary,
            "Cancelled after 1 file"
        );
    }

    struct Fixed(fn() -> dropwheel_core::Result<PathBuf>);

    impl Action for Fixed {
        fn id(&self) -> &'static str {
            "fixed"
        }
        fn label(&self) -> &'static str {
            "FIXED"
        }
        fn group(&self) -> dropwheel_core::ActionGroup {
            dropwheel_core::ActionGroup::Tools
        }
        fn applicable(&self, _: dropwheel_core::FileKind) -> bool {
            true
        }
        fn run(&self, _: &Path, _: &Options, _: &Path) -> dropwheel_core::Result<PathBuf> {
            (self.0)()
        }
    }

    #[test]
    fn panics_cost_one_file() {
        let boom = Fixed(|| panic!("malformed input"));
        let err = run_one(&boom, Path::new("/tmp/a.png"), &Options::default(), None).unwrap_err();
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
