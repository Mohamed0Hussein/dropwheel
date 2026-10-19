//! Writing new PDFs with `krilla`: documents laid out by [`super::layout`], and pages
//! that are each one picture (images converted to PDF, redacted pages).
//!
//! Text is embedded as subsetted fonts with its Unicode text attached, so it can be
//! selected, searched and copied.

use std::collections::HashMap;

use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Rect, Size, Transform};
use krilla::image::Image;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule};
use krilla::text::{Font, GlyphId, KrillaGlyph};
use krilla::{Data, Document as PdfDocument};

use super::layout::{self, Item, PageOut};
use crate::action::Options;
use crate::doc::{Document, Picture, PictureFormat};
use crate::error::{Error, Result};
use crate::kind::FileKind;
use crate::text::{self, FontId};

/// Slant of synthesised italics, as the tangent of the angle (about 12°).
const ITALIC_SLANT: f32 = 0.21;

fn encode_error(reason: impl std::fmt::Display) -> Error {
    Error::Encode {
        kind: FileKind::Pdf,
        reason: reason.to_string(),
    }
}

fn fill(color: [u8; 3]) -> Fill {
    Fill {
        paint: rgb::Color::new(color[0], color[1], color[2]).into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }
}

fn image(picture: &Picture) -> Result<Image> {
    let data: Data = picture.data.clone().into();
    match picture.format {
        PictureFormat::Jpeg => Image::from_jpeg(data, true),
        PictureFormat::Png => Image::from_png(data, true),
    }
    .map_err(encode_error)
}

/// Lays out `doc` and writes it as a PDF.
pub fn document(doc: &Document, opts: &Options) -> Result<Vec<u8>> {
    let pages = layout::layout(doc);
    opts.check_cancelled()?;
    opts.report(0.6);
    render(&pages, opts)
}

fn render(pages: &[PageOut<'_>], opts: &Options) -> Result<Vec<u8>> {
    let mut pdf = PdfDocument::new();
    let mut fonts: HashMap<FontId, Font> = HashMap::new();
    for (n, page) in pages.iter().enumerate() {
        opts.check_cancelled()?;
        let settings = PageSettings::from_wh(page.width, page.height)
            .ok_or_else(|| encode_error("invalid page size"))?;
        let mut out = pdf.start_page_with(settings);
        let mut surface = out.surface();
        for item in &page.items {
            match item {
                Item::Glyphs {
                    x,
                    baseline,
                    glyphs,
                    text,
                    color,
                    italic,
                } => {
                    let Some(first) = glyphs.first() else {
                        continue;
                    };
                    let font = match fonts.get(&first.font) {
                        Some(f) => f.clone(),
                        None => {
                            let source = text::font(first.font);
                            let f = Font::new(Data::from(source.data), source.index)
                                .ok_or_else(|| encode_error("a font could not be embedded"))?;
                            fonts.insert(first.font, f.clone());
                            f
                        }
                    };
                    let size = first.size.max(0.1);
                    let krilla_glyphs: Vec<KrillaGlyph> = glyphs
                        .iter()
                        .map(|g| {
                            KrillaGlyph::new(
                                GlyphId::new(u32::from(g.id)),
                                g.advance / size,
                                g.x_offset / size,
                                g.y_offset / size,
                                0.0,
                                g.text.clone(),
                                None,
                            )
                        })
                        .collect();
                    surface.set_fill(Some(fill(*color)));
                    if *italic {
                        surface.push_transform(&Transform::from_row(
                            1.0,
                            0.0,
                            -ITALIC_SLANT,
                            1.0,
                            *x,
                            *baseline,
                        ));
                        surface.draw_glyphs(
                            Point::from_xy(0.0, 0.0),
                            &krilla_glyphs,
                            font,
                            text,
                            size,
                            false,
                        );
                        surface.pop();
                    } else {
                        surface.draw_glyphs(
                            Point::from_xy(*x, *baseline),
                            &krilla_glyphs,
                            font,
                            text,
                            size,
                            false,
                        );
                    }
                }
                Item::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    let Some(rect) = Rect::from_xywh(*x, *y, width.max(0.01), height.max(0.01))
                    else {
                        continue;
                    };
                    let mut builder = PathBuilder::new();
                    builder.push_rect(rect);
                    if let Some(path) = builder.finish() {
                        surface.set_fill(Some(fill(*color)));
                        surface.draw_path(&path);
                    }
                }
                Item::Picture {
                    x,
                    y,
                    width,
                    height,
                    picture,
                } => {
                    let Some(size) = Size::from_wh(width.max(0.01), height.max(0.01)) else {
                        continue;
                    };
                    let img = image(picture)?;
                    surface.push_transform(&Transform::from_translate(*x, *y));
                    surface.draw_image(img, size);
                    surface.pop();
                }
            }
        }
        surface.finish();
        out.finish();
        opts.report(0.6 + 0.35 * (n + 1) as f32 / pages.len().max(1) as f32);
    }
    pdf.finish().map_err(|e| encode_error(format!("{e:?}")))
}

