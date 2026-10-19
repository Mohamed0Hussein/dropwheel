//! Turning a PDF back into a document: paragraphs rebuilt from lines, headings from
//! larger text, and pictures cut from the rendered page. Pages with no text, such as
//! scans, come out as pictures of the page.

use std::path::Path;

use image::{DynamicImage, RgbImage};

use super::engine;
use super::extract::{self, Box2, Line, PageText};
use crate::action::Options;
use crate::doc::{
    Align, Block, Document, Page, Paragraph, ParagraphStyle, Picture, PictureFormat, Span,
};
use crate::error::Result;
use crate::imaging::{self, EncodeOptions};
use crate::kind::FileKind;

/// Pictures are cut from pages rendered at this resolution.
const PICTURE_DPI: f32 = 150.0;

struct PageContent {
    text: PageText,
    /// Pictures cut from the page, with where they were.
    pictures: Vec<(Box2, Picture)>,
    /// The whole page as a picture, for pages without text.
    scan: Option<Picture>,
}

fn encode(image: &RgbImage) -> Option<Picture> {
    let pixels = u64::from(image.width()) * u64::from(image.height());
    let (format, kind, opts) = if pixels > 40_000 {
        let opts = EncodeOptions {
            quality: 88,
            ..EncodeOptions::default()
        };
        (PictureFormat::Jpeg, FileKind::Jpeg, opts)
    } else {
        let opts = EncodeOptions {
            lossless: true,
            ..EncodeOptions::default()
        };
        (PictureFormat::Png, FileKind::Png, opts)
    };
    let data = imaging::encode(&DynamicImage::ImageRgb8(image.clone()), kind, &opts).ok()?;
    let picture = Picture::from_bytes(data, None)?;
    debug_assert_eq!(picture.format, format);
    Some(picture)
}

fn read_pages(path: &Path, opts: &Options, with_pictures: bool) -> Result<Vec<PageContent>> {
    let data = engine::read(path)?;
    engine::with(|pdfium| {
        let document = engine::open(pdfium, path, &data)?;
        let count = document.pages().len().max(1) as f32;
        let mut pages = Vec::new();
        for (i, page) in document.pages().iter().enumerate() {
            opts.check_cancelled()?;
            let text = extract::page_text(&page);
            let mut content = PageContent {
                text,
                pictures: Vec::new(),
                scan: None,
            };
            let needs_render = with_pictures
                && (content.text.lines.is_empty() || !content.text.pictures.is_empty());
            if needs_render {
                let scale = engine::scale_for(&page, PICTURE_DPI);
                let rendered = engine::render(path, &page, scale)?;
                if content.text.lines.is_empty() {
                    if let Some(mut picture) = encode(&rendered) {
                        picture.width = content.text.width;
                        picture.height = content.text.height;
                        content.scan = Some(picture);
                    }
                } else {
                    for b in &content.text.pictures {
                        let x = (b.left * scale).max(0.0) as u32;
                        let y = (b.top * scale).max(0.0) as u32;
                        let w =
                            ((b.width() * scale) as u32).min(rendered.width().saturating_sub(x));
                        let h =
                            ((b.height() * scale) as u32).min(rendered.height().saturating_sub(y));
                        if w < 8 || h < 8 {
                            continue;
                        }
                        let crop = image::imageops::crop_imm(&rendered, x, y, w, h).to_image();
                        if let Some(mut picture) = encode(&crop) {
                            picture.width = b.width();
                            picture.height = b.height();
                            content.pictures.push((*b, picture));
                        }
                    }
                }
            }
            pages.push(content);
            opts.report(0.05 + 0.75 * (i + 1) as f32 / count);
        }
        Ok(pages)
    })
}

/// A paragraph being put together from lines.
struct Gathered<'a> {
    lines: Vec<&'a Line>,
}

impl Gathered<'_> {
    fn bounds(&self) -> Box2 {
        let mut b = self.lines[0].bounds;
        for l in &self.lines[1..] {
            b = Box2 {
                left: b.left.min(l.bounds.left),
                top: b.top.min(l.bounds.top),
                right: b.right.max(l.bounds.right),
                bottom: b.bottom.max(l.bounds.bottom),
            };
        }
        b
    }
}

