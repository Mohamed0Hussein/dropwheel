//! Rectangles, arrows and text drawn over an image.
//!
//! The geometry here is mirrored by the Annotate dialog's preview, so what the user
//! draws is what gets saved.

use std::path::{Path, PathBuf};

use image::{DynamicImage, RgbaImage};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, PixmapMut, Stroke, Transform};

use super::support;
use crate::action::{Action, ActionGroup, Annotate, Options, Params, Shape};
use crate::error::{Error, Result};
use crate::kind::FileKind::{self, *};
use crate::text::{self, PathSink, Weight};

pub struct AnnotateImage;

pub static ANNOTATE: AnnotateImage = AnnotateImage;

const MAX_SHAPES: usize = 1000;
const MAX_TEXT_CHARS: usize = 20_000;
/// Line height of annotation text, in ems, as in the dialog's CSS.
pub const LINE_HEIGHT: f32 = 1.25;

fn validate(a: &Annotate) -> Result<()> {
    let bad = |what: &str| Err(Error::InvalidOptions(what.to_string()));
    if a.ref_width == 0 || a.ref_height == 0 {
        return bad("the annotations were drawn on an empty image");
    }
    if a.shapes.is_empty() {
        return bad("there is nothing to draw");
    }
    if a.shapes.len() > MAX_SHAPES {
        return bad("there are too many annotations");
    }
    let limit = 10.0 * a.ref_width.max(a.ref_height) as f32;
    let place = |v: f32| v.is_finite() && v.abs() <= limit;
    let mut chars = 0;
    for shape in &a.shapes {
        let ok = match shape {
            Shape::Rect {
                x,
                y,
                width,
                height,
                stroke,
                ..
            } => [*x, *y, *width, *height].into_iter().all(place) && thickness(*stroke),
            Shape::Arrow {
                from, to, stroke, ..
            } => from.iter().chain(to).copied().all(place) && thickness(*stroke),
            Shape::Text {
                x, y, text, size, ..
            } => {
                chars += text.chars().count();
                place(*x) && place(*y) && size.is_finite() && (1.0..=5000.0).contains(size)
            }
        };
        if !ok {
            return bad("an annotation is out of range");
        }
    }
    if chars > MAX_TEXT_CHARS {
        return bad("the annotations hold too much text");
    }
    Ok(())
}

fn thickness(stroke: f32) -> bool {
    stroke.is_finite() && stroke > 0.0 && stroke <= 1000.0
}

/// Where an arrow's line ends and the corners of its head: tip, then the two barbs.
/// `None` for an arrow too short to draw.
pub fn arrow_geometry(
    from: [f32; 2],
    to: [f32; 2],
    stroke: f32,
) -> Option<([f32; 2], [[f32; 2]; 3])> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    if len < 0.5 {
        return None;
    }
    let (ux, uy) = (dx / len, dy / len);
    let head = (stroke * 4.0 + 6.0).min(len);
    let half = head * 0.5;
    let base = [to[0] - ux * head, to[1] - uy * head];
    // The line runs into the head so no gap shows at its base.
    let shaft_end = [to[0] - ux * head * 0.8, to[1] - uy * head * 0.8];
    Some((
        shaft_end,
        [
            to,
            [base[0] - uy * half, base[1] + ux * half],
            [base[0] + uy * half, base[1] - ux * half],
        ],
    ))
}

/// Draws `annotate` over `image`.
pub fn apply(image: DynamicImage, annotate: &Annotate) -> Result<DynamicImage> {
    validate(annotate)?;
    let (width, height) = (image.width(), image.height());
    let mut rgba: RgbaImage = image.into_rgba8();
    let opaque = rgba.pixels().all(|p| p.0[3] == 255);
    if !opaque {
        premultiply(&mut rgba);
    }
    {
        let mut pixmap =
            PixmapMut::from_bytes(rgba.as_mut(), width, height).ok_or(Error::Internal)?;
        let sx = width as f32 / annotate.ref_width as f32;
        let sy = height as f32 / annotate.ref_height as f32;
        let scale = Scale {
            x: sx,
            y: sy,
            size: (sx * sy).sqrt(),
        };
        for shape in &annotate.shapes {
            draw(&mut pixmap, shape, scale);
        }
    }
    if !opaque {
        unpremultiply(&mut rgba);
    }
    Ok(DynamicImage::ImageRgba8(rgba))
}

