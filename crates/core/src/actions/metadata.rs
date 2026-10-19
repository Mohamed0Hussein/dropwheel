//! Showing and removing the metadata embedded in a file.

use std::fs;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

use img_parts::jpeg::{Jpeg, markers};
use img_parts::png::Png;
use img_parts::riff::{RiffChunk, RiffContent};
use img_parts::webp::{CHUNK_EXIF, CHUNK_VP8X, CHUNK_XMP, WebP};
use img_parts::{Bytes, ImageEXIF};
use serde::Serialize;

use super::support;
use crate::action::{Action, ActionGroup, Options, Params};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};
use crate::output;

pub struct Metadata;

pub static METADATA: Metadata = Metadata;

/// One row of the Metadata dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Field {
    pub group: String,
    pub name: String,
    pub value: String,
}

impl Field {
    fn new(group: &str, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut value: String = value.into();
        if value.chars().count() > MAX_VALUE_CHARS {
            value = value.chars().take(MAX_VALUE_CHARS).collect::<String>() + "…";
        }
        Field {
            group: group.to_string(),
            name: name.into(),
            value,
        }
    }
}

const MAX_VALUE_CHARS: usize = 300;
/// A hostile file could hold millions of entries; nobody reads past this.
const MAX_FIELDS: usize = 2000;

/// The metadata of the image at `path`, for display. Documents are read in phase 5.
pub fn read(path: &Path) -> Result<Vec<Field>> {
    let kind = FileKind::detect(path)?;
    support::require_image(kind, "Reading metadata of")?;
    let data = fs::read(path).map_err(|e| Error::io(path, e))?;
    let mut fields = exif_fields(&data);
    match kind {
        Png => fields.extend(png_text(&data)),
        Jpeg => fields.extend(jpeg_extras(&data)),
        Webp => fields.extend(webp_extras(&data)),
        _ => {}
    }
    fields.truncate(MAX_FIELDS);
    Ok(fields)
}

fn exif_fields(data: &[u8]) -> Vec<Field> {
    let Ok(exif) = exif::Reader::new().read_from_container(&mut BufReader::new(Cursor::new(data)))
    else {
        return Vec::new();
    };
    exif.fields()
        .filter(|f| f.ifd_num == exif::In::PRIMARY)
        .filter(|f| !(f.tag.context() == exif::Context::Tiff && is_structural(f.tag.number())))
        .map(|f| {
            let group = match f.tag.context() {
                exif::Context::Tiff => "Image",
                exif::Context::Exif => "Camera",
                exif::Context::Gps => "Location",
                exif::Context::Interop => "Interoperability",
                _ => "Other",
            };
            let value = f.display_value().with_unit(&exif).to_string();
            Field::new(
                group,
                f.tag.to_string(),
                value.trim_matches('"').to_string(),
            )
        })
        .collect()
}

/// TIFF tags that describe how the pixels are stored rather than anything about the
/// picture or its owner: every TIFF has them, and they are not shown as metadata.
fn is_structural(tag: u16) -> bool {
    matches!(
        tag,
        256..=259 // ImageWidth, ImageLength, BitsPerSample, Compression
            | 262 // PhotometricInterpretation
            | 273 // StripOffsets
            | 277..=279 // SamplesPerPixel, RowsPerStrip, StripByteCounts
            | 282..=284 // XResolution, YResolution, PlanarConfiguration
            | 296 // ResolutionUnit
            | 301 // TransferFunction
            | 317..=325 // Predictor, WhitePoint, PrimaryChromaticities, ColorMap, tiles
            | 338 | 339 // ExtraSamples, SampleFormat
            | 513 | 514 // JPEG thumbnail offset and length
            | 529..=532 // YCbCr coefficients, subsampling, positioning, reference
    )
}

