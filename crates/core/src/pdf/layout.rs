//! Laying a [`Document`] out on pages: paragraphs wrapped into lines, lists, simple
//! tables and pictures, flowing from page to page. The result is a list of things to
//! draw on each page, which [`super::write`] turns into PDF.

use std::sync::Arc;

use crate::doc::{Align, Block, Document, Page, Paragraph, Picture, Table};
use crate::text::{self, Glyph, Style, Weight};

/// Gap between a list marker and its text.
const LIST_GAP: f32 = 18.0;
/// Space inside table cells.
const CELL_PADDING: f32 = 4.0;
const BORDER: f32 = 0.5;
const BORDER_COLOR: [u8; 3] = [128, 128, 128];

/// Something to draw. Coordinates are in points from the top left of the page; `y`
/// grows downwards.
#[derive(Debug, Clone)]
pub enum Item<'a> {
    Glyphs {
        x: f32,
        baseline: f32,
        glyphs: Vec<Glyph>,
        /// The text the glyphs' ranges refer to.
        text: Arc<str>,
        color: [u8; 3],
        /// Slanted, for italic text: the bundled fonts have no italic.
        italic: bool,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [u8; 3],
    },
    Picture {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        picture: &'a Picture,
    },
}

impl Item<'_> {
    fn shifted(mut self, dx: f32, dy: f32) -> Self {
        match &mut self {
            Item::Glyphs { x, baseline, .. } => {
                *x += dx;
                *baseline += dy;
            }
            Item::Rect { x, y, .. } | Item::Picture { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PageOut<'a> {
    pub width: f32,
    pub height: f32,
    pub items: Vec<Item<'a>>,
}

/// A laid out line, or any other unit that is not split across pages. Items are
/// relative to its top left corner.
#[derive(Debug, Clone, Default)]
struct Strip<'a> {
    height: f32,
    items: Vec<Item<'a>>,
}

/// Lays `doc` out on pages.
pub fn layout(doc: &Document) -> Vec<PageOut<'_>> {
    let mut flow = Flow::new(doc.page);
    for block in &doc.blocks {
        match block {
            Block::Paragraph(p) => {
                if p.style.page_break_before && !flow.fresh {
                    flow.new_page();
                }
                flow.space(p.style.space_before);
                let width = flow.page.content_width();
                for strip in paragraph(p, width) {
                    flow.place(strip);
                }
                flow.space(p.style.space_after);
            }
            Block::Table(t) => {
                let width = flow.page.content_width();
                for row in table(t, width) {
                    flow.place(row);
                }
                flow.space(6.0);
            }
            Block::Picture(p) => {
                let page = flow.page;
                let scale = (page.content_width() / p.width)
                    .min(page.content_height() / p.height)
                    .min(1.0);
                let (w, h) = (p.width * scale, p.height * scale);
                flow.place(Strip {
                    height: h,
                    items: vec![Item::Picture {
                        x: 0.0,
                        y: 0.0,
                        width: w,
                        height: h,
                        picture: p,
                    }],
                });
                flow.space(6.0);
            }
            Block::PageBreak => flow.new_page(),
        }
    }
    flow.finish()
}

struct Flow<'a> {
    page: Page,
    pages: Vec<PageOut<'a>>,
    current: PageOut<'a>,
    y: f32,
    /// Nothing has been placed on the current page yet.
    fresh: bool,
}

impl<'a> Flow<'a> {
    fn new(page: Page) -> Self {
        Flow {
            page,
            pages: Vec::new(),
            current: PageOut {
                width: page.width,
                height: page.height,
                items: Vec::new(),
            },
            y: page.margin_top,
            fresh: true,
        }
    }

    fn new_page(&mut self) {
        let done = std::mem::replace(
            &mut self.current,
            PageOut {
                width: self.page.width,
                height: self.page.height,
                items: Vec::new(),
            },
        );
        self.pages.push(done);
        self.y = self.page.margin_top;
        self.fresh = true;
    }

    fn space(&mut self, points: f32) {
        if !self.fresh {
            self.y += points.max(0.0);
        }
    }