#[derive(Clone, Copy)]
struct Scale {
    x: f32,
    y: f32,
    /// For thicknesses and font sizes.
    size: f32,
}

fn paint(color: [u8; 3]) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], 255);
    paint
}

fn draw(pixmap: &mut PixmapMut, shape: &Shape, s: Scale) {
    let identity = Transform::identity();
    match shape {
        Shape::Rect {
            x,
            y,
            width,
            height,
            color,
            stroke,
        } => {
            let (x0, x1) = (
                (x * s.x).min((x + width) * s.x),
                (x * s.x).max((x + width) * s.x),
            );
            let (y0, y1) = (
                (y * s.y).min((y + height) * s.y),
                (y * s.y).max((y + height) * s.y),
            );
            let Some(rect) = tiny_skia::Rect::from_ltrb(x0, y0, x1, y1) else {
                return;
            };
            let path = PathBuilder::from_rect(rect);
            let stroke = Stroke {
                width: stroke * s.size,
                line_join: LineJoin::Miter,
                ..Stroke::default()
            };
            pixmap.stroke_path(&path, &paint(*color), &stroke, identity, None);
        }
        Shape::Arrow {
            from,
            to,
            color,
            stroke,
        } => {
            let from = [from[0] * s.x, from[1] * s.y];
            let to = [to[0] * s.x, to[1] * s.y];
            let width = stroke * s.size;
            let Some((shaft_end, head)) = arrow_geometry(from, to, width) else {
                return;
            };
            let paint = paint(*color);
            let mut line = PathBuilder::new();
            line.move_to(from[0], from[1]);
            line.line_to(shaft_end[0], shaft_end[1]);
            if let Some(path) = line.finish() {
                let stroke = Stroke {
                    width,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                };
                pixmap.stroke_path(&path, &paint, &stroke, identity, None);
            }
            let mut tip = PathBuilder::new();
            tip.move_to(head[0][0], head[0][1]);
            tip.line_to(head[1][0], head[1][1]);
            tip.line_to(head[2][0], head[2][1]);
            tip.close();
            if let Some(path) = tip.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, identity, None);
            }
        }
        Shape::Text {
            x,
            y,
            text,
            color,
            size,
            bold,
        } => {
            let weight = if *bold { Weight::Bold } else { Weight::Regular };
            let mut builder = Builder(PathBuilder::new());
            text_outline(text, x * s.x, y * s.y, size * s.size, weight, &mut builder);
            if let Some(path) = builder.0.finish() {
                pixmap.fill_path(&path, &paint(*color), FillRule::Winding, identity, None);
            }
        }
    }
}

/// Traces `text` as the dialog lays it out: lines `LINE_HEIGHT` apart, each aligned to
/// the start of its own direction within the widest line.
pub fn text_outline(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    weight: Weight,
    sink: &mut dyn PathSink,
) {
    let paragraphs: Vec<_> = text::clean_lines(text)
        .iter()
        .map(|line| text::layout(line, weight, None))
        .collect();
    let font = text::primary(weight);
    let half_leading = (LINE_HEIGHT - (font.ascender() - font.descender())) / 2.0;
    let box_width = paragraphs
        .iter()
        .flat_map(|p| &p.lines)
        .map(|l| l.width)
        .fold(0.0f32, f32::max);
    for (i, p) in paragraphs.iter().enumerate() {
        let baseline = y + size * (i as f32 * LINE_HEIGHT + half_leading + font.ascender());
        for line in &p.lines {
            let mut pen = if p.rtl {
                x + (box_width - line.width) * size
            } else {
                x
            };
            for g in &line.glyphs {
                text::font(g.font).outline(
                    g.id,
                    size,
                    pen + g.x_offset * size,
                    baseline - g.y_offset * size,
                    sink,
                );
                pen += g.advance * size;
            }
        }
    }
}