fn png_text(data: &[u8]) -> Vec<Field> {
    let decoder = png::Decoder::new(Cursor::new(data));
    let Ok(reader) = decoder.read_info() else {
        return Vec::new();
    };
    let info = reader.info();
    let mut fields = Vec::new();
    for t in &info.uncompressed_latin1_text {
        fields.push(Field::new("Text", t.keyword.clone(), t.text.clone()));
    }
    for t in &info.compressed_latin1_text {
        let text = t.get_text().unwrap_or_else(|_| "(unreadable)".into());
        fields.push(Field::new("Text", t.keyword.clone(), text));
    }
    for t in &info.utf8_text {
        let text = t.get_text().unwrap_or_else(|_| "(unreadable)".into());
        fields.push(Field::new("Text", t.keyword.clone(), text));
    }
    fields
}

fn jpeg_extras(data: &[u8]) -> Vec<Field> {
    let Ok(jpeg) = Jpeg::from_bytes(Bytes::copy_from_slice(data)) else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    for seg in jpeg.segments() {
        match seg.marker() {
            markers::COM => fields.push(Field::new(
                "Comment",
                "Comment",
                String::from_utf8_lossy(seg.contents()).into_owned(),
            )),
            markers::APP1 if seg.contents().starts_with(XMP_SIGNATURE) => {
                fields.push(xmp_field(seg.contents().len()));
            }
            markers::APP13 => fields.push(Field::new(
                "IPTC",
                "Photoshop / IPTC block",
                format!("{} bytes", seg.contents().len()),
            )),
            _ => {}
        }
    }
    fields
}

fn webp_extras(data: &[u8]) -> Vec<Field> {
    let Ok(webp) = WebP::from_bytes(Bytes::copy_from_slice(data)) else {
        return Vec::new();
    };
    webp.chunk_by_id(CHUNK_XMP)
        .map(|c| vec![xmp_field(c.content().len() as usize)])
        .unwrap_or_default()
}

const XMP_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

fn xmp_field(len: usize) -> Field {
    Field::new("XMP", "XMP packet", format!("{len} bytes"))
}

/// JPEG segments that only carry metadata. APP0 (JFIF), APP2's colour profile and APP14
/// (Adobe colour transform) are needed to show the picture correctly and stay.
fn is_metadata_segment(marker: u8, contents: &[u8]) -> bool {
    match marker {
        markers::APP2 => !contents.starts_with(b"ICC_PROFILE\0"),
        markers::APP0 | markers::APP14 => false,
        m if (markers::APP1..=markers::APP15).contains(&m) => true,
        markers::COM => true,
        _ => false,
    }
}

/// PNG chunks that only carry metadata. Colour information (iCCP, sRGB, gAMA, cHRM)
/// stays.
const PNG_METADATA_CHUNKS: [[u8; 4]; 5] = [*b"tEXt", *b"zTXt", *b"iTXt", *b"eXIf", *b"tIME"];

/// Removes metadata without re-encoding the pixels. Data after a JPEG's end marker
/// (extra pictures from phones, vendor trailers) goes too; the MPF segment that pointed
/// at it is removed with the other APP segments.
fn strip_in_place(kind: FileKind, data: Vec<u8>) -> std::result::Result<Vec<u8>, String> {
    let bytes = Bytes::from(data);
    let out = match kind {
        Jpeg => {
            let mut jpeg = Jpeg::from_bytes(bytes).map_err(|e| e.to_string())?;
            jpeg.segments_mut()
                .retain(|s| !is_metadata_segment(s.marker(), s.contents()));
            jpeg.encoder().bytes()
        }
        Png => {
            let mut png = Png::from_bytes(bytes).map_err(|e| e.to_string())?;
            png.chunks_mut()
                .retain(|c| !PNG_METADATA_CHUNKS.contains(&c.kind()));
            png.encoder().bytes()
        }
        Webp => {
            let mut webp = WebP::from_bytes(bytes).map_err(|e| e.to_string())?;
            webp.set_exif(None);
            webp.remove_chunks_by_id(CHUNK_EXIF);
            webp.remove_chunks_by_id(CHUNK_XMP);
            clear_vp8x_metadata_flags(&mut webp);
            webp.encoder().bytes()
        }
        _ => return Err("not handled here".into()),
    };
    Ok(out.to_vec())
}

