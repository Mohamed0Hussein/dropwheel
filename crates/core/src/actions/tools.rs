use std::path::{Path, PathBuf};

use crate::action::{Action, ActionGroup, Options};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};

/// One of the Tools wedges. Until each tool has its own implementation, this table-driven
/// placeholder carries its identity and the file kinds it will accept.
pub struct Tool {
    id: &'static str,
    label: &'static str,
    /// For messages, as in "Compress is not available yet."
    name: &'static str,
    kinds: &'static [FileKind],
}

/// Images whose pixels can be edited and written back.
const EDITABLE_IMAGES: &[FileKind] = &[Jpeg, Png, Webp, Bmp, Tiff, Gif, Avif, Heic];

pub static COMPRESS: Tool = Tool {
    id: "compress",
    label: "COMPRESS",
    name: "Compress",
    kinds: &[Jpeg, Png, Webp, Avif, Tiff, Pdf],
};

pub static CROP: Tool = Tool {
    id: "crop",
    label: "CROP",
    name: "Crop",
    kinds: &[Jpeg, Png, Webp, Bmp, Tiff, Gif, Avif, Heic, Pdf],
};

pub static EDIT: Tool = Tool {
    id: "edit",
    label: "EDIT",
    name: "Edit",
    kinds: EDITABLE_IMAGES,
};

pub static ANNOTATE: Tool = Tool {
    id: "annotate",
    label: "ANNOTATE",
    name: "Annotate",
    kinds: EDITABLE_IMAGES,
};

pub static REDACT: Tool = Tool {
    id: "redact",
    label: "REDACT",
    name: "Redact",
    kinds: &[Jpeg, Png, Webp, Bmp, Tiff, Gif, Avif, Heic, Pdf],
};

pub static METADATA: Tool = Tool {
    id: "metadata",
    label: "METADATA",
    name: "Metadata",
    kinds: &[Jpeg, Png, Webp, Tiff, Avif, Heic, Pdf, Docx],
};

/// Only formats that can carry transparency.
pub static ADD_BACKGROUND: Tool = Tool {
    id: "add_background",
    label: "ADD BACKGROUND",
    name: "Add background",
    kinds: &[Png, Webp, Avif, Tiff, Gif],
};

impl Action for Tool {
    fn id(&self) -> &'static str {
        self.id
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        self.kinds.contains(&kind)
    }

    fn run(&self, _input: &Path, _opts: &Options, _out: &Path) -> Result<PathBuf> {
        Err(Error::NotAvailable {
            action: self.name.to_string(),
        })
    }
}