struct Builder(PathBuilder);

impl PathSink for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

fn premultiply(image: &mut RgbaImage) {
    for p in image.pixels_mut() {
        let a = u16::from(p.0[3]);
        for c in &mut p.0[..3] {
            *c = ((u16::from(*c) * a + 127) / 255) as u8;
        }
    }
}

fn unpremultiply(image: &mut RgbaImage) {
    for p in image.pixels_mut() {
        let a = u16::from(p.0[3]);
        if a == 0 || a == 255 {
            continue;
        }
        for c in &mut p.0[..3] {
            *c = ((u16::from(*c) * 255 + a / 2) / a).min(255) as u8;
        }
    }
}

impl Action for AnnotateImage {
    fn id(&self) -> &'static str {
        "annotate"
    }

    fn label(&self) -> &'static str {
        "ANNOTATE"
    }

    fn group(&self) -> ActionGroup {
        ActionGroup::Tools
    }

    fn applicable(&self, kind: FileKind) -> bool {
        matches!(kind, Jpeg | Png | Webp | Bmp | Tiff | Gif | Avif | Heic)
    }

    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf> {
        let Params::Annotate(annotate) = &opts.params else {
            return Err(Error::InvalidOptions(
                "annotate needs something to draw".into(),
            ));
        };
        validate(annotate)?;
        let kind = FileKind::detect(input)?;
        if !self.applicable(kind) {
            return Err(Error::Unsupported {
                action: "Annotate",
                kind,
            });
        }
        let (kind, decoded) = support::load(input, opts)?;
        let drawn = apply(decoded.image, annotate)?;
        opts.report(0.7);
        support::save_like(
            input,
            out,
            "annotated",
            kind,
            &drawn,
            decoded.icc.as_deref(),
            opts,
        )
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgb, RgbImage, Rgba};

    use super::*;

    fn white(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([255, 255, 255])))
    }

    fn annotate(shapes: Vec<Shape>) -> Annotate {
        Annotate {
            shapes,
            ref_width: 100,
            ref_height: 100,
        }
    }

    fn px(img: &DynamicImage, x: u32, y: u32) -> [u8; 4] {
        img.to_rgba8().get_pixel(x, y).0
    }

    const RED: [u8; 3] = [255, 0, 0];

    #[test]
    fn rectangles_are_outlines() {
        let shapes = vec![Shape::Rect {
            x: 20.0,
            y: 20.0,
            width: 60.0,
            height: 40.0,
            color: RED,
            stroke: 4.0,
        }];
        let out = apply(white(100, 100), &annotate(shapes)).unwrap();
        assert_eq!(px(&out, 20, 40), [255, 0, 0, 255]);
        assert_eq!(px(&out, 50, 60), [255, 0, 0, 255]);
        assert_eq!(px(&out, 50, 40), [255, 255, 255, 255]);
        assert_eq!(px(&out, 5, 5), [255, 255, 255, 255]);
    }

    #[test]
    fn rectangles_drawn_backwards_still_draw() {
        let shapes = vec![Shape::Rect {
            x: 80.0,
            y: 60.0,
            width: -60.0,
            height: -40.0,
            color: RED,
            stroke: 4.0,
        }];
        let out = apply(white(100, 100), &annotate(shapes)).unwrap();
        assert_eq!(px(&out, 20, 40), [255, 0, 0, 255]);
    }

    #[test]
    fn arrows_end_in_a_head() {
        let shapes = vec![Shape::Arrow {
            from: [10.0, 50.0],
            to: [90.0, 50.0],
            color: RED,
            stroke: 4.0,
        }];
        let out = apply(white(100, 100), &annotate(shapes)).unwrap();
        // Along the line, and in the head's barbs but not beside the line.
        assert_eq!(px(&out, 30, 50), [255, 0, 0, 255]);
        assert_eq!(px(&out, 75, 44), [255, 0, 0, 255]);
        assert_eq!(px(&out, 30, 44), [255, 255, 255, 255]);
        let (_, head) = arrow_geometry([10.0, 50.0], [90.0, 50.0], 4.0).unwrap();
        assert_eq!(head[0], [90.0, 50.0]);
        assert!(arrow_geometry([1.0, 1.0], [1.0, 1.0], 4.0).is_none());
    }

    #[test]
    fn text_is_drawn_inside_its_box() {
        for text in ["Hi", "سلام"] {
            let shapes = vec![Shape::Text {
                x: 10.0,
                y: 10.0,
                text: text.into(),
                color: RED,
                size: 40.0,
                bold: true,
            }];
            let out = apply(white(100, 100), &annotate(shapes))
                .unwrap()
                .to_rgba8();
            let inked: Vec<(u32, u32)> = out
                .enumerate_pixels()
                .filter(|(_, _, p)| p.0 != [255, 255, 255, 255])
                .map(|(x, y, _)| (x, y))
                .collect();
            assert!(inked.len() > 100, "{text}");
            // Inside the line box, give or take Arabic descenders.
            assert!(
                inked
                    .iter()
                    .all(|(x, y)| *x >= 10 && *y >= 10 && *y < 10 + 70),
                "{text}"
            );
        }
    }

    #[test]
    fn shapes_scale_with_the_image() {
        let shapes = vec![Shape::Rect {
            x: 20.0,
            y: 20.0,
            width: 60.0,
            height: 40.0,
            color: RED,
            stroke: 4.0,
        }];
        // Drawn on 100x100, applied to 200x200.
        let out = apply(white(200, 200), &annotate(shapes)).unwrap();
        assert_eq!(px(&out, 40, 80), [255, 0, 0, 255]);
        assert_eq!(px(&out, 100, 80), [255, 255, 255, 255]);
    }

    #[test]
    fn transparency_is_kept_around_the_drawing() {
        let clear = DynamicImage::ImageRgba8(RgbaImage::from_pixel(50, 50, Rgba([0, 0, 255, 0])));
        let half = DynamicImage::ImageRgba8(RgbaImage::from_pixel(50, 50, Rgba([0, 0, 255, 128])));
        let shapes = vec![Shape::Rect {
            x: 10.0,
            y: 10.0,
            width: 30.0,
            height: 30.0,
            color: RED,
            stroke: 4.0,
        }];
        let a = Annotate {
            shapes,
            ref_width: 50,
            ref_height: 50,
        };
        let out = apply(clear, &a).unwrap();
        assert_eq!(px(&out, 25, 25)[3], 0);
        assert_eq!(px(&out, 10, 25), [255, 0, 0, 255]);
        let out = apply(half, &a).unwrap();
        let p = px(&out, 25, 25);
        assert_eq!(p[3], 128);
        assert!(p[2] >= 253, "{p:?}");
    }

    #[test]
    fn bad_annotations_are_refused() {
        let rect = |stroke: f32| Shape::Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            color: RED,
            stroke,
        };
        for a in [
            annotate(vec![]),
            annotate(vec![rect(0.0)]),
            annotate(vec![rect(f32::NAN)]),
            annotate(vec![rect(1.0); MAX_SHAPES + 1]),
            Annotate {
                ref_width: 0,
                ..annotate(vec![rect(1.0)])
            },
            annotate(vec![Shape::Text {
                x: 0.0,
                y: 0.0,
                text: "x".repeat(MAX_TEXT_CHARS + 1),
                color: RED,
                size: 10.0,
                bold: false,
            }]),
        ] {
            let err = apply(white(4, 4), &a).unwrap_err();
            assert_eq!(err.code(), "invalid_options");
        }
    }
}
