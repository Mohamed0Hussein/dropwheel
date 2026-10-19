//! Fonts, shaping and line layout, shared by annotations drawn into images and by the
//! PDFs Dropwheel writes.
//!
//! Text is set in the bundled Noto Sans and Noto Sans Arabic, falling back to fonts that
//! ship with Windows for other scripts. Each paragraph takes its direction from its first
//! strong character, as with `unicode-bidi: plaintext` in CSS, so a dialog can preview
//! text the way it will be drawn.

use std::ops::Range;
use std::sync::OnceLock;

use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};
use rustybuzz::{Direction, UnicodeBuffer};
use unicode_bidi::ParagraphBidiInfo;
use unicode_linebreak::linebreaks;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Weight {
    #[default]
    Regular,
    Bold,
}

/// A parsed font file.
pub struct Font {
    pub data: &'static [u8],
    /// The face within a font collection.
    pub index: u32,
    face: rustybuzz::Face<'static>,
    units_per_em: f32,
}

impl Font {
    fn parse(data: &'static [u8], index: u32) -> Option<Font> {
        let face = rustybuzz::Face::from_slice(data, index)?;
        let units_per_em = (face.units_per_em() as f32).max(1.0);
        Some(Font {
            data,
            index,
            face,
            units_per_em,
        })
    }

    pub fn has(&self, c: char) -> bool {
        self.face.glyph_index(c).is_some()
    }

    /// Distance from the baseline to the top of the line box, in ems.
    pub fn ascender(&self) -> f32 {
        f32::from(self.face.ascender()) / self.units_per_em
    }

    /// Distance from the baseline to the bottom of the line box, in ems; negative.
    pub fn descender(&self) -> f32 {
        f32::from(self.face.descender()) / self.units_per_em
    }

    /// Traces `glyph` at `size` with its origin at `x`, `baseline`, y growing downwards.
    /// False for glyphs without an outline, such as spaces.
    pub fn outline(
        &self,
        glyph: u16,
        size: f32,
        x: f32,
        baseline: f32,
        sink: &mut dyn PathSink,
    ) -> bool {
        let mut adapter = Adapter {
            sink,
            scale: size / self.units_per_em,
            x,
            y: baseline,
        };
        self.face
            .outline_glyph(GlyphId(glyph), &mut adapter)
            .is_some()
    }
}

/// Receives glyph outlines in drawing coordinates.
pub trait PathSink {
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    fn close(&mut self);
}

struct Adapter<'a> {
    sink: &'a mut dyn PathSink,
    scale: f32,
    x: f32,
    y: f32,
}

impl Adapter<'_> {
    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        (self.x + x * self.scale, self.y - y * self.scale)
    }
}

impl OutlineBuilder for Adapter<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.sink.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.sink.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x, y) = self.map(x, y);
        self.sink.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x2, y2) = self.map(x2, y2);
        let (x, y) = self.map(x, y);
        self.sink.curve_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.sink.close();
    }
}

/// Identifies a font in [`font`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(u8);

