//! A small document model: what Dropwheel carries between TXT, DOCX and PDF.
//!
//! It keeps what reading a document needs (paragraphs, headings, emphasis, lists, simple
//! tables, pictures and page breaks) and nothing about exact layout.

/// Sizes are in points (1/72 inch).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub page: Page,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Page {
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
}

impl Default for Page {
    /// A4 with one-inch margins, as Word starts.
    fn default() -> Self {
        Page {
            width: 595.28,
            height: 841.89,
            margin_top: 72.0,
            margin_right: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
        }
    }
}

impl Page {
    pub fn content_width(&self) -> f32 {
        (self.width - self.margin_left - self.margin_right).max(72.0)
    }

    pub fn content_height(&self) -> f32 {
        (self.height - self.margin_top - self.margin_bottom).max(72.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Paragraph),
    Table(Table),
    Picture(Picture),
    PageBreak,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Paragraph {
    pub spans: Vec<Span>,
    pub style: ParagraphStyle,
}

impl Paragraph {
    pub fn plain(text: impl Into<String>) -> Self {
        Paragraph {
            spans: vec![Span::plain(text)],
            style: ParagraphStyle::default(),
        }
    }

    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    /// Whether the paragraph reads right to left: as set, or from its first letter.
    pub fn is_rtl(&self) -> bool {
        self.style.rtl.unwrap_or_else(|| starts_rtl(&self.text()))
    }
}

/// Whether the first strong character of `text` is right to left, as Unicode decides a
/// paragraph's direction.
pub fn starts_rtl(text: &str) -> bool {
    unicode_bidi::ParagraphBidiInfo::new(text, None)
        .paragraph_level
        .is_rtl()
}

/// A stretch of text in one style.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    /// The paragraph's size when `None`.
    pub size: Option<f32>,
    pub color: Option<[u8; 3]>,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Span {
            text: text.into(),
            ..Span::default()
        }
    }

