use std::path::{Path, PathBuf};

use crate::action::{Action, ActionGroup, Options};
use crate::error::{Error, Result};
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
        kind != self.target && targets(kind).contains(&self.target)
    }

    fn run(&self, _input: &Path, _opts: &Options, _out: &Path) -> Result<PathBuf> {
        Err(Error::NotAvailable {
            action: format!("Converting to {}", self.label()),
        })
    }
}