enum Source {
    Bundled(&'static [u8]),
    /// A file in the Windows fonts folder, and the face within it.
    #[cfg_attr(not(windows), allow(dead_code))]
    System(&'static str, u32),
}

const LATIN_REGULAR: FontId = FontId(0);
const LATIN_BOLD: FontId = FontId(1);
const ARABIC_REGULAR: FontId = FontId(2);
const ARABIC_BOLD: FontId = FontId(3);
const FIRST_SYSTEM: u8 = 4;

/// Bundled fonts first, then Windows fonts for scripts Noto Sans does not cover, in the
/// order they are tried.
static SOURCES: [Source; 15] = [
    Source::Bundled(include_bytes!("../fonts/NotoSans-Regular.ttf")),
    Source::Bundled(include_bytes!("../fonts/NotoSans-Bold.ttf")),
    Source::Bundled(include_bytes!("../fonts/NotoSansArabic-Regular.ttf")),
    Source::Bundled(include_bytes!("../fonts/NotoSansArabic-Bold.ttf")),
    // Hebrew, Armenian, Georgian and more.
    Source::System("segoeui.ttf", 0),
    // Chinese, with Japanese kana.
    Source::System("msyh.ttc", 0),
    Source::System("YuGothM.ttc", 0),
    // Korean.
    Source::System("malgun.ttf", 0),
    // Indian scripts.
    Source::System("Nirmala.ttc", 0),
    Source::System("Nirmala.ttf", 0),
    // Thai, Lao, Khmer.
    Source::System("LeelawUI.ttf", 0),
    // Ethiopic and others.
    Source::System("ebrima.ttf", 0),
    Source::System("seguisym.ttf", 0),
    Source::System("seguihis.ttf", 0),
    Source::System("arial.ttf", 0),
];

static LOADED: [OnceLock<Option<Font>>; 15] = [const { OnceLock::new() }; 15];

fn load(id: FontId) -> Option<&'static Font> {
    let i = usize::from(id.0);
    LOADED
        .get(i)?
        .get_or_init(|| match SOURCES.get(i)? {
            Source::Bundled(data) => Font::parse(data, 0),
            Source::System(name, index) => load_system(name, *index),
        })
        .as_ref()
}

#[cfg(windows)]
fn load_system(name: &str, index: u32) -> Option<Font> {
    let dir = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
    let path = std::path::Path::new(&dir).join("Fonts").join(name);
    let data = std::fs::read(path).ok()?;
    // Loaded once per process and kept for its lifetime.
    let data: &'static [u8] = Box::leak(data.into_boxed_slice());
    Font::parse(data, index)
}

#[cfg(not(windows))]
fn load_system(_name: &str, _index: u32) -> Option<Font> {
    None
}

/// The font `id` stands for. Ids come from layouts, so they always resolve.
pub fn font(id: FontId) -> &'static Font {
    match load(id) {
        Some(f) => f,
        None => primary(Weight::Regular),
    }
}

/// The font whose metrics set the line height.
pub fn primary(weight: Weight) -> &'static Font {
    let id = match weight {
        Weight::Regular => LATIN_REGULAR,
        Weight::Bold => LATIN_BOLD,
    };
    #[allow(clippy::expect_used)]
    load(id).expect("the bundled fonts parse")
}

fn chain(weight: Weight) -> impl Iterator<Item = FontId> {
    let bundled = match weight {
        Weight::Regular => [LATIN_REGULAR, ARABIC_REGULAR],
        Weight::Bold => [LATIN_BOLD, ARABIC_BOLD],
    };
    bundled
        .into_iter()
        .chain((FIRST_SYSTEM..SOURCES.len() as u8).map(FontId))
}

/// Characters that should stay in the font of the text around them: spaces,
/// punctuation, digits and combining marks. Switching fonts mid-word would break the
/// joining of Arabic letters.
fn sticky(c: char) -> bool {
    !c.is_alphabetic()
        || matches!(c,
            '\u{0300}'..='\u{036F}'
            | '\u{0483}'..='\u{0489}'
            | '\u{0591}'..='\u{05C7}'
            | '\u{0610}'..='\u{061A}'
            | '\u{064B}'..='\u{065F}'
            | '\u{0670}'
            | '\u{06D6}'..='\u{06ED}'
            | '\u{08D3}'..='\u{08FF}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}')
}

fn pick(c: char, weight: Weight, previous: Option<FontId>) -> FontId {
    if let Some(p) = previous
        && sticky(c)
        && load(p).is_some_and(|f| f.has(c))
    {
        return p;
    }
    chain(weight)
        .find(|id| load(*id).is_some_and(|f| f.has(c)))
        .unwrap_or_else(|| previous.unwrap_or(chain_start(weight)))
}

fn chain_start(weight: Weight) -> FontId {
    match weight {
        Weight::Regular => LATIN_REGULAR,
        Weight::Bold => LATIN_BOLD,
    }
}

/// One shaped glyph. Lengths are in ems: multiply by the font size.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub font: FontId,
    pub id: u16,
    /// The characters this glyph shows, as byte offsets into the paragraph.
    pub text: Range<usize>,
    pub advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

