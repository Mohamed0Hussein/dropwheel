//! Rotate, flip, grayscale, brightness and contrast.

use std::path::{Path, PathBuf};

use image::DynamicImage;

use super::support;
use crate::action::{Action, ActionGroup, Edit, Options, Params};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};

pub struct EditImage;

pub static EDIT: EditImage = EditImage;

/// Applies `edit` the way the dialog previews it with CSS: `filter: brightness()
/// contrast() grayscale()` on the colours, then the flips, then the rotation.
pub fn apply(image: DynamicImage, edit: &Edit) -> Result<DynamicImage> {
    if !matches!(edit.rotate, 0 | 90 | 180 | 270) {
        return Err(Error::InvalidOptions(format!(
            "rotation must be 0, 90, 180 or 270 degrees, not {}",
            edit.rotate
        )));
    }
    if !(-100..=100).contains(&edit.brightness) || !(-100..=100).contains(&edit.contrast) {
        return Err(Error::InvalidOptions(
            "brightness and contrast run from -100 to 100".into(),
        ));
    }

    let mut image = if edit.brightness != 0 || edit.contrast != 0 || edit.grayscale {
        adjust_colours(&image, edit)
    } else {
        image
    };
    if edit.flip_horizontal {
        image = image.fliph();
    }
    if edit.flip_vertical {
        image = image.flipv();
    }
    Ok(match edit.rotate {
        90 => image.rotate90(),
        180 => image.rotate180(),
        270 => image.rotate270(),
        _ => image,
    })
}

fn adjust_colours(image: &DynamicImage, edit: &Edit) -> DynamicImage {
    // CSS brightness(b) multiplies; contrast(k) scales around the middle grey.
    let b = 1.0 + f32::from(edit.brightness) / 100.0;
    let k = 1.0 + f32::from(edit.contrast) / 100.0;
    let lut: [u8; 256] = std::array::from_fn(|v| {
        let c = (v as f32 * b).clamp(0.0, 255.0);
        ((c - 127.5) * k + 127.5).round().clamp(0.0, 255.0) as u8
    });
    let gray = |r: u8, g: u8, bl: u8| {
        // CSS grayscale(1) uses the Rec. 709 luma weights.
        (0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(bl)).round() as u8
    };

    if image.color().has_alpha() {
        let mut rgba = image.to_rgba8();
        for p in rgba.pixels_mut() {
            let [r, g, bl, a] = p.0;
            let (r, g, bl) = (
                lut[usize::from(r)],
                lut[usize::from(g)],
                lut[usize::from(bl)],
            );
            p.0 = if edit.grayscale {
                let y = gray(r, g, bl);
                [y, y, y, a]
            } else {
                [r, g, bl, a]
            };
        }
        DynamicImage::ImageRgba8(rgba)
    } else {
        let mut rgb = image.to_rgb8();
        for p in rgb.pixels_mut() {
            let [r, g, bl] = p.0;
            let (r, g, bl) = (
                lut[usize::from(r)],
                lut[usize::from(g)],
                lut[usize::from(bl)],
            );
            p.0 = if edit.grayscale {
                let y = gray(r, g, bl);
                [y, y, y]
            } else {
                [r, g, bl]
            };
        }
        DynamicImage::ImageRgb8(rgb)
    }
}

impl Action for EditImage {
    fn id(&self) -> &'static str {
        "edit"
    }

    fn label(&self) -> &'static str {
        "EDIT"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        matches!(kind, Jpeg | Png | Webp | Bmp | Tiff | Gif | Avif | Heic)
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        let Params::Edit(edit) = opts.params else {
            return Err(Error::InvalidOptions("edit needs its settings".into()));
        };
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Edit",
                kind,
            });
        }
        let (kind, decoded) = support::load(input, opts)?;
        let edited = apply(decoded.image, &edit)?;
        opts.report(0.7);
        support::save_like(
            input,
            out,
            "edited",
            kind,
            &edited,
            decoded.icc.as_deref(),
            opts,
        )
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    use super::*;

    fn two_by_one() -> DynamicImage {
        let mut img = RgbImage::new(2, 1);
        img.put_pixel(0, 0, Rgb([200, 100, 50]));
        img.put_pixel(1, 0, Rgb([10, 20, 30]));
        DynamicImage::ImageRgb8(img)
    }

    fn px(img: &DynamicImage, x: u32, y: u32) -> [u8; 3] {
        img.to_rgb8().get_pixel(x, y).0
    }

    #[test]
    fn nothing_changes_by_default() {
        let out = apply(two_by_one(), &Edit::default()).unwrap();
        assert_eq!(px(&out, 0, 0), [200, 100, 50]);
    }

    #[test]
    fn rotation_and_flips() {
        let r = apply(
            two_by_one(),
            &Edit {
                rotate: 90,
                ..Edit::default()
            },
        )
        .unwrap();
        assert_eq!((r.width(), r.height()), (1, 2));
        assert_eq!(px(&r, 0, 0), [200, 100, 50]);
        let f = apply(
            two_by_one(),
            &Edit {
                flip_horizontal: true,
                ..Edit::default()
            },
        )
        .unwrap();
        assert_eq!(px(&f, 0, 0), [10, 20, 30]);
        // Flip first, then rotate, like CSS `rotate() scaleX(-1)`.
        let both = Edit {
            rotate: 90,
            flip_horizontal: true,
            ..Edit::default()
        };
        assert_eq!(px(&apply(two_by_one(), &both).unwrap(), 0, 0), [10, 20, 30]);
    }

    #[test]
    fn colours_follow_css_filters() {
        let bright = Edit {
            brightness: 50,
            ..Edit::default()
        };
        assert_eq!(
            px(&apply(two_by_one(), &bright).unwrap(), 0, 0),
            [255, 150, 75]
        );
        let flat = Edit {
            contrast: -100,
            ..Edit::default()
        };
        assert_eq!(
            px(&apply(two_by_one(), &flat).unwrap(), 0, 0),
            [128, 128, 128]
        );
        let gray = Edit {
            grayscale: true,
            ..Edit::default()
        };
        let [r, g, b] = px(&apply(two_by_one(), &gray).unwrap(), 0, 0);
        assert!(r == g && g == b && r == 118, "{r}");
    }

    #[test]
    fn alpha_survives() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([100, 100, 100, 40])));
        let out = apply(
            img,
            &Edit {
                grayscale: true,
                ..Edit::default()
            },
        )
        .unwrap();
        assert_eq!(out.to_rgba8().get_pixel(0, 0).0[3], 40);
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(
            apply(
                two_by_one(),
                &Edit {
                    rotate: 45,
                    ..Edit::default()
                }
            )
            .is_err()
        );
        assert!(
            apply(
                two_by_one(),
                &Edit {
                    brightness: 101,
                    ..Edit::default()
                }
            )
            .is_err()
        );
    }
}