    pub fn same_style(&self, other: &Span) -> bool {
        self.bold == other.bold
            && self.italic == other.italic
            && self.underline == other.underline
            && self.strike == other.strike
            && self.size == other.size
            && self.color == other.color
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    /// Left for left-to-right text, right for right-to-left.
    #[default]
    Start,
    Center,
    End,
    Justify,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphStyle {
    /// 0 for body text, 1 to 6 for headings.
    pub heading: u8,
    pub align: Align,
    /// Right to left when `Some(true)`; decided by the text when `None`.
    pub rtl: Option<bool>,
    /// List marker, such as "•" or "2.".
    pub list: Option<String>,
    /// Indent from the start edge.
    pub indent: f32,
    pub space_before: f32,
    pub space_after: f32,
    /// Size of text without its own.
    pub size: f32,
    /// Starts on a new page.
    pub page_break_before: bool,
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        ParagraphStyle {
            heading: 0,
            align: Align::Start,
            rtl: None,
            list: None,
            indent: 0.0,
            space_before: 0.0,
            space_after: 6.0,
            size: 11.0,
            page_break_before: false,
        }
    }
}

impl ParagraphStyle {
    /// Size and spacing for a heading of `level`.
    pub fn heading(level: u8) -> Self {
        let size = match level {
            1 => 20.0,
            2 => 16.0,
            3 => 13.0,
            _ => 12.0,
        };
        ParagraphStyle {
            heading: level.clamp(1, 6),
            size,
            space_before: 12.0,
            space_after: 6.0,
            ..ParagraphStyle::default()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table {
    pub rows: Vec<Vec<Cell>>,
    /// Relative column widths; equal when empty.
    pub widths: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub paragraphs: Vec<Paragraph>,
    /// Columns this cell covers.
    pub span: u16,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            paragraphs: Vec::new(),
            span: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    /// An encoded PNG or JPEG.
    pub data: Vec<u8>,
    pub format: PictureFormat,
    pub pixel_width: u32,
    pub pixel_height: u32,
    /// Displayed size in points.
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureFormat {
    Png,
    Jpeg,
}

impl PictureFormat {
    pub fn extension(self) -> &'static str {
        match self {
            PictureFormat::Png => "png",
            PictureFormat::Jpeg => "jpeg",
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            PictureFormat::Png => "image/png",
            PictureFormat::Jpeg => "image/jpeg",
        }
    }
}

impl Document {
    /// Plain text: paragraphs on their own lines, table cells separated by tabs and pages
    /// by a blank line. Lines end in `\r\n`, as Windows tools expect.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let line = |out: &mut String, text: &str| {
            out.push_str(text);
            out.push_str("\r\n");
        };
        for block in &self.blocks {
            match block {
                Block::Paragraph(p) => line(&mut out, &paragraph_text(p)),
                Block::Table(t) => {
                    for row in &t.rows {
                        let cells: Vec<String> = row
                            .iter()
                            .map(|c| {
                                c.paragraphs
                                    .iter()
                                    .map(paragraph_text)
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            })
                            .collect();
                        line(&mut out, &cells.join("\t"));
                    }
                }
                Block::Picture(_) => {}
                Block::PageBreak => line(&mut out, ""),
            }
        }
        out
    }
}

fn paragraph_text(p: &Paragraph) -> String {
    let text = p.text();
    match &p.style.list {
        Some(marker) => format!(
            "{}{marker} {text}",
            "  ".repeat((p.style.indent / 36.0) as usize)
        ),
        None => text,
    }
}

impl Picture {
    /// A picture from an image file's bytes, shown at `size` in points, or at its pixel
    /// size at 96 dpi. PNG and JPEG are kept as they are; GIF, BMP, TIFF and WebP become
    /// PNG. `None` for anything else, such as Windows metafiles.
    pub fn from_bytes(data: Vec<u8>, size: Option<(f32, f32)>) -> Option<Picture> {
        use image::{ImageFormat, ImageReader};
        let reader = ImageReader::new(std::io::Cursor::new(&data))
            .with_guessed_format()
            .ok()?;
        let format = reader.format()?;
        let (format, data) = match format {
            ImageFormat::Png => (PictureFormat::Png, data),
            ImageFormat::Jpeg => (PictureFormat::Jpeg, data),
            ImageFormat::Gif | ImageFormat::Bmp | ImageFormat::Tiff | ImageFormat::WebP => {
                let mut reader = reader;
                reader.limits(crate::imaging::limits());
                let image = reader.decode().ok()?;
                let mut png = Vec::new();
                image
                    .write_to(&mut std::io::Cursor::new(&mut png), ImageFormat::Png)
                    .ok()?;
                (PictureFormat::Png, png)
            }
            _ => return None,
        };
        let (pixel_width, pixel_height) = ImageReader::new(std::io::Cursor::new(&data))
            .with_guessed_format()
            .ok()?
            .into_dimensions()
            .ok()?;
        if pixel_width == 0 || pixel_height == 0 {
            return None;
        }
        let (width, height) = match size {
            Some((w, h)) if w > 0.5 && h > 0.5 && w.is_finite() && h.is_finite() => (w, h),
            _ => (pixel_width as f32 * 0.75, pixel_height as f32 * 0.75),
        };
        Some(Picture {
            data,
            format,
            pixel_width,
            pixel_height,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_keeps_structure() {
        let mut item = Paragraph::plain("milk");
        item.style.list = Some("•".into());
        let doc = Document {
            blocks: vec![
                Block::Paragraph(Paragraph::plain("Title")),
                Block::Paragraph(item),
                Block::Table(Table {
                    rows: vec![vec![
                        Cell {
                            paragraphs: vec![Paragraph::plain("a")],
                            span: 1,
                        },
                        Cell {
                            paragraphs: vec![Paragraph::plain("b")],
                            span: 1,
                        },
                    ]],
                    widths: vec![],
                }),
                Block::PageBreak,
                Block::Paragraph(Paragraph::plain("end")),
            ],
            page: Page::default(),
        };
        assert_eq!(doc.to_text(), "Title\r\n• milk\r\na\tb\r\n\r\nend\r\n");
    }

    #[test]
    fn direction_comes_from_the_text() {
        assert!(Paragraph::plain("مرحبا Dropwheel").is_rtl());
        assert!(!Paragraph::plain("Dropwheel مرحبا").is_rtl());
        assert!(!Paragraph::plain("123").is_rtl());
    }
}