/// One line of a paragraph.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// In drawing order, left to right.
    pub glyphs: Vec<Glyph>,
    /// In ems.
    pub width: f32,
    /// Byte range of the paragraph shown on this line.
    pub text: Range<usize>,
}

/// A paragraph laid out into lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub lines: Vec<Line>,
    /// Right to left: lines align to the right.
    pub rtl: bool,
}

struct Run {
    range: Range<usize>,
    rtl: bool,
    glyphs: Vec<Glyph>,
}

/// Lays out `text`, one paragraph with no line breaks in it, wrapping lines at
/// `max_width` ems when given. Tabs and other control characters should be replaced
/// beforehand.
pub fn layout(text: &str, weight: Weight, max_width: Option<f32>) -> Paragraph {
    let bidi = ParagraphBidiInfo::new(text, None);
    let rtl = bidi.paragraph_level.is_rtl();
    let runs = shape_runs(text, &bidi, weight);

    // Advance of each cluster, indexed by the byte where it starts.
    let mut advance = vec![0.0f32; text.len() + 1];
    let mut cluster_start = vec![false; text.len() + 1];
    for g in runs.iter().flat_map(|r| &r.glyphs) {
        advance[g.text.start] += g.advance;
        cluster_start[g.text.start] = true;
    }
    cluster_start[text.len()] = true;
    let mut prefix = Vec::with_capacity(advance.len() + 1);
    prefix.push(0.0f32);
    for a in &advance {
        prefix.push(prefix.last().copied().unwrap_or(0.0) + a);
    }
    let width = |r: &Range<usize>| prefix[r.end] - prefix[r.start];
    let trim = |r: Range<usize>| {
        let end = r.start + text[r.clone()].trim_end().len();
        r.start..end
    };

    let ranges = match max_width {
        Some(max) => break_lines(text, max, &width, &trim, &cluster_start),
        None => std::iter::once(0..text.len()).collect(),
    };

    let lines = ranges
        .into_iter()
        .map(|r| {
            let r = trim(r);
            let glyphs = visual_order(&bidi, &runs, r.clone());
            Line {
                width: glyphs.iter().map(|g| g.advance).sum(),
                glyphs,
                text: r,
            }
        })
        .collect();
    Paragraph { lines, rtl }
}

fn shape_runs(text: &str, bidi: &ParagraphBidiInfo, weight: Weight) -> Vec<Run> {
    let mut items: Vec<(Range<usize>, bool, FontId)> = Vec::new();
    let mut previous = None;
    for (i, c) in text.char_indices() {
        let rtl = bidi.levels.get(i).is_some_and(|l| l.is_rtl());
        let font = pick(c, weight, previous);
        previous = Some(font);
        let end = i + c.len_utf8();
        match items.last_mut() {
            Some((range, r, f)) if *r == rtl && *f == font => range.end = end,
            _ => items.push((i..end, rtl, font)),
        }
    }
    items
        .into_iter()
        .map(|(range, rtl, font_id)| {
            let glyphs = shape(text, range.clone(), rtl, font_id);
            Run { range, rtl, glyphs }
        })
        .collect()
}

fn shape(text: &str, range: Range<usize>, rtl: bool, font_id: FontId) -> Vec<Glyph> {
    let f = font(font_id);
    let mut buffer = UnicodeBuffer::new();
    buffer.set_pre_context(&text[..range.start]);
    buffer.set_post_context(&text[range.end..]);
    buffer.push_str(&text[range.clone()]);
    buffer.set_direction(if rtl {
        Direction::RightToLeft
    } else {
        Direction::LeftToRight
    });
    buffer.guess_segment_properties();
    let shaped = rustybuzz::shape(&f.face, &[], buffer);

    let mut starts: Vec<usize> = shaped
        .glyph_infos()
        .iter()
        .map(|i| i.cluster as usize)
        .collect();
    starts.sort_unstable();
    starts.dedup();
    let len = range.len();
    let em = f.units_per_em;
    shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions())
        .map(|(info, pos)| {
            let start = (info.cluster as usize).min(len);
            let next = starts.partition_point(|s| *s <= start);
            let end = starts.get(next).copied().unwrap_or(len);
            Glyph {
                font: font_id,
                id: u16::try_from(info.glyph_id).unwrap_or(0),
                text: range.start + start..range.start + end,
                advance: pos.x_advance as f32 / em,
                x_offset: pos.x_offset as f32 / em,
                y_offset: pos.y_offset as f32 / em,
            }
        })
        .collect()
}

