use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// What a file is, judged by its content first and its extension second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Jpeg,
    Png,
    Webp,
    Bmp,
    Tiff,
    Gif,
    Ico,
    Avif,
    Heic,
    Pdf,
    Docx,
    Txt,
    Unknown,
}

/// Enough to cover a PDF header preceded by junk and a long ISO-BMFF `ftyp` box.
const SNIFF_LEN: usize = 1024;

impl FileKind {
    /// Detects the kind of the file at `path` by reading its first bytes.
    ///
    /// Binary formats are only recognised by their signature, so a corrupt `photo.jpg` is
    /// [`FileKind::Unknown`] rather than a JPEG that fails later. Plain text has no
    /// signature and is recognised by extension.
    pub fn detect(path: &Path) -> Result<FileKind> {
        let mut file = File::open(path).map_err(|e| Error::io(path, e))?;
        let mut head = Vec::with_capacity(SNIFF_LEN);
        file.by_ref()
            .take(SNIFF_LEN as u64)
            .read_to_end(&mut head)
            .map_err(|e| Error::io(path, e))?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        Ok(Self::classify(&head, ext))
    }

    /// Classifies a file from its leading bytes and its extension (without the dot).
    pub fn classify(head: &[u8], extension: &str) -> FileKind {
        let ext = extension.to_ascii_lowercase();
        match Self::sniff(head) {
            Some(Sniffed::Zip) if ext == "docx" => FileKind::Docx,
            Some(Sniffed::Zip) => FileKind::Unknown,
            Some(Sniffed::Kind(kind)) => kind,
            None if is_text_extension(&ext) && looks_like_text(head) => FileKind::Txt,
            None => FileKind::Unknown,
        }
    }

    fn sniff(head: &[u8]) -> Option<Sniffed> {
        let kind = |k| Some(Sniffed::Kind(k));
        if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return kind(FileKind::Jpeg);
        }
        if head.starts_with(b"\x89PNG\r\n\x1a\n") {
            return kind(FileKind::Png);
        }
        if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
            return kind(FileKind::Gif);
        }
        if head.len() >= 12
            && head.starts_with(b"RIFF")
            && head.get(8..12) == Some(b"WEBP".as_slice())
        {
            return kind(FileKind::Webp);
        }
        // "BM" alone is too weak; also require the reserved header fields to be zero.
        if head.starts_with(b"BM") && head.get(6..10) == Some([0u8; 4].as_slice()) {
            return kind(FileKind::Bmp);
        }
        if [b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"]
            .iter()
            .any(|sig| head.starts_with(*sig))
        {
            return kind(FileKind::Tiff);
        }
        if head.starts_with(&[0, 0, 1, 0]) && head.get(4..6).is_some_and(|n| n != [0, 0]) {
            return kind(FileKind::Ico);
        }
        if let Some(kind) = sniff_iso_bmff(head) {
            return Some(Sniffed::Kind(kind));
        }
        // PDF readers accept the header anywhere in the first kilobyte.
        if head.windows(5).any(|w| w == b"%PDF-") {
            return kind(FileKind::Pdf);
        }
        if head.starts_with(b"PK\x03\x04") {
            return Some(Sniffed::Zip);
        }
        None
    }

    pub fn is_image(self) -> bool {
        matches!(
            self,
            FileKind::Jpeg
                | FileKind::Png
                | FileKind::Webp
                | FileKind::Bmp
                | FileKind::Tiff
                | FileKind::Gif
                | FileKind::Ico
                | FileKind::Avif
                | FileKind::Heic
        )
    }

    /// The extension used when writing this kind, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            FileKind::Jpeg => "jpg",
            FileKind::Png => "png",
            FileKind::Webp => "webp",
            FileKind::Bmp => "bmp",
            FileKind::Tiff => "tiff",
            FileKind::Gif => "gif",
            FileKind::Ico => "ico",
            FileKind::Avif => "avif",
            FileKind::Heic => "heic",
            FileKind::Pdf => "pdf",
            FileKind::Docx => "docx",
            FileKind::Txt => "txt",
            FileKind::Unknown => "bin",
        }
    }

    /// Short upper-case name shown in the UI.
    pub fn label(self) -> &'static str {
        match self {
            FileKind::Jpeg => "JPG",
            FileKind::Png => "PNG",
            FileKind::Webp => "WEBP",
            FileKind::Bmp => "BMP",
            FileKind::Tiff => "TIFF",
            FileKind::Gif => "GIF",
            FileKind::Ico => "ICO",
            FileKind::Avif => "AVIF",
            FileKind::Heic => "HEIC",
            FileKind::Pdf => "PDF",
            FileKind::Docx => "DOCX",
            FileKind::Txt => "TXT",
            FileKind::Unknown => "unknown",
        }
    }
}

enum Sniffed {
    Kind(FileKind),
    /// A zip container; only the extension can tell a DOCX from any other zip.
    Zip,
}

/// AVIF and HEIC are both ISO-BMFF files told apart by the brands in their `ftyp` box.
fn sniff_iso_bmff(head: &[u8]) -> Option<FileKind> {
    if head.get(4..8)? != b"ftyp" {
        return None;
    }
    let size = u32::from_be_bytes(head.get(0..4)?.try_into().ok()?) as usize;
    let ftyp = head.get(8..size.min(head.len()))?;
    let major = ftyp.get(0..4)?;
    // Skip the 4-byte minor version to reach the compatible brands.
    let compatible = ftyp.get(8..).unwrap_or_default().chunks_exact(4);
    let brands: Vec<&[u8]> = std::iter::once(major).chain(compatible).collect();

    let has = |names: &[&[u8; 4]]| brands.iter().any(|b| names.iter().any(|n| b == n));
    if has(&[b"avif", b"avis"]) {
        Some(FileKind::Avif)
    } else if has(&[
        b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx", b"mif1", b"msf1",
    ]) {
        Some(FileKind::Heic)
    } else {
        None
    }
}