/// One page that is a single picture filling a page of `width` × `height` points, or
/// placed inside it at `x`, `y` with the given size.
pub struct PicturePage {
    pub picture: Picture,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub picture_width: f32,
    pub picture_height: f32,
}

impl PicturePage {
    /// The picture covering a whole page of its own size.
    pub fn full(picture: Picture, width: f32, height: f32) -> Self {
        PicturePage {
            picture,
            width,
            height,
            x: 0.0,
            y: 0.0,
            picture_width: width,
            picture_height: height,
        }
    }
}

/// Writes a PDF with one picture per page.
pub fn pictures(pages: &[PicturePage], opts: &Options) -> Result<Vec<u8>> {
    let laid: Vec<PageOut> = pages
        .iter()
        .map(|p| PageOut {
            width: p.width,
            height: p.height,
            items: vec![Item::Picture {
                x: p.x,
                y: p.y,
                width: p.picture_width,
                height: p.picture_height,
                picture: &p.picture,
            }],
        })
        .collect();
    render(&laid, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{Block, Paragraph, ParagraphStyle, Span};

    fn sample() -> Document {
        let mut heading = Paragraph {
            spans: vec![Span {
                bold: true,
                ..Span::plain("Quarterly report")
            }],
            style: ParagraphStyle::heading(1),
        };
        heading.style.size = 20.0;
        let mixed = Paragraph {
            spans: vec![
                Span::plain("Plain, "),
                Span {
                    italic: true,
                    ..Span::plain("slanted")
                },
                Span::plain(" and Arabic: مرحبا بالعالم"),
            ],
            style: ParagraphStyle::default(),
        };
        Document {
            blocks: vec![
                Block::Paragraph(heading),
                Block::Paragraph(mixed),
                Block::PageBreak,
                Block::Paragraph(Paragraph::plain("هذا تقرير ربع سنوي")),
            ],
            ..Document::default()
        }
    }

    #[test]
    fn text_survives_into_the_pdf() {
        let bytes = document(&sample(), &Options::default()).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        if !super::super::test_library() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.pdf");
        std::fs::write(&path, &bytes).unwrap();
        if let Some(debug) = std::env::var_os("DROPWHEEL_DEBUG_PDF") {
            std::fs::write(debug, &bytes).unwrap();
        }
        let text = super::super::read::text(&path, &Options::default()).unwrap();
        let texts = [text.as_str(), text.as_str()];
        assert_eq!(super::super::page_sizes(&path).unwrap().len(), 2);
        assert!(texts[0].contains("Quarterly report"), "{texts:?}");
        assert!(texts[0].contains("slanted"), "{texts:?}");
        assert!(texts[0].contains("مرحبا بالعالم"), "{texts:?}");
        assert!(texts[1].contains("هذا تقرير ربع سنوي"), "{texts:?}");
    }

    #[test]
    fn pictures_fill_their_pages() {
        let png = {
            let img = image::RgbImage::from_pixel(20, 10, image::Rgb([200, 0, 0]));
            let mut out = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .unwrap();
            out
        };
        let picture = Picture::from_bytes(png, None).unwrap();
        let pages = [PicturePage::full(picture, 200.0, 100.0)];
        let bytes = pictures(&pages, &Options::default()).unwrap();
        if !super::super::test_library() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.pdf");
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(super::super::page_sizes(&path).unwrap(), [(200.0, 100.0)]);
        let img = super::super::render_page(&path, 0, 200).unwrap();
        let p = img.get_pixel(100, 50).0;
        assert!(p[0] > 180 && p[1] < 30, "{p:?}");
    }
}
