use std::path::{Path, PathBuf};

use super::support;
use crate::action::{Action, ActionGroup, Options, Params};
use crate::error::{Error, Result};
use crate::imaging::{self, EncodeOptions};
use crate::kind::FileKind::{self, *};

/// Converts a file to another format.
pub struct Convert {
    id: &'static str,
    target: FileKind,
}

impl Convert {
    const fn new(id: &'static str, target: FileKind) -> Self {
        Convert { id, target }
    }

    pub fn target(&self) -> FileKind {
        self.target
    }
}

pub static DOCX: Convert = Convert::new("convert.docx", Docx);
pub static PDF: Convert = Convert::new("convert.pdf", Pdf);
pub static TXT: Convert = Convert::new("convert.txt", Txt);
pub static JPG: Convert = Convert::new("convert.jpg", Jpeg);
pub static PNG: Convert = Convert::new("convert.png", Png);
pub static WEBP: Convert = Convert::new("convert.webp", Webp);
pub static BMP: Convert = Convert::new("convert.bmp", Bmp);
pub static TIFF: Convert = Convert::new("convert.tiff", Tiff);
pub static AVIF: Convert = Convert::new("convert.avif", Avif);
pub static HEIC: Convert = Convert::new("convert.heic", Heic);

/// The formats a file of kind `source` can sensibly be converted to.
///
/// Images go to other image formats or a PDF page. A PDF gives its text or page images.
/// DOCX and TXT go to each other and to PDF. Image to text would need OCR, which is out of
/// scope.
pub fn targets(source: FileKind) -> &'static [FileKind] {
    match source {
        Jpeg | Png | Webp | Bmp | Tiff | Gif | Ico | Avif | Heic => {
            &[Pdf, Jpeg, Png, Webp, Bmp, Tiff, Avif, Heic]
        }
        Pdf => &[Docx, Txt, Jpeg, Png, Webp, Bmp, Tiff],
        Docx => &[Pdf, Txt],
        Txt => &[Docx, Pdf],
        Unknown => &[],
    }
}

impl Action for Convert {
    fn id(&self) -> &'static str {
        self.id
    }

    fn label(&self) -> &'static str {
        self.target.label()
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Convert
    }

    fn applicable(&self, kind: FileKind) -> bool {
        kind != self.target
            && targets(kind).contains(&self.target)
            // Writing HEIC needs Microsoft's HEVC encoder, which may not be installed.
            && (self.target != Heic || imaging::can_encode(Heic))
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        if opts.params != Params::None {
            return Err(Error::InvalidOptions("conversions take no settings".into()));
        }
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "This conversion",
                kind,
            });
        }
        if !kind.is_image() || !self.target.is_image() {
            return Err(Error::NotAvailable {
                action: format!("Converting {} to {}", kind.label(), self.label()),
            });
        }
        let (_, decoded) = support::load(input, opts)?;
        let encode = EncodeOptions {
            quality: if self.target == Avif { 80 } else { 90 },
            // Photos stay lossy in WebP; graphics stay pixel-exact.
            lossless: !matches!(kind, Jpeg | Webp | Avif | Heic),
            palette: None,
            icc: decoded.icc.as_deref(),
        };
        support::save(
            input,
            out,
            "converted",
            &decoded.image,
            self.target,
            &encode,
            opts,
        )
    }
}