    fn place(&mut self, strip: Strip<'a>) {
        let bottom = self.page.height - self.page.margin_bottom;
        if self.y + strip.height > bottom + 0.01 && !self.fresh {
            self.new_page();
        }
        let (dx, dy) = (self.page.margin_left, self.y);
        self.current
            .items
            .extend(strip.items.into_iter().map(|i| i.shifted(dx, dy)));
        self.y += strip.height;
        self.fresh = false;
    }

    fn finish(mut self) -> Vec<PageOut<'a>> {
        self.pages.push(self.current);
        self.pages
    }
}

/// The text of a span as layout takes it: tabs as spaces, no control characters other
/// than line breaks.
fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\t' => out.push_str("    "),
            '\n' => out.push('\n'),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// One line-broken piece of a paragraph and the spans its bytes come from.
struct Segment {
    text: String,
    styles: Vec<(std::ops::Range<usize>, usize)>,
}

fn segments(p: &Paragraph) -> Vec<Segment> {
    let mut out = vec![Segment {
        text: String::new(),
        styles: Vec::new(),
    }];
    for (i, span) in p.spans.iter().enumerate() {
        for (n, piece) in clean(&span.text).split('\n').enumerate() {
            if n > 0 {
                out.push(Segment {
                    text: String::new(),
                    styles: Vec::new(),
                });
            }
            if let Some(seg) = out.last_mut()
                && !piece.is_empty()
            {
                let start = seg.text.len();
                seg.text.push_str(piece);
                seg.styles.push((start..seg.text.len(), i));
            }
        }
    }
    out
}

fn line_metrics(glyphs: &[Glyph], base_size: f32, weight: Weight) -> (f32, f32) {
    if glyphs.is_empty() {
        let f = text::primary(weight);
        return (f.ascender() * base_size, -f.descender() * base_size);
    }
    glyphs.iter().fold((0.0f32, 0.0f32), |(a, d), g| {
        let f = text::font(g.font);
        (a.max(f.ascender() * g.size), d.max(-f.descender() * g.size))
    })
}

/// Lays out paragraph `p` in a column `width` points wide.
fn paragraph<'a>(p: &Paragraph, width: f32) -> Vec<Strip<'a>> {
    let style = &p.style;
    let rtl = p.is_rtl();
    let indent = if style.list.is_some() {
        style.indent.max(LIST_GAP)
    } else {
        style.indent
    }
    .min(width * 0.75);
    let available = (width - indent).max(36.0);
    let base_weight = if style.heading > 0 && p.spans.is_empty() {
        Weight::Bold
    } else {
        Weight::Regular
    };

    let mut strips = Vec::new();
    let segments = segments(p);
    for (n, segment) in segments.iter().enumerate() {
        let text: Arc<str> = Arc::from(segment.text.as_str());
        let styles: Vec<(std::ops::Range<usize>, Style)> = segment
            .styles
            .iter()
            .map(|(range, i)| {
                let span = &p.spans[*i];
                let style = Style {
                    weight: if span.bold {
                        Weight::Bold
                    } else {
                        Weight::Regular
                    },
                    size: span.size.unwrap_or(style.size),
                };
                (range.clone(), style)
            })
            .collect();
        let laid = text::layout_styled(&text, &styles, Some(available), Some(rtl));
        let count = laid.lines.len();
        for (k, line) in laid.lines.into_iter().enumerate() {
            let mut glyphs = line.glyphs;
            let last = k + 1 == count;
            let mut line_width = line.width;
            let free = (available - line_width).max(0.0);
            let offset = match style.align {
                Align::Center => free / 2.0,
                Align::End if rtl => 0.0,
                Align::End => free,
                Align::Start | Align::Justify if rtl => free,
                Align::Start | Align::Justify => 0.0,
            };
            let mut offset = offset;
            if style.align == Align::Justify && !last && free > 0.0 {
                let spaces: Vec<usize> = glyphs
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| &text[g.text.clone()] == " ")
                    .map(|(i, _)| i)
                    .collect();
                if !spaces.is_empty() {
                    let extra = free / spaces.len() as f32;
                    for i in spaces {
                        glyphs[i].advance += extra;
                    }
                    line_width = available;
                    offset = 0.0;
                }
            }
            let _ = line_width;
            let (ascent, descent) = line_metrics(&glyphs, style.size, base_weight);
            let x0 = if rtl { offset } else { indent + offset };
            let mut items = glyph_items(p, &segment.styles, glyphs, &text, x0, ascent);

            if n == 0
                && k == 0
                && let Some(marker) = &style.list
            {
                let marked = text::layout_styled(
                    marker,
                    &[(
                        0..marker.len(),
                        Style {
                            weight: Weight::Regular,
                            size: style.size,
                        },
                    )],
                    None,
                    Some(false),
                );
                if let Some(mline) = marked.lines.into_iter().next() {
                    let x = if rtl {
                        width - indent + LIST_GAP - mline.width
                    } else {
                        indent - LIST_GAP
                    };
                    items.push(Item::Glyphs {
                        x,
                        baseline: ascent,
                        glyphs: mline.glyphs,
                        text: Arc::from(marker.as_str()),
                        color: [0, 0, 0],
                        italic: false,
                    });
                }
            }
            strips.push(Strip {
                height: ascent + descent,
                items,
            });
        }
    }
    strips
}

