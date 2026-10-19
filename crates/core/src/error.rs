use std::io;
use std::path::{Path, PathBuf};

use crate::kind::FileKind;
use crate::paths;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors carry a message meant for the user, so the app can show `to_string()` as is.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("\"{}\" is open in another program. Close it and try again.", paths::display(.path))]
    Locked { path: PathBuf },

    #[error("Access to \"{}\" was denied.", paths::display(.path))]
    PermissionDenied { path: PathBuf },

    #[error("\"{}\" was not found.", paths::display(.path))]
    NotFound { path: PathBuf },

    #[error("There is not enough disk space to write \"{}\".", paths::display(.path))]
    DiskFull { path: PathBuf },

    #[error("Could not access \"{}\": {source}", paths::display(.path))]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{action} does not support {} files.", .kind.label())]
    Unsupported {
        action: &'static str,
        kind: FileKind,
    },

    #[error("\"{}\" could not be read as {}: {reason}", paths::display(.path), .kind.label())]
    Decode {
        path: PathBuf,
        kind: FileKind,
        reason: String,
    },

    #[error("Could not write {}: {reason}", .kind.label())]
    Encode { kind: FileKind, reason: String },

    #[error("\"{}\" is protected with a password. Save a copy without the password in a PDF reader, then try again.", paths::display(.path))]
    Protected { path: PathBuf },

    /// A library this build depends on at run time could not be loaded.
    #[error("{what} is not available in this build: {reason}")]
    MissingLibrary { what: &'static str, reason: String },

    #[error("Invalid options: {0}")]
    InvalidOptions(String),

    #[error("Cancelled.")]
    Cancelled,

    /// A bug, such as a panic caught at the edge of an action. The log has the details.
    #[error("Something went wrong while processing this file. The log has details.")]
    Internal,

    /// The action is in the menu but its implementation has not landed yet.
    #[error("{action} is not available yet.")]
    NotAvailable { action: String },
}

impl Error {
    /// Wraps an I/O error on `path`, sorting out the cases the user can act on.
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        let path = path.as_ref().to_path_buf();
        if is_lock_violation(&source) {
            return Error::Locked { path };
        }
        match source.kind() {
            io::ErrorKind::NotFound => Error::NotFound { path },
            io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => {
                Error::PermissionDenied { path }
            }
            io::ErrorKind::ResourceBusy => Error::Locked { path },
            io::ErrorKind::StorageFull => Error::DiskFull { path },
            _ => Error::Io { path, source },
        }
    }

    /// A stable, machine-readable code for the UI.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Locked { .. } => "locked",
            Error::PermissionDenied { .. } => "permission_denied",
            Error::NotFound { .. } => "not_found",
            Error::DiskFull { .. } => "disk_full",
            Error::Io { .. } => "io",
            Error::Unsupported { .. } => "unsupported",
            Error::Decode { .. } => "decode",
            Error::Encode { .. } => "encode",
            Error::Protected { .. } => "protected",
            Error::MissingLibrary { .. } => "missing_library",
            Error::InvalidOptions(_) => "invalid_options",
            Error::Cancelled => "cancelled",
            Error::Internal => "internal",
            Error::NotAvailable { .. } => "not_available",
        }
    }
}

/// Windows reports a file held open by another process as a sharing or lock violation
/// rather than as a permission error.
fn is_lock_violation(err: &io::Error) -> bool {
    const ERROR_SHARING_VIOLATION: i32 = 32;
    const ERROR_LOCK_VIOLATION: i32 = 33;
    const ERROR_USER_MAPPED_FILE: i32 = 1224;
    cfg!(windows)
        && matches!(
            err.raw_os_error(),
            Some(ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION | ERROR_USER_MAPPED_FILE)
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_io_errors() {
        let p = Path::new("a.txt");
        let err = |kind| Error::io(p, io::Error::from(kind));
        assert_eq!(err(io::ErrorKind::NotFound).code(), "not_found");
        assert_eq!(
            err(io::ErrorKind::PermissionDenied).code(),
            "permission_denied"
        );
        assert_eq!(err(io::ErrorKind::StorageFull).code(), "disk_full");
        assert_eq!(err(io::ErrorKind::UnexpectedEof).code(), "io");
    }

    #[cfg(windows)]
    #[test]
    fn sharing_violation_is_locked() {
        let err = Error::io("a.txt", io::Error::from_raw_os_error(32));
        assert_eq!(err.code(), "locked");
    }

    #[test]
    fn messages_hide_verbatim_prefix() {
        let err = Error::PermissionDenied {
            path: PathBuf::from(r"\\?\C:\Users\مستخدم\صورة.png"),
        };
        assert_eq!(
            err.to_string(),
            r#"Access to "C:\Users\مستخدم\صورة.png" was denied."#
        );
    }
}
