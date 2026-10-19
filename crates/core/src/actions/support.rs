//! Steps every image action shares: open, re-save in the same format, write safely.

use std::path::{Path, PathBuf};

use image::DynamicImage;

use crate::action::Options;
use crate::error::{Error, Result};
use crate::imaging::{self, Decoded, EncodeOptions};
use crate::kind::FileKind;
use crate::output;

/// Detects and decodes `input`, checking for cancellation on either side.
pub fn load(input: &Path, opts: &Options) -> Result<(FileKind, Decoded)> {
    opts.check_cancelled()?;
    let kind = FileKind::detect(input)?;
    let decoded = imaging::decode(input, kind)?;
    opts.check_cancelled()?;
    opts.report(0.4);
    Ok((kind, decoded))
}

/// Tools registered for documents only work on images until phase 5 brings PDF and DOCX
/// support.
pub fn require_image(kind: FileKind, what: &str) -> Result<()> {
    if kind.is_image() {
        Ok(())
    } else {
        Err(Error::NotAvailable {
            action: format!("{what} {} files", kind.label()),
        })
    }
}

/// How to write an edited image back in its own format, close to its original quality.
/// The original quality of a lossy file is unknown, so this errs high to avoid visible
/// loss. A format this machine cannot write (HEIC without Microsoft's encoder, or ICO)
/// becomes the nearest one it can: JPEG for photos, PNG for icons.
pub fn rewrite_as<'a>(kind: FileKind) -> (FileKind, EncodeOptions<'a>) {
    let target = match kind {
        FileKind::Ico => FileKind::Png,
        k if imaging::can_encode(k) => k,
        _ => FileKind::Jpeg,
    };
    let lossy_source = matches!(
        kind,
        FileKind::Jpeg | FileKind::Webp | FileKind::Avif | FileKind::Heic
    );
    let opts = EncodeOptions {
        quality: if target == FileKind::Avif { 85 } else { 92 },
        lossless: !lossy_source,
        ..EncodeOptions::default()
    };
    (target, opts)
}

/// Encodes `image` and writes it next to `input` (or into `out`) with `suffix`.
pub fn save(
    input: &Path,
    out: &Path,
    suffix: &str,
    image: &DynamicImage,
    target: FileKind,
    encode: &EncodeOptions,
    opts: &Options,
) -> Result<PathBuf> {
    let bytes = imaging::encode(image, target, encode)?;
    opts.check_cancelled()?;
    opts.report(0.9);
    output::write_bytes(input, out, suffix, target.extension(), &bytes)
}

/// Re-saves `image`, decoded from a `kind` file, in that format where possible, keeping
/// its colour profile.
pub fn save_like(
    input: &Path,
    out: &Path,
    suffix: &str,
    kind: FileKind,
    image: &DynamicImage,
    icc: Option<&[u8]>,
    opts: &Options,
) -> Result<PathBuf> {
    let (target, mut encode) = rewrite_as(kind);
    encode.icc = icc;
    save(input, out, suffix, image, target, &encode, opts)
}