/// Groups a line's glyphs into runs of one font, size and span, with the rules for
/// underlined and struck-through spans.
fn glyph_items<'a>(
    p: &Paragraph,
    styles: &[(std::ops::Range<usize>, usize)],
    glyphs: Vec<Glyph>,
    text: &Arc<str>,
    x0: f32,
    baseline: f32,
) -> Vec<Item<'a>> {
    let span_of = |g: &Glyph| styles.get(g.style).map(|(_, i)| *i);
    let mut items = Vec::new();
    let mut pen = x0;
    let mut group: Vec<Glyph> = Vec::new();
    let mut group_x = pen;
    let flush = |items: &mut Vec<Item<'a>>, group: &mut Vec<Glyph>, x: f32| {
        let Some(first) = group.first() else { return };
        let span = span_of(first).and_then(|i| p.spans.get(i));
        let width: f32 = group.iter().map(|g| g.advance).sum();
        let size = first.size;
        let color = span.and_then(|s| s.color).unwrap_or([0, 0, 0]);
        if let Some(s) = span {
            if s.underline {
                items.push(Item::Rect {
                    x,
                    y: baseline + size * 0.1,
                    width,
                    height: (size * 0.06).max(0.5),
                    color,
                });
            }
            if s.strike {
                items.push(Item::Rect {
                    x,
                    y: baseline - size * 0.3,
                    width,
                    height: (size * 0.06).max(0.5),
                    color,
                });
            }
        }
        items.push(Item::Glyphs {
            x,
            baseline,
            glyphs: std::mem::take(group),
            text: text.clone(),
            color,
            italic: span.is_some_and(|s| s.italic),
        });
    };
    for g in glyphs {
        let same = group
            .first()
            .is_some_and(|f| f.font == g.font && f.size == g.size && f.style == g.style);
        if !same {
            flush(&mut items, &mut group, group_x);
            group_x = pen;
        }
        pen += g.advance;
        group.push(g);
    }
    flush(&mut items, &mut group, group_x);
    items
}