/// Greedy line breaking at the opportunities Unicode allows, splitting words that are
/// wider than a whole line between clusters.
fn break_lines(
    text: &str,
    max: f32,
    width: &dyn Fn(&Range<usize>) -> f32,
    trim: &dyn Fn(Range<usize>) -> Range<usize>,
    cluster_start: &[bool],
) -> Vec<Range<usize>> {
    let fits = |r: Range<usize>| width(&trim(r)) <= max;
    let mut lines = Vec::new();
    let mut start = 0;
    let mut last_fit: Option<usize> = None;
    for (pos, _) in linebreaks(text) {
        if pos == 0 {
            continue;
        }
        if !fits(start..pos) {
            if let Some(b) = last_fit.take() {
                lines.push(start..b);
                start = b;
            }
            while !fits(start..pos) {
                let cut = widest_fit(start, pos, max, width, cluster_start);
                lines.push(start..cut);
                start = cut;
            }
        }
        last_fit = Some(pos);
    }
    if start < text.len() || lines.is_empty() {
        lines.push(start..text.len());
    }
    lines
}

/// The furthest cluster boundary after `start` that keeps the line within `max`, and
/// at least one cluster on.
fn widest_fit(
    start: usize,
    end: usize,
    max: f32,
    width: &dyn Fn(&Range<usize>) -> f32,
    cluster_start: &[bool],
) -> usize {
    let mut best = None;
    for cut in start + 1..=end {
        if !cluster_start.get(cut).copied().unwrap_or(false) {
            continue;
        }
        if best.is_none() || width(&(start..cut)) <= max {
            best = Some(cut);
        } else {
            break;
        }
    }
    best.unwrap_or(end)
}

fn visual_order(bidi: &ParagraphBidiInfo, runs: &[Run], line: Range<usize>) -> Vec<Glyph> {
    if line.is_empty() {
        return Vec::new();
    }
    let (levels, visual_runs) = bidi.visual_runs(line);
    let mut glyphs = Vec::new();
    for vr in visual_runs {
        let rtl = levels.get(vr.start).is_some_and(|l| l.is_rtl());
        let touching = runs
            .iter()
            .filter(|r| r.range.start < vr.end && vr.start < r.range.end);
        let ordered: Vec<&Run> = if rtl {
            touching.rev().collect()
        } else {
            touching.collect()
        };
        for run in ordered {
            // A run shaped in the other direction lands here only for characters the
            // bidi algorithm reset at the line's edge; keep its glyphs' own order.
            let in_line = run.glyphs.iter().filter(|g| vr.contains(&g.text.start));
            if run.rtl == rtl || !rtl {
                glyphs.extend(in_line.cloned());
            } else {
                glyphs.extend(in_line.rev().cloned());
            }
        }
    }
    glyphs
}

