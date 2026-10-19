use std::path::{Path, PathBuf};

use crate::action::{Action, ActionGroup, Options};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};

/// A Tools wedge whose implementation has not landed yet: it carries the tool's identity
/// and the file kinds it will accept.
pub struct Tool {
    id: &'static str,
    label: &'static str,
    /// For messages, as in "Compress is not available yet."
    name: &'static str,
    kinds: &'static [FileKind],
}

/// Images whose pixels can be edited and written back.
const EDITABLE_IMAGES: &[FileKind] = &[Jpeg, Png, Webp, Bmp, Tiff, Gif, Avif, Heic];

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