fn starts_item(text: &str) -> bool {
    let t = text.trim_start();
    if t.starts_with(['•', '◦', '▪', '■', '●', '–', '-', '*', '·']) {
        return true;
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    (1..=3).contains(&digits) && t[digits..].starts_with(['.', ')'])
}

/// Groups a page's lines into paragraphs: lines continue a paragraph when they follow
/// closely, in the same size and direction, and the line before ran to the margin.
fn paragraphs(text: &PageText) -> Vec<Gathered<'_>> {
    let left = text
        .lines
        .iter()
        .map(|l| l.bounds.left)
        .fold(f32::MAX, f32::min);
    let right = text
        .lines
        .iter()
        .map(|l| l.bounds.right)
        .fold(f32::MIN, f32::max);
    let column = (right - left).max(1.0);
    let mut out: Vec<Gathered> = Vec::new();
    for line in &text.lines {
        let joins = out.last().and_then(|g| g.lines.last()).is_some_and(|prev| {
            let size = prev.size.max(line.size).max(1.0);
            let gap = line.bounds.top - prev.bounds.bottom;
            let ratio = line.size / prev.size.max(0.1);
            let slack = (3.0 * size).max(0.12 * column);
            let prev_full = if prev.rtl {
                prev.bounds.left <= left + slack
            } else {
                prev.bounds.right >= right - slack
            };
            let indented = if line.rtl {
                line.bounds.right < prev.bounds.right - 1.5 * size
            } else {
                line.bounds.left > prev.bounds.left + 1.5 * size
            };
            prev.rtl == line.rtl
                && gap > -0.5 * size
                && gap < 0.8 * size
                && (0.85..=1.18).contains(&ratio)
                && prev_full
                && !indented
                && !starts_item(&line.text)
        });
        match out.last_mut() {
            Some(g) if joins => g.lines.push(line),
            _ => out.push(Gathered { lines: vec![line] }),
        }
    }
    out
}

/// The text of a paragraph's lines, joined with spaces, or without one where a word
/// was hyphenated across the break. Each piece keeps how it looks.
fn joined(lines: &[&Line]) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let push = |text: &str, look: &extract::Look, spans: &mut Vec<Span>| {
        if text.is_empty() {
            return;
        }
        let span = Span {
            text: String::new(),
            bold: look.bold,
            italic: look.italic,
            size: Some(look.size),
            ..Span::default()
        };
        match spans.last_mut() {
            Some(last) if last.same_style(&span) => last.text.push_str(text),
            _ => spans.push(Span {
                text: text.to_string(),
                ..span
            }),
        }
    };
    for (i, line) in lines.iter().enumerate() {
        let mut text = line.text.as_str();
        let next_lower = lines
            .get(i + 1)
            .and_then(|n| n.text.chars().next())
            .is_some_and(char::is_lowercase);
        let hyphenated = next_lower
            && text.ends_with('-')
            && text[..text.len() - 1]
                .chars()
                .last()
                .is_some_and(char::is_alphabetic);
        if hyphenated {
            text = &text[..text.len() - 1];
        }
        for (range, look) in &line.looks {
            let start = range.start.min(text.len());
            let end = range.end.min(text.len());
            if let Some(piece) = text.get(start..end) {
                push(piece, look, &mut spans);
            }
        }
        if i + 1 < lines.len()
            && !hyphenated
            && let Some(look) = line.looks.last().map(|(_, l)| *l)
        {
            push(" ", &look, &mut spans);
        }
    }
    spans
}

