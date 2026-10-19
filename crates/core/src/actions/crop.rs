//! Cutting an image down to a rectangle.

use std::path::{Path, PathBuf};

use super::support;
use crate::action::{Action, ActionGroup, Crop, Options, Params};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};

pub struct CropImage;

pub static CROP: CropImage = CropImage;

/// Fits `crop` to an image of `width` × `height`: scaled when that differs from the size
/// the rectangle was drawn on, then clamped inside. Returns x, y, width, height.
pub fn fit(crop: &Crop, width: u32, height: u32) -> Result<(u32, u32, u32, u32)> {
    if crop.width == 0 || crop.height == 0 || crop.ref_width == 0 || crop.ref_height == 0 {
        return Err(Error::InvalidOptions("the crop rectangle is empty".into()));
    }
    let scale = |v: u32, to: u32, from: u32| {
        if to == from {
            v
        } else {
            ((u64::from(v) * u64::from(to) + u64::from(from) / 2) / u64::from(from)) as u32
        }
    };
    let x = scale(crop.x, width, crop.ref_width).min(width.saturating_sub(1));
    let y = scale(crop.y, height, crop.ref_height).min(height.saturating_sub(1));
    let w = scale(crop.width, width, crop.ref_width).clamp(1, width - x);
    let h = scale(crop.height, height, crop.ref_height).clamp(1, height - y);
    Ok((x, y, w, h))
}

impl Action for CropImage {
    fn id(&self) -> &'static str {
        "crop"
    }

    fn label(&self) -> &'static str {
        "CROP"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        matches!(
            kind,
            Jpeg | Png | Webp | Bmp | Tiff | Gif | Avif | Heic | Pdf
        )
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        let Params::Crop(crop) = opts.params else {
            return Err(Error::InvalidOptions("crop needs a rectangle".into()));
        };
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Crop",
                kind,
            });
        }
        if kind == Pdf {
            let bytes = crate::pdf::edit::crop(input, &crop, opts)?;
            return crate::output::write_bytes(input, out, "cropped", "pdf", &bytes);
        }
        let (kind, decoded) = support::load(input, opts)?;
        let (x, y, w, h) = fit(&crop, decoded.image.width(), decoded.image.height())?;
        let cropped = decoded.image.crop_imm(x, y, w, h);
        support::save_like(
            input,
            out,
            "cropped",
            kind,
            &cropped,
            decoded.icc.as_deref(),
            opts,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crop(x: u32, y: u32, width: u32, height: u32, ref_width: u32, ref_height: u32) -> Crop {
        Crop {
            x,
            y,
            width,
            height,
            ref_width,
            ref_height,
        }
    }

    #[test]
    fn exact_when_sizes_match() {
        assert_eq!(
            fit(&crop(10, 20, 30, 40, 100, 100), 100, 100).unwrap(),
            (10, 20, 30, 40)
        );
    }

    #[test]
    fn scales_to_other_images_in_a_batch() {
        // Drawn on a 400x300 image, applied to one twice the size.
        assert_eq!(
            fit(&crop(100, 75, 200, 150, 400, 300), 800, 600).unwrap(),
            (200, 150, 400, 300)
        );
    }

    #[test]
    fn clamps_inside_the_image() {
        assert_eq!(
            fit(&crop(90, 90, 50, 50, 100, 100), 100, 100).unwrap(),
            (90, 90, 10, 10)
        );
        assert_eq!(
            fit(&crop(500, 500, 50, 50, 100, 100), 100, 100).unwrap(),
            (99, 99, 1, 1)
        );
    }

    #[test]
    fn rejects_empty_rectangles() {
        assert!(fit(&crop(0, 0, 0, 10, 100, 100), 100, 100).is_err());
        assert!(fit(&crop(0, 0, 10, 10, 0, 100), 100, 100).is_err());
    }
}
