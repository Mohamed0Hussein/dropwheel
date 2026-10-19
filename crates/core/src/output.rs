//! Where results go. Originals are never touched and existing files are never replaced.

use std::fs::{self, File};
use std::io::{BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::paths;

/// Longest file name, in UTF-16 units on Windows and bytes elsewhere.
const MAX_NAME: usize = 255;
/// Give up looking for a free name after `photo_compressed (999).jpg`.
const MAX_ATTEMPTS: u32 = 999;

pub type Sink<'a> = BufWriter<&'a mut File>;

/// Writes a new file named `<stem>_<suffix>.<ext>` into `out_dir` and returns its path.
///
/// `write` fills a temporary file in `out_dir`, which is moved into place only once it
/// succeeds, so a failed or cancelled run leaves nothing behind. When the name is taken,
/// ` (2)`, ` (3)`, … is added, as Explorer does. `out_dir` is created if missing.
pub fn write_new(
    input: &Path,
    out_dir: &Path,
    suffix: &str,
    ext: &str,
    write: impl FnOnce(&mut Sink<'_>) -> Result<()>,
) -> Result<PathBuf> {
    validate_part("suffix", suffix)?;
    validate_part("extension", ext)?;
    let stem = input
        .file_stem()
        .ok_or_else(|| {
            Error::InvalidOptions(format!("\"{}\" is not a file.", paths::display(input)))
        })?
        .to_string_lossy();

    let dir = paths::extended(out_dir).map_err(|e| Error::io(out_dir, e))?;
    fs::create_dir_all(&dir).map_err(|e| Error::io(out_dir, e))?;
    let mut tmp = tempfile::Builder::new()
        .prefix(".dropwheel-")
        .suffix(".tmp")
        .tempfile_in(&dir)
        .map_err(|e| Error::io(out_dir, e))?;

    let mut sink = BufWriter::new(tmp.as_file_mut());
    write(&mut sink)?;
    sink.flush().map_err(|e| Error::io(out_dir, e))?;
    drop(sink);

    for n in 1..=MAX_ATTEMPTS {
        let target = dir.join(file_name(&stem, suffix, n, ext));
        match tmp.persist_noclobber(&target) {
            Ok(_) => return Ok(paths::strip_verbatim(&target)),
            Err(e) if e.error.kind() == ErrorKind::AlreadyExists => tmp = e.file,
            Err(e) => return Err(Error::io(paths::strip_verbatim(&target), e.error)),
        }
    }
    Err(Error::InvalidOptions(format!(
        "No free file name for \"{stem}_{suffix}.{ext}\" in \"{}\".",
        paths::display(out_dir)
    )))
}

/// Convenience for actions that produce their whole output in memory.
pub fn write_bytes(
    input: &Path,
    out_dir: &Path,
    suffix: &str,
    ext: &str,
    bytes: &[u8],
) -> Result<PathBuf> {
    write_new(input, out_dir, suffix, ext, |w| {
        w.write_all(bytes).map_err(|e| Error::io(out_dir, e))
    })
}

fn file_name(stem: &str, suffix: &str, n: u32, ext: &str) -> String {
    let counter = if n > 1 {
        format!(" ({n})")
    } else {
        String::new()
    };
    let tail = format!("_{suffix}{counter}.{ext}");
    let stem = truncate(stem, MAX_NAME.saturating_sub(name_len(&tail)));
    format!("{stem}{tail}")
}

fn name_len(s: &str) -> usize {
    s.chars().map(char_len).sum()
}

fn char_len(c: char) -> usize {
    if cfg!(windows) {
        c.len_utf16()
    } else {
        c.len_utf8()
    }
}

fn truncate(s: &str, budget: usize) -> &str {
    let mut used = 0;
    for (i, c) in s.char_indices() {
        used += char_len(c);
        if used > budget {
            return s.get(..i).unwrap_or_default();
        }
    }
    s
}

/// Suffixes and extensions come from code and settings, but keep them from escaping the
/// output directory or producing names Windows rejects.
fn validate_part(what: &str, s: &str) -> Result<()> {
    let bad = |c: char| c.is_control() || r#"<>:"/\|?*"#.contains(c);
    if s.is_empty() || s.chars().count() > 64 || s.chars().any(bad) || s.ends_with(['.', ' ']) {
        return Err(Error::InvalidOptions(format!(
            "Invalid file name {what} \"{s}\"."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    fn write(input: &Path, out: &Path, data: &[u8]) -> PathBuf {
        write_bytes(input, out, "compressed", "jpg", data).unwrap()
    }

    #[test]
    fn writes_next_to_input_with_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("photo.jpg");
        fs::write(&input, b"original").unwrap();

        let out = write(&input, dir.path(), b"result");
        assert_eq!(out, dir.path().join("photo_compressed.jpg"));
        assert_eq!(fs::read(&out).unwrap(), b"result");
        assert_eq!(fs::read(&input).unwrap(), b"original");
    }

    #[test]
    fn never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("photo.jpg");
        fs::write(dir.path().join("photo_compressed.jpg"), b"keep me").unwrap();

        let second = write(&input, dir.path(), b"2");
        let third = write(&input, dir.path(), b"3");
        assert!(second.ends_with("photo_compressed (2).jpg"));
        assert!(third.ends_with("photo_compressed (3).jpg"));
        assert_eq!(
            fs::read(dir.path().join("photo_compressed.jpg")).unwrap(),
            b"keep me"
        );
    }

    #[test]
    fn failed_write_leaves_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("doc.pdf");
        fs::write(&input, b"%PDF-").unwrap();

        let err = write_new(&input, dir.path(), "x", "pdf", |_| Err(Error::Cancelled));
        assert!(matches!(err, Err(Error::Cancelled)));
        assert_eq!(names(dir.path()), ["doc.pdf"]);
    }

    #[test]
    fn unicode_names() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("تقرير المبيعات ٢٠٢٦.docx");
        let out = write_bytes(&input, dir.path(), "converted", "pdf", b"x").unwrap();
        assert_eq!(
            out.file_name().unwrap().to_str().unwrap(),
            "تقرير المبيعات ٢٠٢٦_converted.pdf"
        );
    }

    #[test]
    fn creates_missing_output_dir() {
        let dir = tempfile::tempdir().unwrap();
        let out_dir = dir.path().join("نتائج").join("nested");
        let out = write(&dir.path().join("a.jpg"), &out_dir, b"x");
        assert_eq!(out, out_dir.join("a_compressed.jpg"));
    }

    #[test]
    fn long_names_are_shortened() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join(format!("{}.jpg", "ص".repeat(200)));
        let out = write(&input, dir.path(), b"x");
        let name = out.file_name().unwrap().to_str().unwrap();
        assert!(name_len(name) <= MAX_NAME, "{}", name_len(name));
        assert!(name.ends_with("_compressed.jpg"));
        assert!(name.starts_with("صص"));
    }

    #[test]
    fn long_directory_paths() {
        let dir = tempfile::tempdir().unwrap();
        let mut deep = dir.path().to_path_buf();
        for i in 0..12 {
            deep.push(format!("folder number {i:02} with a long name"));
        }
        assert!(deep.as_os_str().len() > 300);
        let input = deep.join("photo.jpg");
        fs::create_dir_all(&deep).unwrap();
        fs::write(&input, b"original").unwrap();

        let out = write(&input, &deep, b"result");
        assert_eq!(out, deep.join("photo_compressed.jpg"));
        assert_eq!(fs::read(&out).unwrap(), b"result");
    }

    #[test]
    fn rejects_bad_suffixes() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("a.png");
        for suffix in ["", "../escape", r"a\b", "con:", "trailing.", "x\n"] {
            let err = write_bytes(&input, dir.path(), suffix, "png", b"").unwrap_err();
            assert_eq!(err.code(), "invalid_options", "{suffix:?}");
        }
        assert!(names(dir.path()).is_empty());
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("abc", 2), "ab");
        assert_eq!(truncate("", 0), "");
        let arabic = "صورة";
        let cut = truncate(arabic, 3);
        assert!(arabic.starts_with(cut) && name_len(cut) <= 3);
    }
}