/// The most common text size, weighted by length: the body text.
fn body_size(pages: &[PageContent]) -> f32 {
    let mut counts: Vec<(f32, usize)> = Vec::new();
    for line in pages.iter().flat_map(|p| &p.text.lines) {
        let n = line.text.chars().count();
        match counts
            .iter_mut()
            .find(|(s, _)| (*s - line.size).abs() < 0.26)
        {
            Some((_, c)) => *c += n,
            None => counts.push((line.size, n)),
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map_or(11.0, |(s, _)| s)
}

/// Reads `path` as a document for DOCX.
pub fn document(path: &Path, opts: &Options) -> Result<Document> {
    let pages = read_pages(path, opts, true)?;
    let body = body_size(&pages);
    let mut blocks = Vec::new();
    let mut page_size = Page::default();
    if let Some(first) = pages.first() {
        page_size.width = first.text.width.clamp(72.0, 14_400.0);
        page_size.height = first.text.height.clamp(72.0, 14_400.0);
        let margin = (page_size.width.min(page_size.height) * 0.08).clamp(18.0, 72.0);
        page_size.margin_top = margin;
        page_size.margin_bottom = margin;
        page_size.margin_left = margin;
        page_size.margin_right = margin;
    }
    for (n, page) in pages.into_iter().enumerate() {
        if n > 0 {
            blocks.push(Block::PageBreak);
        }
        if let Some(scan) = page.scan {
            blocks.push(Block::Picture(scan));
            continue;
        }
        // Paragraphs and pictures in the order they appear down the page.
        let mut placed: Vec<(f32, Block)> = Vec::new();
        let page_centre = page.text.width / 2.0;
        for g in paragraphs(&page.text) {
            let b = g.bounds();
            let size = g.lines.iter().map(|l| l.size).fold(0.0f32, f32::max);
            let spans = joined(&g.lines);
            let text_len: usize = spans.iter().map(|s| s.text.chars().count()).sum();
            let all_bold = spans.iter().all(|s| s.bold || s.text.trim().is_empty());
            let heading = if size >= body * 1.6 {
                1
            } else if size >= body * 1.3 {
                2
            } else if size >= body * 1.12 || (all_bold && g.lines.len() == 1 && text_len < 80) {
                3
            } else {
                0
            };
            let rtl = g.lines[0].rtl;
            let centred = g.lines.len() <= 3
                && ((b.left + b.right) / 2.0 - page_centre).abs() < page.text.width * 0.04
                && b.width() < page.text.width * 0.6;
            let style = ParagraphStyle {
                heading,
                align: if centred { Align::Center } else { Align::Start },
                rtl: Some(rtl),
                size: (size * 2.0).round() / 2.0,
                space_after: 6.0,
                ..ParagraphStyle::default()
            };
            placed.push((b.top, Block::Paragraph(Paragraph { spans, style })));
        }
        for (b, picture) in page.pictures {
            placed.push((b.top, Block::Picture(picture)));
        }
        placed.sort_by(|a, b| a.0.total_cmp(&b.0));
        blocks.extend(placed.into_iter().map(|(_, b)| b));
    }
    Ok(Document {
        blocks,
        page: page_size,
    })
}

/// Reads the text of `path`: each line as on the page, a blank line between paragraphs
/// and between pages. Lines end in `\r\n`.
pub fn text(path: &Path, opts: &Options) -> Result<String> {
    let pages = read_pages(path, opts, false)?;
    let mut out = String::new();
    for (n, page) in pages.iter().enumerate() {
        if n > 0 {
            out.push_str("\r\n");
        }
        for (i, g) in paragraphs(&page.text).iter().enumerate() {
            if i > 0 {
                out.push_str("\r\n");
            }
            for line in &g.lines {
                out.push_str(&line.text);
                out.push_str("\r\n");
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn plain_text_of_a_pdf() {
        if !super::super::test_library() {
            return;
        }
        let text = text(&fixture("text.pdf"), &Options::default()).unwrap();
        assert!(
            text.starts_with(
                "Quarterly report\r\n\r\nDropwheel turns this page into text.\r\nSecond line"
            ),
            "{text:?}"
        );
        assert!(text.contains("Page two has a picture below."));
        let arabic = super::text(&fixture("arabic.pdf"), &Options::default()).unwrap();
        assert!(arabic.contains("مرحبا بالعالم"), "{arabic:?}");
    }

    #[test]
    fn documents_keep_headings_paragraphs_and_pictures() {
        if !super::super::test_library() {
            return;
        }
        let doc = document(&fixture("text.pdf"), &Options::default()).unwrap();
        let paragraphs: Vec<&Paragraph> = doc
            .blocks
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(paragraphs[0].text(), "Quarterly report");
        assert_eq!(paragraphs[0].style.heading, 1);
        assert_eq!(
            paragraphs[1].text(),
            "Dropwheel turns this page into text. Second line of the first paragraph."
        );
        assert_eq!(paragraphs[1].style.heading, 0);
        assert_eq!(paragraphs[2].text(), "A new paragraph after a blank line.");
        assert!(doc.blocks.contains(&Block::PageBreak));
        let picture = doc
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Picture(p) => Some(p),
                _ => None,
            })
            .unwrap();
        assert!((picture.width - 320.0).abs() < 2.0, "{}", picture.width);

        let scan = document(&fixture("scan.pdf"), &Options::default()).unwrap();
        assert!(matches!(scan.blocks.as_slice(), [Block::Picture(_)]));
    }

    #[test]
    fn mixed_directions_come_out_in_reading_order() {
        if !super::super::test_library() {
            return;
        }
        let doc = Document {
            blocks: vec![
                Block::Paragraph(Paragraph::plain("Hello مرحبا بالعالم world")),
                Block::Paragraph(Paragraph::plain("مرحبا Hello big world بالعالم")),
                Block::Paragraph(Paragraph::plain("هذا تقرير ربع سنوي ٢٠٢٦")),
                Block::Paragraph(Paragraph::plain("لا تنسَ الخبز")),
            ],
            ..Document::default()
        };
        let bytes = super::super::write::document(&doc, &Options::default()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mixed.pdf");
        std::fs::write(&path, bytes).unwrap();
        let text = text(&path, &Options::default()).unwrap();
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            lines,
            [
                "Hello مرحبا بالعالم world",
                "مرحبا Hello big world بالعالم",
                "هذا تقرير ربع سنوي ٢٠٢٦",
                "لا تنسَ الخبز"
            ]
        );
    }
}