/// The VP8X header announces EXIF and XMP chunks. img-parts refreshes only the EXIF bit,
/// and decoders reject a file whose header promises a chunk that is not there.
fn clear_vp8x_metadata_flags(webp: &mut WebP) {
    const EXIF_FLAG: u8 = 0b0000_1000;
    const XMP_FLAG: u8 = 0b0000_0100;
    for chunk in webp.chunks_mut().iter_mut() {
        if chunk.id() != CHUNK_VP8X {
            continue;
        }
        if let Some(data) = chunk.content().data() {
            let mut header = data.to_vec();
            if let Some(flags) = header.first_mut() {
                *flags &= !(EXIF_FLAG | XMP_FLAG);
            }
            *chunk = RiffChunk::new(CHUNK_VP8X, RiffContent::Data(Bytes::from(header)));
        }
    }
}

impl Action for Metadata {
    fn id(&self) -> &'static str {
        "metadata"
    }

    fn label(&self) -> &'static str {
        "METADATA"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        matches!(kind, Jpeg | Png | Webp | Tiff | Avif | Heic | Pdf | Docx)
    }

    /// Running the action removes all metadata; reading it is [`read`].
    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        if opts.params != Params::None {
            return Err(Error::InvalidOptions(
                "removing metadata takes no settings".into(),
            ));
        }
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Metadata",
                kind,
            });
        }
        support::require_image(kind, "Removing metadata from")?;
        opts.check_cancelled()?;
        if matches!(kind, Jpeg | Png | Webp) {
            let data = fs::read(input).map_err(|e| Error::io(input, e))?;
            let clean = strip_in_place(kind, data).map_err(|reason| Error::Decode {
                path: input.to_path_buf(),
                kind,
                reason,
            })?;
            opts.report(0.8);
            return output::write_bytes(input, out, "clean", kind.extension(), &clean);
        }
        // TIFF tags, and HEIF/AVIF boxes, are rewritten by encoding the pixels afresh.
        let (kind, decoded) = support::load(input, opts)?;
        support::save_like(
            input,
            out,
            "clean",
            kind,
            &decoded.image,
            decoded.icc.as_deref(),
            opts,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_what_the_picture_needs() {
        assert!(!is_metadata_segment(markers::APP0, b"JFIF\0"));
        assert!(!is_metadata_segment(
            markers::APP2,
            b"ICC_PROFILE\0\x01\x01"
        ));
        assert!(!is_metadata_segment(markers::APP14, b"Adobe"));
        assert!(is_metadata_segment(markers::APP1, b"Exif\0\0"));
        assert!(is_metadata_segment(markers::APP2, b"MPF\0"));
        assert!(is_metadata_segment(markers::APP13, b"Photoshop 3.0\0"));
        assert!(is_metadata_segment(markers::COM, b"hello"));
        assert!(!is_metadata_segment(markers::DQT, b""));
    }

    #[test]
    fn long_values_are_cut() {
        let f = Field::new("Text", "k", "x".repeat(1000));
        assert_eq!(f.value.chars().count(), MAX_VALUE_CHARS + 1);
    }

    #[test]
    fn garbage_reads_as_no_metadata() {
        assert!(exif_fields(b"\xff\xd8\xff\xe1\x00\x10Exif\0\0garbage").is_empty());
        assert!(png_text(b"\x89PNG\r\n\x1a\nnope").is_empty());
        assert!(jpeg_extras(b"\xff\xd8").is_empty());
        assert!(strip_in_place(Jpeg, b"\xff\xd8\xff".to_vec()).is_err());
    }
}