/// Lines of text separated by `\n`, as the Annotate tool and plain-text documents have
/// them. Tabs become four spaces and other control characters are dropped.
pub fn clean_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|line| {
            line.chars()
                .flat_map(|c| match c {
                    '\t' => "    ".chars().collect::<Vec<_>>(),
                    c if c.is_control() => Vec::new(),
                    c => vec![c],
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text of a line in drawing order. A cluster can have several glyphs, such as a
    /// letter and its dots.
    fn letters(p: &Paragraph, line: usize, text: &str) -> String {
        let mut ranges: Vec<Range<usize>> = p.lines[line]
            .glyphs
            .iter()
            .map(|g| g.text.clone())
            .collect();
        ranges.dedup();
        ranges.into_iter().map(|r| &text[r]).collect()
    }

    #[test]
    fn bundled_fonts_cover_latin_and_arabic() {
        assert!(primary(Weight::Regular).has('A'));
        assert!(primary(Weight::Bold).has('é'));
        assert!(font(ARABIC_REGULAR).has('ب'));
        assert!(font(ARABIC_BOLD).has('ب'));
        assert!(primary(Weight::Regular).ascender() > 0.5);
        assert!(primary(Weight::Regular).descender() < 0.0);
    }

    #[test]
    fn latin_lays_out_left_to_right() {
        let p = layout("Hello", Weight::Regular, None);
        assert!(!p.rtl);
        assert_eq!(p.lines.len(), 1);
        assert_eq!(letters(&p, 0, "Hello"), "Hello");
        assert!(p.lines[0].width > 2.0 && p.lines[0].width < 3.5);
        assert!(p.lines[0].glyphs.iter().all(|g| g.font == LATIN_REGULAR));
    }

    #[test]
    fn arabic_is_shaped_and_reversed() {
        let text = "مرحبا بالعالم";
        let p = layout(text, Weight::Regular, None);
        assert!(p.rtl);
        let line = &p.lines[0];
        assert!(line.glyphs.iter().all(|g| g.font == ARABIC_REGULAR));
        // Drawn from the end of the text on the left to its start on the right.
        assert_eq!(
            line.glyphs.first().map(|g| g.text.start),
            Some(text.len() - "م".len())
        );
        assert_eq!(line.glyphs.last().map(|g| g.text.start), Some(0));
        // Joined letters use contextual forms, so the same letter gets different glyphs.
        let mims: Vec<u16> = line
            .glyphs
            .iter()
            .filter(|g| &text[g.text.clone()] == "م")
            .map(|g| g.id)
            .collect();
        assert_eq!(mims.len(), 2);
        assert_ne!(mims[0], mims[1]);
    }

    #[test]
    fn mixed_direction_keeps_runs_in_reading_order() {
        let text = "Dropwheel يعمل";
        let p = layout(text, Weight::Regular, None);
        assert!(!p.rtl);
        let order = letters(&p, 0, text);
        assert!(order.starts_with("Dropwheel"), "{order}");
        // The Arabic word is reversed for display.
        assert!(order.ends_with("لمعي"), "{order}");
    }

    #[test]
    fn wraps_at_spaces_and_splits_long_words() {
        let text = "one two three four five six";
        let p = layout(text, Weight::Regular, Some(5.0));
        assert!(p.lines.len() > 1);
        for line in &p.lines {
            assert!(line.width <= 5.0 + 1e-3, "{line:?}");
            assert!(!text[line.text.clone()].ends_with(' '));
        }
        let rebuilt: Vec<&str> = p.lines.iter().map(|l| &text[l.text.clone()]).collect();
        assert_eq!(rebuilt.join(" "), text);

        let long = "a".repeat(200);
        let p = layout(&long, Weight::Regular, Some(10.0));
        assert!(p.lines.len() > 3);
        assert_eq!(p.lines.iter().map(|l| l.text.len()).sum::<usize>(), 200);
    }

    #[test]
    fn empty_text_has_one_empty_line() {
        let p = layout("", Weight::Bold, Some(10.0));
        assert_eq!(p.lines.len(), 1);
        assert!(p.lines[0].glyphs.is_empty());
    }

    #[test]
    fn unknown_characters_do_not_fail() {
        // No bundled font covers CJK; on other systems it may fall back to nothing.
        let p = layout("漢字 ok", Weight::Regular, None);
        assert!(!p.lines[0].glyphs.is_empty());
    }

    #[test]
    fn control_characters_are_cleaned() {
        assert_eq!(clean_lines("a\tb\r\nc\u{7}"), ["a    b", "c"]);
    }

    #[test]
    fn outlines_reach_the_sink() {
        struct Count(usize);
        impl PathSink for Count {
            fn move_to(&mut self, _: f32, _: f32) {
                self.0 += 1;
            }
            fn line_to(&mut self, _: f32, _: f32) {}
            fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
            fn close(&mut self) {}
        }
        let p = layout("O", Weight::Regular, None);
        let g = &p.lines[0].glyphs[0];
        let mut sink = Count(0);
        assert!(font(g.font).outline(g.id, 20.0, 0.0, 20.0, &mut sink));
        // An O has an outer and an inner contour.
        assert_eq!(sink.0, 2);
    }
}
