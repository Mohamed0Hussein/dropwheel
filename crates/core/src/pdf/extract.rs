//! Reading a PDF page's text from the positions of its characters.
//!
//! PDFium's own plain text puts right-to-left words in different orders depending on how
//! the PDF was made. Here every line is rebuilt the same way: its characters sorted
//! left to right as they appear on the page, then put back in reading order with the
//! Unicode bidirectional algorithm. Ligatures (several characters for one glyph) and
//! combining marks stay with their letters.

use pdfium_render::prelude::{
    PdfFontWeight, PdfPage, PdfPageObjectCommon, PdfPageObjectType, PdfPageObjectsCommon,
    PdfPageRenderRotation, PdfRect,
};
use unicode_bidi::BidiInfo;
use unicode_normalization::UnicodeNormalization;

/// A rectangle on the page as displayed: points from the top left, `y` growing down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Box2 {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Box2 {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    fn union(&self, other: &Box2) -> Box2 {
        Box2 {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    fn center_x(&self) -> f32 {
        (self.left + self.right) / 2.0
    }
}

/// How a run of characters looks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

/// One line of text in reading order, with where it sits.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    /// Byte ranges of `text` and how they look, in order.
    pub looks: Vec<(std::ops::Range<usize>, Look)>,
    pub bounds: Box2,
    /// The most common font size on the line.
    pub size: f32,
    pub rtl: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PageText {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<Line>,
    /// Pictures on the page, where they are shown.
    pub pictures: Vec<Box2>,
}

/// Maps user space to the displayed page, whatever its rotation.
struct Orientation {
    rotation: PdfPageRenderRotation,
    left: f32,
    bottom: f32,
    width: f32,
    height: f32,
}

impl Orientation {
    fn of(page: &PdfPage) -> Orientation {
        let media = page.page_size();
        Orientation {
            rotation: page.rotation().unwrap_or(PdfPageRenderRotation::None),
            left: media.left().value,
            bottom: media.bottom().value,
            width: media.width().value,
            height: media.height().value,
        }
    }

    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        let (u, v) = (x - self.left, y - self.bottom);
        match self.rotation {
            PdfPageRenderRotation::None => (u, self.height - v),
            PdfPageRenderRotation::Degrees90 => (v, u),
            PdfPageRenderRotation::Degrees180 => (self.width - u, v),
            PdfPageRenderRotation::Degrees270 => (self.height - v, self.width - u),
        }
    }

    fn rect(&self, r: &PdfRect) -> Box2 {
        let (x1, y1) = self.point(r.left().value, r.bottom().value);
        let (x2, y2) = self.point(r.right().value, r.top().value);
        Box2 {
            left: x1.min(x2),
            top: y1.min(y2),
            right: x1.max(x2),
            bottom: y1.max(y2),
        }
    }
}

/// A character as found on the page.
#[derive(Debug, Clone)]
struct Glyph {
    chars: String,
    bounds: Box2,
    /// Where the glyph's pen position is along the line, left to right as displayed.
    origin: f32,
    /// Which word it belongs to, as PDFium splits words.
    word: usize,
    look: Look,
    marks: String,
}

fn is_mark(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036F}'
        | '\u{0483}'..='\u{0489}'
        | '\u{0591}'..='\u{05BD}'
        | '\u{05BF}' | '\u{05C1}' | '\u{05C2}' | '\u{05C4}' | '\u{05C5}' | '\u{05C7}'
        | '\u{0610}'..='\u{061A}'
        | '\u{064B}'..='\u{065F}'
        | '\u{0670}'
        | '\u{06D6}'..='\u{06DC}'
        | '\u{06DF}'..='\u{06E4}'
        | '\u{06E7}' | '\u{06E8}'
        | '\u{06EA}'..='\u{06ED}'
        | '\u{08D3}'..='\u{08FF}'
        | '\u{20D0}'..='\u{20FF}'
        | '\u{FE20}'..='\u{FE2F}')
}

/// Letters some PDFs store as presentation forms or ligatures, as plain letters.
fn normalise(c: char, out: &mut String) {
    match c {
        '\u{FB00}'..='\u{FB06}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFE}' => {
            out.extend(std::iter::once(c).nfkc());
        }
        '\u{00A0}' => out.push(' '),
        c if c.is_control() || matches!(c, '\u{FEFF}' | '\u{FFFE}' | '\u{FFFF}') => {}
        c => out.push(c),
    }
}

