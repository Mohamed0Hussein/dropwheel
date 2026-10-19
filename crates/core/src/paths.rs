//! Windows path handling.
//!
//! `std::fs` already lifts the 260-character `MAX_PATH` limit by switching to `\\?\`
//! paths internally. Win32 calls and C libraries that take a path do not, so code that
//! hands a path to them goes through [`extended`] first. Paths shown to people, or given
//! to Explorer, go through [`display`] or [`strip_verbatim`] instead.

use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};

const VERBATIM: &str = r"\\?\";
const VERBATIM_UNC: &str = r"\\?\UNC\";
const DEVICE: &str = r"\\.\";

/// The path as a person would type it, without any `\\?\` prefix.
pub fn display(path: &Path) -> String {
    strip_verbatim_str(&path.to_string_lossy()).into_owned()
}

/// Undoes [`extended`], for passing a path to programs that reject `\\?\` paths.
pub fn strip_verbatim(path: &Path) -> PathBuf {
    match path.to_str() {
        Some(s) => PathBuf::from(strip_verbatim_str(s).as_ref()),
        None => path.to_path_buf(),
    }
}

/// Makes `path` absolute and, on Windows, converts it to the `\\?\` form that APIs accept
/// beyond `MAX_PATH`. Other platforms only make it absolute.
pub fn extended(path: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    if cfg!(windows)
        && let Some(s) = absolute.to_str()
    {
        return Ok(PathBuf::from(to_verbatim_str(s).as_ref()));
    }
    Ok(absolute)
}

fn strip_verbatim_str(s: &str) -> Cow<'_, str> {
    if let Some(rest) = s.strip_prefix(VERBATIM_UNC) {
        Cow::Owned(format!(r"\\{rest}"))
    } else if let Some(rest) = s.strip_prefix(VERBATIM) {
        // Only drive paths have a plain equivalent; leave `\\?\Volume{…}\` and the like.
        if is_drive_absolute(rest) {
            Cow::Borrowed(rest)
        } else {
            Cow::Borrowed(s)
        }
    } else {
        Cow::Borrowed(s)
    }
}

/// Expects an absolute, normalised path: verbatim paths skip all normalisation, so `..`
/// and `/` would be taken literally. [`std::path::absolute`] provides this on Windows.
fn to_verbatim_str(s: &str) -> Cow<'_, str> {
    if s.starts_with(VERBATIM) || s.starts_with(DEVICE) {
        Cow::Borrowed(s)
    } else if let Some(rest) = s.strip_prefix(r"\\") {
        Cow::Owned(format!("{VERBATIM_UNC}{rest}"))
    } else if is_drive_absolute(s) {
        Cow::Owned(format!("{VERBATIM}{s}"))
    } else {
        Cow::Borrowed(s)
    }
}

fn is_drive_absolute(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_verbatim_prefix() {
        assert_eq!(to_verbatim_str(r"C:\a\b.png"), r"\\?\C:\a\b.png");
        assert_eq!(
            to_verbatim_str(r"\\server\share\ملف.pdf"),
            r"\\?\UNC\server\share\ملف.pdf"
        );
        assert_eq!(to_verbatim_str(r"\\?\C:\a"), r"\\?\C:\a");
        assert_eq!(to_verbatim_str(r"\\.\PIPE\x"), r"\\.\PIPE\x");
        assert_eq!(to_verbatim_str("relative"), "relative");
    }

    #[test]
    fn strips_verbatim_prefix() {
        assert_eq!(strip_verbatim_str(r"\\?\C:\a\b.png"), r"C:\a\b.png");
        assert_eq!(strip_verbatim_str(r"\\?\UNC\srv\share\x"), r"\\srv\share\x");
        assert_eq!(strip_verbatim_str(r"C:\plain"), r"C:\plain");
        let volume = r"\\?\Volume{0b1c}\x";
        assert_eq!(strip_verbatim_str(volume), volume);
    }

    #[test]
    fn round_trips() {
        for p in [r"C:\Users\سارة\Desktop\صورة.jpg", r"\\nas\photos\a.png"] {
            assert_eq!(strip_verbatim_str(&to_verbatim_str(p)), p);
        }
    }

    #[test]
    fn extended_is_absolute() {
        let p = extended(Path::new("some/relative.txt")).unwrap();
        assert!(p.is_absolute());
        assert!(p.ends_with("relative.txt"));
    }

    #[cfg(windows)]
    #[test]
    fn extended_on_windows() {
        let p = extended(Path::new(r"C:\a\..\b\c.txt")).unwrap();
        assert_eq!(p, Path::new(r"\\?\C:\b\c.txt"));
        assert_eq!(display(&p), r"C:\b\c.txt");
    }
}