fn table<'a>(t: &Table, width: f32) -> Vec<Strip<'a>> {
    let columns = t
        .rows
        .iter()
        .map(|r| r.iter().map(|c| usize::from(c.span.max(1))).sum::<usize>())
        .max()
        .unwrap_or(1)
        .max(1);
    let given: f32 = t.widths.iter().take(columns).sum();
    let widths: Vec<f32> = if t.widths.len() >= columns && given > 1.0 {
        t.widths
            .iter()
            .take(columns)
            .map(|w| w / given * width)
            .collect()
    } else {
        vec![width / columns as f32; columns]
    };

    let mut rows = Vec::new();
    for row in &t.rows {
        let mut cells = Vec::new();
        let mut x = 0.0;
        let mut column = 0usize;
        for cell in row {
            let span = usize::from(cell.span.max(1)).min(columns.saturating_sub(column).max(1));
            let w: f32 = widths.iter().skip(column).take(span).sum();
            column += span;
            let inner = (w - 2.0 * CELL_PADDING).max(12.0);
            let mut y = CELL_PADDING;
            let mut items = Vec::new();
            for (i, p) in cell.paragraphs.iter().enumerate() {
                if i > 0 {
                    y += p.style.space_before;
                }
                for strip in paragraph(p, inner) {
                    items.extend(
                        strip
                            .items
                            .into_iter()
                            .map(|it| it.shifted(x + CELL_PADDING, y)),
                    );
                    y += strip.height;
                }
                if i + 1 < cell.paragraphs.len() {
                    y += p.style.space_after;
                }
            }
            cells.push((x, w, y + CELL_PADDING, items));
            x += w;
        }
        let height = cells
            .iter()
            .map(|c| c.2)
            .fold(2.0 * CELL_PADDING + 8.0, f32::max);
        let mut items = Vec::new();
        for (x, w, _, cell_items) in cells {
            items.extend(cell_items);
            let rule = |x: f32, y: f32, width: f32, height: f32| Item::Rect {
                x,
                y,
                width,
                height,
                color: BORDER_COLOR,
            };
            items.push(rule(x, 0.0, w, BORDER));
            items.push(rule(x, height - BORDER, w, BORDER));
            items.push(rule(x, 0.0, BORDER, height));
            items.push(rule(x + w - BORDER, 0.0, BORDER, height));
        }
        rows.push(Strip { height, items });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{Cell, ParagraphStyle, Span};

    fn texts(page: &PageOut) -> Vec<String> {
        page.items
            .iter()
            .filter_map(|i| match i {
                Item::Glyphs { glyphs, text, .. } => {
                    let mut ranges: Vec<_> = glyphs.iter().map(|g| g.text.clone()).collect();
                    ranges.dedup();
                    Some(ranges.into_iter().map(|r| &text[r]).collect())
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn long_text_flows_onto_more_pages() {
        let words = "word ".repeat(400);
        let doc = Document {
            blocks: (0..10)
                .map(|_| Block::Paragraph(Paragraph::plain(words.clone())))
                .collect(),
            page: Page::default(),
        };
        let pages = layout(&doc);
        assert!(pages.len() > 3, "{}", pages.len());
        let page = Page::default();
        for p in &pages {
            for item in &p.items {
                if let Item::Glyphs { x, baseline, .. } = item {
                    assert!(*x >= page.margin_left - 0.01);
                    assert!(*baseline <= page.height - page.margin_bottom + 0.01);
                }
            }
        }
    }

    #[test]
    fn page_breaks_and_alignment() {
        let mut centred = Paragraph::plain("middle");
        centred.style.align = Align::Center;
        let doc = Document {
            blocks: vec![
                Block::Paragraph(Paragraph::plain("first")),
                Block::PageBreak,
                Block::Paragraph(centred),
                Block::Paragraph(Paragraph::plain("مرحبا")),
            ],
            page: Page::default(),
        };
        let pages = layout(&doc);
        assert_eq!(pages.len(), 2);
        assert_eq!(texts(&pages[0]), ["first"]);
        let xs: Vec<f32> = pages[1]
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Glyphs { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        // Centred text starts well inside the margin; Arabic hugs the right margin.
        assert!(xs[0] > 200.0, "{xs:?}");
        assert!(xs[1] > 400.0, "{xs:?}");
    }

    #[test]
    fn styles_lists_and_tables() {
        let mut item = Paragraph {
            spans: vec![
                Span::plain("normal "),
                Span {
                    bold: true,
                    underline: true,
                    size: Some(20.0),
                    ..Span::plain("big")
                },
            ],
            style: ParagraphStyle::default(),
        };
        item.style.list = Some("•".into());
        let doc = Document {
            blocks: vec![
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
                    widths: vec![1.0, 3.0],
                }),
            ],
            page: Page::default(),
        };
        let pages = layout(&doc);
        let t = texts(&pages[0]);
        assert_eq!(t, ["normal ", "big", "•", "a", "b"]);
        let rects = pages[0]
            .items
            .iter()
            .filter(|i| matches!(i, Item::Rect { .. }))
            .count();
        // The underline and four borders per cell.
        assert_eq!(rects, 1 + 8);
    }
}
