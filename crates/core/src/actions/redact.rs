//! Blacking out parts of an image for good.
//!
//! The boxes replace the pixels under them, and the result is encoded afresh from those
//! pixels, so neither the original picture nor its embedded thumbnail and metadata
//! survive.

use std::path::{Path, PathBuf};

use image::DynamicImage;

use super::support;
use crate::action::{Action, ActionGroup, Options, Params, Redact, Region};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};

pub struct RedactFile;

pub static REDACT: RedactFile = RedactFile;

const MAX_REGIONS: usize = 10_000;

pub fn validate(redact: &Redact) -> Result<()> {
    if redact.regions.is_empty() {
        return Err(Error::InvalidOptions("there is nothing to redact".into()));
    }
    if redact.regions.len() > MAX_REGIONS {
        return Err(Error::InvalidOptions("there are too many boxes".into()));
    }
    let ok = |r: &Region| {
        [r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite())
            && r.width > 0.0
            && r.height > 0.0
    };
    if !redact.regions.iter().all(ok) {
        return Err(Error::InvalidOptions("a box is out of range".into()));
    }
    Ok(())
}

/// The pixels `region` covers in an image of `width` × `height`, rounded outwards so
/// partly covered pixels are blacked out too: x, y, width, height. `None` when the
/// region lies outside the image.
pub fn pixels(region: &Region, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let span = |start: f32, len: f32, size: u32| {
        let size_f = size as f32;
        let a = (start.min(start + len) * size_f).floor().clamp(0.0, size_f) as u32;
        let b = (start.max(start + len) * size_f).ceil().clamp(0.0, size_f) as u32;
        (b > a).then_some((a, b - a))
    };
    let (x, w) = span(region.x, region.width, width)?;
    let (y, h) = span(region.y, region.height, height)?;
    Some((x, y, w, h))
}

/// Fills the regions of `page` with opaque black.
pub fn black_out(image: DynamicImage, regions: &[Region], page: u32) -> DynamicImage {
    let (width, height) = (image.width(), image.height());
    let boxes: Vec<_> = regions
        .iter()
        .filter(|r| r.page == page)
        .filter_map(|r| pixels(r, width, height))
        .collect();
    if image.color().has_alpha() {
        let mut rgba = image.into_rgba8();
        for (x, y, w, h) in boxes {
            for yy in y..y + h {
                for xx in x..x + w {
                    rgba.put_pixel(xx, yy, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
        DynamicImage::ImageRgba8(rgba)
    } else {
        let mut rgb = image.into_rgb8();
        for (x, y, w, h) in boxes {
            for yy in y..y + h {
                for xx in x..x + w {
                    rgb.put_pixel(xx, yy, image::Rgb([0, 0, 0]));
                }
            }
        }
        DynamicImage::ImageRgb8(rgb)
    }
}

impl Action for RedactFile {
    fn id(&self) -> &'static str {
        "redact"
    }

    fn label(&self) -> &'static str {
        "REDACT"
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
        let Params::Redact(redact) = &opts.params else {
            return Err(Error::InvalidOptions("redact needs boxes to fill".into()));
        };
        validate(redact)?;
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Redact",
                kind,
            });
        }
        support::require_image(kind, "Redacting")?;
        let (kind, decoded) = support::load(input, opts)?;
        let redacted = black_out(decoded.image, &redact.regions, 0);
        opts.report(0.7);
        support::save_like(
            input,
            out,
            "redacted",
            kind,
            &redacted,
            decoded.icc.as_deref(),
            opts,
        )
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    use super::*;

    fn region(x: f32, y: f32, width: f32, height: f32) -> Region {
        Region {
            page: 0,
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn regions_round_outwards() {
        assert_eq!(
            pixels(&region(0.0, 0.0, 0.5, 0.5), 10, 10),
            Some((0, 0, 5, 5))
        );
        assert_eq!(
            pixels(&region(0.11, 0.11, 0.1, 0.1), 10, 10),
            Some((1, 1, 2, 2))
        );
        assert_eq!(
            pixels(&region(0.9, 0.9, 0.5, 0.5), 10, 10),
            Some((9, 9, 1, 1))
        );
        assert_eq!(pixels(&region(1.5, 0.0, 0.5, 0.5), 10, 10), None);
        // Drawn from the bottom right.
        assert_eq!(
            pixels(&region(0.5, 0.5, -0.5, -0.5), 10, 10),
            Some((0, 0, 5, 5))
        );
    }

    #[test]
    fn boxes_are_solid_black() {
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(10, 10, Rgb([200, 10, 10])));
        let out = black_out(img, &[region(0.2, 0.2, 0.3, 0.3)], 0).to_rgb8();
        assert_eq!(out.get_pixel(2, 2).0, [0, 0, 0]);
        assert_eq!(out.get_pixel(4, 4).0, [0, 0, 0]);
        assert_eq!(out.get_pixel(5, 5).0, [200, 10, 10]);
        assert_eq!(out.get_pixel(1, 1).0, [200, 10, 10]);
    }

    #[test]
    fn transparent_pixels_become_opaque_black() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([9, 9, 9, 0])));
        let out = black_out(img, &[region(0.0, 0.0, 0.5, 1.0)], 0).to_rgba8();
        assert_eq!(out.get_pixel(0, 3).0, [0, 0, 0, 255]);
        assert_eq!(out.get_pixel(3, 3).0, [9, 9, 9, 0]);
    }

    #[test]
    fn other_pages_are_ignored() {
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(4, 4, Rgb([255, 255, 255])));
        let r = Region {
            page: 1,
            ..region(0.0, 0.0, 1.0, 1.0)
        };
        let out = black_out(img, &[r], 0).to_rgb8();
        assert_eq!(out.get_pixel(0, 0).0, [255, 255, 255]);
    }

    #[test]
    fn bad_boxes_are_refused() {
        for r in [
            Redact { regions: vec![] },
            Redact {
                regions: vec![region(0.0, 0.0, 0.0, 0.5)],
            },
            Redact {
                regions: vec![region(f32::INFINITY, 0.0, 0.1, 0.5)],
            },
        ] {
            assert_eq!(validate(&r).unwrap_err().code(), "invalid_options");
        }
    }
}
