//! Plain text files, in whatever encoding they were saved.

use std::io::Read;
use std::path::Path;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, UTF_16BE, UTF_16LE};

use crate::doc::{Block, Document, Paragraph, ParagraphStyle};
use crate::error::{Error, Result};
use crate::kind::FileKind;

/// Larger text files would make thousands of pages; nobody converts those by dropping
/// them on a shelf.
pub const MAX_BYTES: u64 = 32 * 1024 * 1024;

/// Decodes text: by its byte order mark, as UTF-16 when every other byte is zero, as UTF-8
/// when it is valid, and otherwise in the legacy encoding it most likely uses, such as
/// Windows-1252 or Windows-1256 for Arabic.
pub fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, bom)) = Encoding::for_bom(bytes) {
        return encoding
            .decode_without_bom_handling(&bytes[bom..])
            .0
            .into_owned();
    }
    // Before UTF-8: ASCII in UTF-16 is also valid UTF-8, full of NUL characters.
    if let Some(encoding) = bomless_utf16(bytes) {
        return encoding.decode_without_bom_handling(bytes).0.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    detector
        .guess(None, Utf8Detection::Allow)
        .decode_without_bom_handling(bytes)
        .0
        .into_owned()
}

fn bomless_utf16(bytes: &[u8]) -> Option<&'static Encoding> {
    if bytes.len() < 4 || !bytes.len().is_multiple_of(2) {
        return None;
    }
    let pairs = bytes.len() / 2;
    let zeros_at = |offset: usize| {
        bytes
            .iter()
            .skip(offset)
            .step_by(2)
            .filter(|b| **b == 0)
            .count()
    };
    // Mostly ASCII text in UTF-16 has a zero in every other byte.
    if zeros_at(1) * 10 >= pairs * 7 && zeros_at(0) == 0 {
        Some(UTF_16LE)
    } else if zeros_at(0) * 10 >= pairs * 7 && zeros_at(1) == 0 {
        Some(UTF_16BE)
    } else {
        None
    }
}

/// Reads a text file at `path`, refusing files over [`MAX_BYTES`].
pub fn read_text(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::io(path, e))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Error::Decode {
            path: path.to_path_buf(),
            kind: FileKind::Txt,
            reason: format!("text files over {} MB are not converted", MAX_BYTES >> 20),
        });
    }
    Ok(decode(&bytes))
}

/// A text file as a document: one paragraph per line, blank lines kept.
pub fn to_document(text: &str) -> Document {
    let style = ParagraphStyle {
        space_after: 0.0,
        ..ParagraphStyle::default()
    };
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.strip_suffix('\n').unwrap_or(&text);
    let blocks = text
        .split('\n')
        .map(|line| {
            // A form feed is a page break in plain text.
            if line == "\u{c}" {
                Block::PageBreak
            } else {
                Block::Paragraph(Paragraph {
                    style: style.clone(),
                    ..Paragraph::plain(line.replace('\u{c}', ""))
                })
            }
        })
        .collect();
    Document {
        blocks,
        ..Document::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodings() {
        assert_eq!(decode("héllo".as_bytes()), "héllo");
        assert_eq!(decode(b"\xEF\xBB\xBFhi"), "hi");
        let utf16: Vec<u8> = "\u{FEFF}سلام"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(decode(&utf16), "سلام");
        let bomless: Vec<u8> = "plain text"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(decode(&bomless), "plain text");
        let be: Vec<u8> = "plain text"
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect();
        assert_eq!(decode(&be), "plain text");
        // "café" in Windows-1252.
        assert_eq!(
            decode(b"caf\xe9 au lait, s'il vous pla\xeet"),
            "café au lait, s'il vous plaît"
        );
        // Arabic in Windows-1256.
        let (arabic, _, _) = encoding_rs::WINDOWS_1256.encode("مرحبا بالعالم، هذا نص عربي قديم");
        assert_eq!(decode(&arabic), "مرحبا بالعالم، هذا نص عربي قديم");
    }

    #[test]
    fn lines_become_paragraphs() {
        let doc = to_document("one\r\n\r\ntwo\n\u{c}\nthree\n");
        let texts: Vec<String> = doc
            .blocks
            .iter()
            .map(|b| match b {
                Block::Paragraph(p) => p.text(),
                Block::PageBreak => "<page>".into(),
                _ => "?".into(),
            })
            .collect();
        assert_eq!(texts, ["one", "", "two", "<page>", "three"]);
    }
}