/// The text, lines and pictures of `page`.
pub fn page_text(page: &PdfPage) -> PageText {
    let orientation = Orientation::of(page);
    let (width, height) = super::engine::page_size(page);
    let mut out = PageText {
        width,
        height,
        ..PageText::default()
    };

    for object in page.objects().iter() {
        if object.object_type() == PdfPageObjectType::Image
            && let Ok(quad) = object.bounds()
        {
            let b = orientation.rect(&quad.to_rect());
            if b.width() >= 24.0 && b.height() >= 24.0 {
                out.pictures.push(b);
            }
        }
    }

    let Ok(text) = page.text() else {
        return out;
    };
    // Characters as PDFium lists them. Its word breaks (spaces in the PDF, or ones it
    // infers from gaps) are reliable; its order within a line is not.
    let mut glyphs: Vec<Glyph> = Vec::new();
    let mut word = 0usize;
    for ch in text.chars().iter() {
        let Some(c) = ch.unicode_char() else { continue };
        if c.is_whitespace() {
            word += 1;
            continue;
        }
        if ch.is_generated().unwrap_or(false) {
            continue;
        }
        let Ok(rect) = ch.loose_bounds().or_else(|_| ch.tight_bounds()) else {
            continue;
        };
        let bounds = orientation.rect(&rect);
        let origin = match ch.origin() {
            Ok((x, y)) => orientation.point(x.value, y.value).0,
            Err(_) => bounds.left,
        };
        let font = ch.font_name().to_lowercase();
        let weight = match ch.font_weight() {
            Some(PdfFontWeight::Custom(w)) => w,
            Some(PdfFontWeight::Weight600) => 600,
            Some(PdfFontWeight::Weight700Bold) => 700,
            Some(PdfFontWeight::Weight800) => 800,
            Some(PdfFontWeight::Weight900) => 900,
            _ => 400,
        };
        let look = Look {
            size: (ch.scaled_font_size().value * 2.0).round() / 2.0,
            bold: weight >= 600
                || ["bold", "black", "heavy", "semibold", "demi"]
                    .iter()
                    .any(|w| font.contains(w)),
            italic: font.contains("italic") || font.contains("oblique"),
        };
        let mut chars = String::new();
        normalise(c, &mut chars);
        if chars.is_empty() {
            continue;
        }
        // Several characters from one glyph, such as a ligature, share its position.
        if let Some(last) = glyphs.last_mut()
            && last.bounds == bounds
            && last.origin == origin
            && bounds.width() > 0.0
        {
            last.chars.push_str(&chars);
            continue;
        }
        glyphs.push(Glyph {
            chars,
            bounds,
            origin,
            word,
            look,
            marks: String::new(),
        });
    }

    // Lines from where the characters sit: same height on the page, and not across a
    // wide gap, which separates columns. Lines keep the order the PDF draws them in.
    let mut lines: Vec<Vec<Glyph>> = Vec::new();
    let mut extents: Vec<Box2> = Vec::new();
    for g in glyphs {
        let size = g.look.size.max(1.0);
        let centre = (g.bounds.top + g.bounds.bottom) / 2.0;
        let mark = g.chars.chars().all(is_mark);
        let tolerance = if mark { 0.9 } else { 0.35 };
        let found = extents
            .iter()
            .enumerate()
            .rev()
            .take(8)
            .find(|(_, e)| {
                let line_centre = (e.top + e.bottom) / 2.0;
                let line_size = e.height().max(size);
                (centre - line_centre).abs() <= tolerance * line_size
                    && g.bounds.left <= e.right + 3.0 * size
                    && g.bounds.right >= e.left - 3.0 * size
            })
            .map(|(i, _)| i);
        match found {
            Some(i) => {
                if !mark {
                    extents[i] = extents[i].union(&g.bounds);
                }
                lines[i].push(g);
            }
            None => {
                extents.push(Box2 {
                    // A line's height is its letters', not its accents'.
                    top: centre - size * 0.4,
                    bottom: centre + size * 0.4,
                    ..g.bounds
                });
                lines.push(vec![g]);
            }
        }
    }

    out.lines = lines.into_iter().filter_map(build_line).collect();
    out
}