fn is_text_extension(ext: &str) -> bool {
    matches!(ext, "txt" | "text" | "log" | "md")
}

/// Rejects binary content saved with a `.txt` name. UTF-16 text has NULs, so allow it
/// when it carries a byte-order mark.
fn looks_like_text(head: &[u8]) -> bool {
    head.starts_with(&[0xFF, 0xFE]) || head.starts_with(&[0xFE, 0xFF]) || !head.contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ftyp(major: &[u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let size = 16 + 4 * compatible.len();
        let mut v = (size as u32).to_be_bytes().to_vec();
        v.extend_from_slice(b"ftyp");
        v.extend_from_slice(major);
        v.extend_from_slice(&[0, 0, 0, 0]);
        for b in compatible {
            v.extend_from_slice(*b);
        }
        v
    }

    #[test]
    fn sniffs_signatures() {
        let cases: &[(&[u8], FileKind)] = &[
            (&[0xFF, 0xD8, 0xFF, 0xE0], FileKind::Jpeg),
            (b"\x89PNG\r\n\x1a\n\0\0", FileKind::Png),
            (b"GIF89a..", FileKind::Gif),
            (b"RIFF\x24\0\0\0WEBPVP8 ", FileKind::Webp),
            (b"BM\x36\0\0\0\0\0\0\0\x36\0", FileKind::Bmp),
            (b"II*\0\x08\0\0\0", FileKind::Tiff),
            (b"MM\0*\0\0\0\x08", FileKind::Tiff),
            (&[0, 0, 1, 0, 1, 0, 16, 16], FileKind::Ico),
            (b"%PDF-1.7\n", FileKind::Pdf),
            (b"\r\n  junk %PDF-1.4", FileKind::Pdf),
        ];
        for (head, want) in cases {
            assert_eq!(FileKind::classify(head, "bin"), *want, "{head:?}");
        }
    }

    #[test]
    fn content_beats_extension() {
        assert_eq!(
            FileKind::classify(b"\x89PNG\r\n\x1a\n", "jpg"),
            FileKind::Png
        );
        assert_eq!(
            FileKind::classify(b"not a jpeg at all", "jpg"),
            FileKind::Unknown
        );
    }

    #[test]
    fn tells_avif_from_heic() {
        assert_eq!(
            FileKind::classify(&ftyp(b"avif", &[b"mif1"]), ""),
            FileKind::Avif
        );
        assert_eq!(
            FileKind::classify(&ftyp(b"mif1", &[b"avif"]), ""),
            FileKind::Avif
        );
        assert_eq!(
            FileKind::classify(&ftyp(b"heic", &[b"mif1"]), ""),
            FileKind::Heic
        );
        assert_eq!(
            FileKind::classify(&ftyp(b"mif1", &[b"heic"]), ""),
            FileKind::Heic
        );
        // An MP4 is ISO-BMFF too, but not an image.
        assert_eq!(
            FileKind::classify(&ftyp(b"isom", &[b"mp41"]), ""),
            FileKind::Unknown
        );
    }

    #[test]
    fn zip_needs_docx_extension() {
        assert_eq!(FileKind::classify(b"PK\x03\x04", "DOCX"), FileKind::Docx);
        assert_eq!(FileKind::classify(b"PK\x03\x04", "zip"), FileKind::Unknown);
    }

    #[test]
    fn text_by_extension() {
        assert_eq!(FileKind::classify("مرحبا".as_bytes(), "txt"), FileKind::Txt);
        assert_eq!(FileKind::classify(b"", "txt"), FileKind::Txt);
        assert_eq!(
            FileKind::classify(&[0xFF, 0xFE, b'h', 0], "txt"),
            FileKind::Txt
        );
        assert_eq!(FileKind::classify(&[1, 0, 2, 0], "txt"), FileKind::Unknown);
        assert_eq!(FileKind::classify(b"hello", "exe"), FileKind::Unknown);
    }

    #[test]
    fn truncated_headers_do_not_panic() {
        for len in 0..=16 {
            let head = &ftyp(b"avif", &[])[..len.min(16)];
            let _ = FileKind::classify(head, "");
            let _ = FileKind::classify(&b"RIFF\0\0\0\0WEB"[..len.min(11)], "");
        }
        // An ftyp box claiming to be huge, or smaller than its own header.
        let mut huge = ftyp(b"avif", &[]);
        huge[..4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(FileKind::classify(&huge, ""), FileKind::Avif);
        let mut tiny = ftyp(b"avif", &[]);
        tiny[..4].copy_from_slice(&3u32.to_be_bytes());
        assert_eq!(FileKind::classify(&tiny, ""), FileKind::Unknown);
    }

    #[test]
    fn detects_files_with_unicode_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("صورة العائلة.PNG");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\n").unwrap();
        assert_eq!(FileKind::detect(&path).unwrap(), FileKind::Png);
    }

    #[test]
    fn detect_missing_file_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let err = FileKind::detect(&dir.path().join("missing.png")).unwrap_err();
        assert_eq!(err.code(), "not_found");
    }
}
