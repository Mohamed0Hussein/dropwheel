//! Putting a solid colour behind a transparent image.

use std::path::{Path, PathBuf};

use image::DynamicImage;

use super::support;
use crate::action::{Action, ActionGroup, Options, Params};
use crate::error::{Error, Result};
use crate::imaging;
use crate::kind::FileKind::{self, *};

pub struct AddBackground;

pub static ADD_BACKGROUND: AddBackground = AddBackground;

impl Action for AddBackground {
    fn id(&self) -> &'static str {
        "add_background"
    }

    fn label(&self) -> &'static str {
        "ADD BACKGROUND"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    /// Only formats that can carry transparency.
    fn applicable(&self, kind: FileKind) -> bool {
        matches!(kind, Png | Webp | Avif | Tiff | Gif)
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        let Params::Background(background) = opts.params else {
            return Err(Error::InvalidOptions(
                "add background needs a colour".into(),
            ));
        };
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Add background",
                kind,
            });
        }
        let (kind, decoded) = support::load(input, opts)?;
        let filled = DynamicImage::ImageRgb8(imaging::flatten(&decoded.image, background.color));
        support::save_like(
            input,
            out,
            "background",
            kind,
            &filled,
            decoded.icc.as_deref(),
            opts,
        )
    }
}