/// Sorts a line's glyphs into reading order and joins them into text.
fn build_line(glyphs: Vec<Glyph>) -> Option<Line> {
    // Combining marks ride on the letter they sit over.
    let mut bases: Vec<Glyph> = Vec::new();
    let mut marks: Vec<Glyph> = Vec::new();
    for g in glyphs {
        if g.chars.chars().all(is_mark) {
            marks.push(g);
        } else {
            bases.push(g);
        }
    }
    for mark in marks {
        let centre = mark.bounds.center_x();
        let nearest = bases
            .iter_mut()
            .filter(|g| !g.chars.trim().is_empty())
            .min_by(|a, b| {
                let da = (a.bounds.center_x() - centre).abs();
                let db = (b.bounds.center_x() - centre).abs();
                da.total_cmp(&db)
            });
        if let Some(base) = nearest {
            base.marks.push_str(&mark.chars);
        }
    }
    if bases.iter().all(|g| g.chars.trim().is_empty()) {
        return None;
    }

    // Left to right as seen: words by where they start, and the glyphs of each word by
    // where each is drawn from, which steps along the line even where the boxes of
    // joined Arabic letters overlap. Words are separated by one space.
    for g in &mut bases {
        fix_ligature(&mut g.chars);
    }
    let mut words: Vec<Vec<Glyph>> = Vec::new();
    for g in bases {
        match words.iter_mut().find(|w| w[0].word == g.word) {
            Some(w) => w.push(g),
            None => words.push(vec![g]),
        }
    }
    for w in &mut words {
        w.sort_by(|a, b| a.origin.total_cmp(&b.origin));
    }
    words.sort_by(|a, b| a[0].origin.total_cmp(&b[0].origin));
    let punctuation = |w: &[Glyph]| {
        w.iter().all(|g| {
            g.chars
                .chars()
                .all(|c| c.is_ascii_punctuation() || "،؛؟«»".contains(c))
        })
    };
    let mut visual: Vec<Glyph> = Vec::new();
    let mut previous_word: Vec<Glyph> = Vec::new();
    for w in words {
        let touching = visual.last().zip(w.first()).is_some_and(|(prev, first)| {
            let size = prev.look.size.max(first.look.size).max(1.0);
            (first.bounds.left - prev.bounds.right) < 0.08 * size
                && (punctuation(&w) || punctuation(&previous_word))
        });
        if let (Some(prev), Some(first)) = (visual.last(), w.first())
            && !touching
        {
            visual.push(Glyph {
                chars: " ".into(),
                bounds: Box2 {
                    left: prev.bounds.right,
                    right: first.bounds.left.max(prev.bounds.right),
                    ..prev.bounds
                },
                origin: prev.bounds.right,
                word: usize::MAX,
                look: prev.look,
                marks: String::new(),
            });
        }
        previous_word = w.clone();
        visual.extend(w);
    }

    // Reading order: laying out the visual order again with the bidirectional algorithm
    // turns it back into reading order, for lines that nest one direction in another
    // (the usual case). One character stands for each glyph.
    let probe: String = visual
        .iter()
        .map(|g| g.chars.chars().next().unwrap_or(' '))
        .collect();
    // The line's direction: that of the letters at both its ends, or, when they
    // differ, the one most of its letters have.
    let strong = |c: &char| c.is_alphabetic();
    let first = probe.chars().find(strong).is_some_and(is_rtl_letter);
    let last = probe.chars().rev().find(strong).is_some_and(is_rtl_letter);
    let rtl = if first == last {
        first
    } else {
        let rtl_letters = probe.chars().filter(|c| is_rtl_letter(*c)).count();
        let letters = probe.chars().filter(strong).count();
        rtl_letters * 2 > letters
    };
    let level = if rtl {
        unicode_bidi::Level::rtl()
    } else {
        unicode_bidi::Level::ltr()
    };
    let bidi = BidiInfo::new(&probe, Some(level));
    let mut order: Vec<usize> = Vec::with_capacity(visual.len());
    let byte_to_index: std::collections::HashMap<usize, usize> = probe
        .char_indices()
        .enumerate()
        .map(|(i, (b, _))| (b, i))
        .collect();
    for para in &bidi.paragraphs {
        let (levels, runs) = bidi.visual_runs(para, para.range.clone());
        for run in runs {
            let mut indices: Vec<usize> = probe[run.clone()]
                .char_indices()
                .filter_map(|(b, _)| byte_to_index.get(&(run.start + b)).copied())
                .collect();
            if levels.get(run.start).is_some_and(|l| l.is_rtl()) {
                indices.reverse();
            }
            order.extend(indices);
        }
    }

    let mut text = String::new();
    let mut looks: Vec<(std::ops::Range<usize>, Look)> = Vec::new();
    let mut bounds: Option<Box2> = None;
    let mut sizes: Vec<f32> = Vec::new();
    for i in order {
        let Some(g) = visual.get(i) else { continue };
        let start = text.len();
        text.push_str(&g.chars);
        text.push_str(&g.marks);
        let end = text.len();
        match looks.last_mut() {
            Some((range, look)) if *look == g.look || g.chars == " " => range.end = end,
            _ => looks.push((start..end, g.look)),
        }
        if g.chars != " " {
            bounds = Some(bounds.map_or(g.bounds, |b| b.union(&g.bounds)));
            sizes.push(g.look.size);
        }
    }
    let trimmed_end = text.trim_end().len();
    let trimmed_start = text.len() - text.trim_start().len();
    if trimmed_start > 0 || trimmed_end < text.len() {
        let t = text[trimmed_start..trimmed_end].to_string();
        looks = looks
            .into_iter()
            .filter_map(|(r, l)| {
                let s = r.start.max(trimmed_start).min(trimmed_end) - trimmed_start;
                let e = r.end.max(trimmed_start).min(trimmed_end) - trimmed_start;
                (e > s).then_some((s..e, l))
            })
            .collect();
        text = t;
    }
    sizes.sort_by(f32::total_cmp);
    let size = sizes.get(sizes.len() / 2).copied().unwrap_or(10.0);
    Some(Line {
        text,
        looks,
        bounds: bounds?,
        size,
        rtl,
    })
}

/// PDFium lists the letters of a right-to-left ligature, such as lam-alef, backwards.
fn fix_ligature(chars: &mut String) {
    if chars.chars().count() > 1 && chars.chars().all(is_rtl_letter) {
        *chars = chars.chars().rev().collect();
    }
}

fn is_rtl_letter(c: char) -> bool {
    matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFC}')
        && c.is_alphabetic()
}
