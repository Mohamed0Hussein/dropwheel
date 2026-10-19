//! Reading a DOCX into a [`Document`].
//!
//! Covers what carries meaning when a document is turned into text or a PDF: paragraph
//! and character styles, headings, bold, italic, underline, colours and sizes, bullets
//! and numbering, tables, pictures, text boxes, page breaks and right-to-left paragraphs.
//! Headers, footers, footnotes and comments are left out.

use std::collections::HashMap;
use std::path::Path;

use super::{Package, REL_NUMBERING, REL_STYLES, Rel, by_id};
use crate::doc::{
    Align, Block, Cell, Document, Page, Paragraph, ParagraphStyle, Picture, Span, Table,
};
use crate::error::Result;
use crate::xml::Element;

/// Points per twip, the unit of most OOXML lengths.
const TWIP: f32 = 1.0 / 20.0;
/// Points per EMU, the unit of drawing sizes.
const EMU: f32 = 1.0 / 12_700.0;

pub fn read(path: &Path) -> Result<Document> {
    let mut package = Package::open(path)?;
    let main = package.main_part()?;
    let root = package
        .xml(&main)?
        .ok_or_else(|| package.error("its main document is missing"))?;
    let rels = package.rels(&main)?;
    let styles = match rels.iter().find(|r| r.kind == REL_STYLES) {
        Some(r) => package.xml(&r.target)?.map(|x| Styles::parse(&x)),
        None => None,
    }
    .unwrap_or_default();
    let numbering = match rels.iter().find(|r| r.kind == REL_NUMBERING) {
        Some(r) => package.xml(&r.target)?.map(|x| Numbering::parse(&x)),
        None => None,
    }
    .unwrap_or_default();
    let body = root
        .child("body")
        .ok_or_else(|| package.error("the document has no body"))?;

    let mut reader = Reader {
        package: &mut package,
        rels: by_id(&rels),
        styles,
        numbering,
        counters: HashMap::new(),
    };
    let mut blocks = Vec::new();
    reader.blocks(body, &mut blocks);
    Ok(Document {
        blocks,
        page: body.child("sectPr").map(page).unwrap_or_default(),
    })
}

fn twips(value: Option<&str>) -> Option<f32> {
    value
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v * TWIP)
}

fn page(sect: &Element) -> Page {
    let mut page = Page::default();
    if let Some(size) = sect.child("pgSz") {
        if let Some(w) = twips(size.attr("w")) {
            page.width = w.clamp(72.0, 14_400.0);
        }
        if let Some(h) = twips(size.attr("h")) {
            page.height = h.clamp(72.0, 14_400.0);
        }
    }
    if let Some(m) = sect.child("pgMar") {
        let side = |name: &str, current: f32, limit: f32| {
            twips(m.attr(name)).map_or(current, |v| v.abs().min(limit))
        };
        page.margin_top = side("top", page.margin_top, page.height / 3.0);
        page.margin_bottom = side("bottom", page.margin_bottom, page.height / 3.0);
        page.margin_left = side("left", page.margin_left, page.width / 3.0);
        page.margin_right = side("right", page.margin_right, page.width / 3.0);
    }
    page
}

/// Character formatting; `None` leaves what is inherited.
#[derive(Debug, Clone, Default)]
struct RunProps {
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    hidden: Option<bool>,
    size: Option<f32>,
    color: Option<Option<[u8; 3]>>,
}

impl RunProps {
    fn parse(rpr: &Element) -> RunProps {
        let half_points = |name: &str| {
            rpr.val(name)
                .and_then(|v| v.parse::<f32>().ok())
                .filter(|v| v.is_finite() && *v > 0.0)
                .map(|v| (v / 2.0).clamp(1.0, 1638.0))
        };
        RunProps {
            bold: rpr.flag("b").or(rpr.flag("bCs")),
            italic: rpr.flag("i").or(rpr.flag("iCs")),
            underline: rpr.child("u").map(|u| u.attr("val") != Some("none")),
            strike: rpr.flag("strike").or(rpr.flag("dstrike")),
            hidden: rpr.flag("vanish"),
            size: half_points("sz").or_else(|| half_points("szCs")),
            color: rpr.val("color").map(parse_color),
        }
    }

    fn apply(&mut self, over: &RunProps) {
        macro_rules! take {
            ($($f:ident),*) => { $( if over.$f.is_some() { self.$f = over.$f; } )* };
        }
        take!(bold, italic, underline, strike, hidden, size, color);
    }

    fn span(&self, text: String) -> Span {
        Span {
            text,
            bold: self.bold.unwrap_or(false),
            italic: self.italic.unwrap_or(false),
            underline: self.underline.unwrap_or(false),
            strike: self.strike.unwrap_or(false),
            size: self.size,
            color: self.color.flatten(),
        }
    }
}

fn parse_color(value: &str) -> Option<[u8; 3]> {
    if value.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(value, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

/// Paragraph formatting; `None` leaves what is inherited.
#[derive(Debug, Clone, Default)]
struct ParaProps {
    align: Option<Align>,
    rtl: Option<bool>,
    /// Numbering instance and level. Instance "0" turns inherited numbering off.
    num: Option<(String, u8)>,
    indent: Option<f32>,
    before: Option<f32>,
    after: Option<f32>,
    break_before: Option<bool>,
    outline: Option<u8>,
}

impl ParaProps {
    fn parse(ppr: &Element) -> ParaProps {
        let ind = ppr.child("ind");
        let spacing = ppr.child("spacing");
        ParaProps {
            align: ppr.val("jc").map(|v| match v {
                "center" => Align::Center,
                "right" | "end" => Align::End,
                "both" | "distribute" | "lowKashida" | "mediumKashida" | "highKashida"
                | "thaiDistribute" => Align::Justify,
                _ => Align::Start,
            }),
            rtl: ppr.flag("bidi"),
            num: ppr.child("numPr").and_then(|n| {
                let id = n.val("numId")?.to_string();
                let level = n.val("ilvl").and_then(|l| l.parse().ok()).unwrap_or(0u8);
                Some((id, level.min(8)))
            }),
            indent: ind.and_then(|i| twips(i.attr("start").or(i.attr("left")))),
            before: spacing.and_then(|s| twips(s.attr("before"))),
            after: spacing.and_then(|s| twips(s.attr("after"))),
            break_before: ppr.flag("pageBreakBefore"),
            outline: ppr
                .val("outlineLvl")
                .and_then(|v| v.parse::<u8>().ok())
                .filter(|v| *v < 9),
        }
    }

    fn apply(&mut self, over: &ParaProps) {
        macro_rules! take {
            ($($f:ident),*) => { $( if over.$f.is_some() { self.$f = over.$f.clone(); } )* };
        }
        take!(
            align,
            rtl,
            num,
            indent,
            before,
            after,
            break_before,
            outline
        );
    }
}

#[derive(Debug, Clone, Default)]
struct Style {
    name: String,
    based_on: Option<String>,
    para: ParaProps,
    run: RunProps,
}

#[derive(Debug, Default)]
struct Styles {
    styles: HashMap<String, Style>,
    default_paragraph: Option<String>,
    run: RunProps,
    para: ParaProps,
}

impl Styles {
    fn parse(root: &Element) -> Styles {
        let mut styles = Styles::default();
        if let Some(defaults) = root.child("docDefaults") {
            if let Some(rpr) = defaults.child("rPrDefault").and_then(|d| d.child("rPr")) {
                styles.run = RunProps::parse(rpr);
            }
            if let Some(ppr) = defaults.child("pPrDefault").and_then(|d| d.child("pPr")) {
                styles.para = ParaProps::parse(ppr);
            }
        }
        for s in root.children_named("style") {
            let Some(id) = s.attr("styleId") else {
                continue;
            };
            if s.attr("type") == Some("paragraph")
                && matches!(s.attr("default"), Some("1" | "true"))
            {
                styles.default_paragraph = Some(id.to_string());
            }
            styles.styles.insert(
                id.to_string(),
                Style {
                    name: s.val("name").unwrap_or(id).to_lowercase(),
                    based_on: s.val("basedOn").map(str::to_string),
                    para: s.child("pPr").map(ParaProps::parse).unwrap_or_default(),
                    run: s.child("rPr").map(RunProps::parse).unwrap_or_default(),
                },
            );
        }
        styles
    }

    /// `id` and the styles it is based on, the most basic first.
    fn chain(&self, id: &str) -> Vec<&Style> {
        let mut chain = Vec::new();
        let mut next = Some(id);
        while let Some(id) = next {
            let Some(style) = self.styles.get(id) else {
                break;
            };
            if chain.len() > 16 {
                break;
            }
            chain.push(style);
            next = style.based_on.as_deref();
        }
        chain.reverse();
        chain
    }

    /// Formatting of a paragraph in style `id`, and its heading level (0 for body text).
    fn paragraph(&self, id: Option<&str>) -> (ParaProps, RunProps, u8) {
        let mut para = self.para.clone();
        let mut run = self.run.clone();
        let mut heading = 0;
        if let Some(id) = id.or(self.default_paragraph.as_deref()) {
            for style in self.chain(id) {
                para.apply(&style.para);
                run.apply(&style.run);
                if let Some(level) = heading_level(&style.name) {
                    heading = level;
                }
            }
        }
        if heading == 0
            && let Some(level) = para.outline
        {
            heading = level + 1;
        }
        (para, run, heading.min(6))
    }

    fn character(&self, id: &str) -> RunProps {
        let mut run = RunProps::default();
        for style in self.chain(id) {
            run.apply(&style.run);
        }
        run
    }
}

fn heading_level(name: &str) -> Option<u8> {
    if name == "title" {
        return Some(1);
    }
    let n = name.strip_prefix("heading")?.trim();
    n.parse::<u8>().ok().filter(|n| (1..=9).contains(n))
}

#[derive(Debug, Clone, Default)]
struct Level {
    format: String,
    text: String,
    start: u32,
    indent: Option<f32>,
}

#[derive(Debug, Default)]
struct Numbering {
    abstracts: HashMap<String, HashMap<u8, Level>>,
    /// Numbering instance: its abstract definition and per-level start overrides.
    nums: HashMap<String, (String, HashMap<u8, u32>)>,
}

impl Numbering {
    fn parse(root: &Element) -> Numbering {
        let mut numbering = Numbering::default();
        for a in root.children_named("abstractNum") {
            let Some(id) = a.attr("abstractNumId") else {
                continue;
            };
            let levels = a
                .children_named("lvl")
                .filter_map(|l| {
                    let level = l.attr("ilvl")?.parse::<u8>().ok()?;
                    Some((
                        level,
                        Level {
                            format: l.val("numFmt").unwrap_or("decimal").to_string(),
                            text: l.val("lvlText").unwrap_or("").to_string(),
                            start: l.val("start").and_then(|s| s.parse().ok()).unwrap_or(1),
                            indent: l
                                .child("pPr")
                                .and_then(|p| p.child("ind"))
                                .and_then(|i| twips(i.attr("start").or(i.attr("left")))),
                        },
                    ))
                })
                .collect();
            numbering.abstracts.insert(id.to_string(), levels);
        }
        for n in root.children_named("num") {
            let (Some(id), Some(abstract_id)) = (n.attr("numId"), n.val("abstractNumId")) else {
                continue;
            };
            let overrides = n
                .children_named("lvlOverride")
                .filter_map(|o| {
                    let level = o.attr("ilvl")?.parse::<u8>().ok()?;
                    let start = o.val("startOverride")?.parse::<u32>().ok()?;
                    Some((level, start))
                })
                .collect();
            numbering
                .nums
                .insert(id.to_string(), (abstract_id.to_string(), overrides));
        }
        numbering
    }

    fn level(&self, num: &str, level: u8) -> Option<(Level, u32)> {
        let (abstract_id, overrides) = self.nums.get(num)?;
        let def = self.abstracts.get(abstract_id)?.get(&level)?.clone();
        let start = overrides.get(&level).copied().unwrap_or(def.start);
        Some((def, start))
    }
}

/// Counters of one numbering instance, per level; `None` before the level is used.
type Counters = [Option<u32>; 9];

fn format_number(n: u32, format: &str) -> String {
    match format {
        "none" => String::new(),
        "decimalZero" => format!("{n:02}"),
        "lowerLetter" | "upperLetter" => {
            let n = n.max(1) - 1;
            let letter = char::from(b'a' + (n % 26) as u8);
            let s: String = std::iter::repeat_n(letter, (n / 26 + 1) as usize).collect();
            if format == "upperLetter" {
                s.to_uppercase()
            } else {
                s
            }
        }
        "lowerRoman" => roman(n).to_lowercase(),
        "upperRoman" => roman(n),
        _ => n.to_string(),
    }
}

fn roman(mut n: u32) -> String {
    if n == 0 || n >= 4000 {
        return n.to_string();
    }
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (value, digits) in table {
        while n >= value {
            out.push_str(digits);
            n -= value;
        }
    }
    out
}

/// Bullets drawn from symbol fonts sit in the private use area; show them as plain
/// Unicode bullets.
fn bullet(text: &str) -> String {
    let mapped: String = text
        .chars()
        .map(|c| match c {
            '\u{F0A7}' | '\u{F06E}' | '\u{F0FC}' => '▪',
            '\u{F000}'..='\u{F0FF}' => '•',
            'o' => '◦',
            c => c,
        })
        .collect();
    if mapped.trim().is_empty() {
        "•".into()
    } else {
        mapped
    }
}

/// A piece of a paragraph's content, in order.
enum Piece {
    Text(Span),
    Picture(Picture),
    PageBreak,
    /// The content of a text box.
    Blocks(Vec<Block>),
}

struct Reader<'a> {
    package: &'a mut Package,
    rels: HashMap<&'a str, &'a Rel>,
    styles: Styles,
    numbering: Numbering,
    counters: HashMap<String, Counters>,
}

impl Reader<'_> {
    fn blocks(&mut self, container: &Element, out: &mut Vec<Block>) {
        for e in container.elements() {
            match e.name.as_str() {
                "p" => self.paragraph(e, out),
                "tbl" => self.table(e, out),
                "sdt" => {
                    if let Some(content) = e.child("sdtContent") {
                        self.blocks(content, out);
                    }
                }
                "customXml" | "ins" | "moveTo" => self.blocks(e, out),
                _ => {}
            }
        }
    }

    fn marker(&mut self, num: &str, level: u8) -> Option<(String, Option<f32>)> {
        let (def, start) = self.numbering.level(num, level)?;
        let counters = self.counters.entry(num.to_string()).or_insert([None; 9]);
        let i = usize::from(level);
        counters[i] = Some(counters[i].map_or(start, |n| n.saturating_add(1)));
        for deeper in counters.iter_mut().skip(i + 1) {
            *deeper = None;
        }
        let counters = *counters;
        let text = if def.format == "bullet" {
            bullet(&def.text)
        } else {
            let mut text = def.text.clone();
            for k in (0..9u8).rev() {
                let placeholder = format!("%{}", k + 1);
                if !text.contains(&placeholder) {
                    continue;
                }
                let (format, value) = match self.numbering.level(num, k) {
                    Some((d, s)) => (d.format, counters[usize::from(k)].unwrap_or(s)),
                    None => ("decimal".to_string(), 1),
                };
                text = text.replace(&placeholder, &format_number(value, &format));
            }
            text
        };
        Some((text, def.indent))
    }

    fn paragraph(&mut self, p: &Element, out: &mut Vec<Block>) {
        let ppr = p.child("pPr");
        let (mut props, run, heading) = self.styles.paragraph(ppr.and_then(|x| x.val("pStyle")));
        if let Some(ppr) = ppr {
            props.apply(&ParaProps::parse(ppr));
        }
        let mut style = ParagraphStyle {
            heading,
            align: props.align.unwrap_or_default(),
            rtl: Some(props.rtl.unwrap_or(false)),
            indent: props.indent.unwrap_or(0.0).clamp(0.0, 400.0),
            space_before: props.before.unwrap_or(0.0).clamp(0.0, 200.0),
            space_after: props.after.unwrap_or(0.0).clamp(0.0, 200.0),
            size: run.size.unwrap_or(10.0),
            page_break_before: props.break_before.unwrap_or(false),
            ..ParagraphStyle::default()
        };
        if let Some((num, level)) = &props.num
            && num != "0"
            && let Some((marker, indent)) = self.marker(num, *level)
        {
            style.list = Some(marker);
            style.indent = indent
                .unwrap_or(36.0 * f32::from(level + 1))
                .clamp(0.0, 400.0);
        }

        let mut pieces = Vec::new();
        self.inline(p, &run, &mut pieces);
        emit(pieces, style, out);

        // A section break ends the page unless it is continuous.
        if let Some(sect) = ppr.and_then(|x| x.child("sectPr"))
            && sect.val("type") != Some("continuous")
        {
            out.push(Block::PageBreak);
        }
    }

    fn inline(&mut self, e: &Element, run: &RunProps, pieces: &mut Vec<Piece>) {
        for c in e.elements() {
            match c.name.as_str() {
                "r" => self.run(c, run, pieces),
                "hyperlink" | "smartTag" | "customXml" | "ins" | "moveTo" | "fldSimple" | "bdo"
                | "dir" => self.inline(c, run, pieces),
                "sdt" => {
                    if let Some(content) = c.child("sdtContent") {
                        self.inline(content, run, pieces);
                    }
                }
                _ => {}
            }
        }
    }

    fn run(&mut self, r: &Element, inherited: &RunProps, pieces: &mut Vec<Piece>) {
        let rpr = r.child("rPr");
        let mut props = inherited.clone();
        if let Some(id) = rpr.and_then(|x| x.val("rStyle")) {
            props.apply(&self.styles.character(id));
        }
        if let Some(rpr) = rpr {
            props.apply(&RunProps::parse(rpr));
        }
        if props.hidden == Some(true) {
            return;
        }
        self.run_content(r, &props, pieces);
    }

    fn run_content(&mut self, r: &Element, props: &RunProps, pieces: &mut Vec<Piece>) {
        let text = |pieces: &mut Vec<Piece>, s: &str| {
            if s.is_empty() {
                return;
            }
            if let Some(Piece::Text(last)) = pieces.last_mut() {
                let span = props.span(String::new());
                if last.same_style(&span) {
                    last.text.push_str(s);
                    return;
                }
            }
            pieces.push(Piece::Text(props.span(s.to_string())));
        };
        for c in r.elements() {
            match c.name.as_str() {
                "t" => text(pieces, &c.text()),
                "tab" | "ptab" => text(pieces, "\t"),
                "br" => match c.attr("type") {
                    Some("page") => pieces.push(Piece::PageBreak),
                    _ => text(pieces, "\n"),
                },
                "cr" => text(pieces, "\n"),
                "noBreakHyphen" => text(pieces, "-"),
                "sym" => {
                    let c = c
                        .attr("char")
                        .and_then(|h| u32::from_str_radix(h, 16).ok())
                        .and_then(char::from_u32);
                    if let Some(c) = c.filter(|c| !('\u{F000}'..='\u{F0FF}').contains(c)) {
                        text(pieces, &c.to_string());
                    }
                }
                "drawing" | "pict" | "object" => self.graphics(c, pieces),
                "AlternateContent" => {
                    // The first choice is the modern form; the fallback repeats it.
                    if let Some(choice) = c.elements().next() {
                        self.run_content(choice, props, pieces);
                    }
                }
                _ => {}
            }
        }
    }

    fn graphics(&mut self, g: &Element, pieces: &mut Vec<Piece>) {
        let size = g.find("extent").and_then(|e| {
            let cx = e.attr("cx")?.parse::<f32>().ok()?;
            let cy = e.attr("cy")?.parse::<f32>().ok()?;
            Some((cx * EMU, cy * EMU))
        });
        let mut ids = Vec::new();
        let mut blips = Vec::new();
        g.find_all("blip", &mut blips);
        ids.extend(blips.iter().filter_map(|b| b.attr("embed")));
        let mut vml = Vec::new();
        g.find_all("imagedata", &mut vml);
        ids.extend(vml.iter().filter_map(|v| v.attr("id")));
        let single = ids.len() == 1;
        for id in ids {
            if let Some(picture) = self.picture(id, if single { size } else { None }) {
                pieces.push(Piece::Picture(picture));
            }
        }
        let mut boxes = Vec::new();
        g.find_all("txbxContent", &mut boxes);
        for b in boxes {
            let mut blocks = Vec::new();
            self.blocks(b, &mut blocks);
            if !blocks.is_empty() {
                pieces.push(Piece::Blocks(blocks));
            }
        }
    }

    fn picture(&mut self, id: &str, size: Option<(f32, f32)>) -> Option<Picture> {
        let rel = self.rels.get(id).filter(|r| !r.external)?;
        let target = rel.target.clone();
        let bytes = self.package.read(&target).ok()??;
        Picture::from_bytes(bytes, size)
    }

    fn table(&mut self, t: &Element, out: &mut Vec<Block>) {
        let widths: Vec<f32> = t
            .child("tblGrid")
            .map(|g| {
                g.children_named("gridCol")
                    .map(|c| twips(c.attr("w")).unwrap_or(0.0).max(0.0))
                    .collect()
            })
            .unwrap_or_default();
        let mut rows = Vec::new();
        for tr in t.children_named("tr") {
            let mut cells = Vec::new();
            for tc in tr.children_named("tc") {
                let span = tc
                    .child("tcPr")
                    .and_then(|p| p.val("gridSpan"))
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(1)
                    .clamp(1, 64);
                let mut blocks = Vec::new();
                self.blocks(tc, &mut blocks);
                cells.push(Cell {
                    paragraphs: flatten(blocks),
                    span,
                });
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        if !rows.is_empty() {
            out.push(Block::Table(Table { rows, widths }));
        }
    }
}

/// The paragraphs in `blocks`, with nested tables read row by row.
fn flatten(blocks: Vec<Block>) -> Vec<Paragraph> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Paragraph(p) => out.push(p),
            Block::Table(t) => {
                for cell in t.rows.into_iter().flatten() {
                    out.extend(cell.paragraphs);
                }
            }
            Block::Picture(_) | Block::PageBreak => {}
        }
    }
    out
}

/// Turns a paragraph's pieces into blocks: text runs become paragraphs, split around
/// pictures, page breaks and text boxes.
fn emit(pieces: Vec<Piece>, style: ParagraphStyle, out: &mut Vec<Block>) {
    let mut spans: Vec<Span> = Vec::new();
    let mut emitted = false;
    let mut first = true;
    let flush = |spans: &mut Vec<Span>, out: &mut Vec<Block>, first: &mut bool| {
        if spans.iter().all(|s| s.text.is_empty()) {
            spans.clear();
            return;
        }
        let mut style = style.clone();
        if !*first {
            // Only the first part keeps the list marker and the break before it.
            style.list = None;
            style.page_break_before = false;
        }
        *first = false;
        out.push(Block::Paragraph(Paragraph {
            spans: std::mem::take(spans),
            style,
        }));
    };
    for piece in pieces {
        match piece {
            Piece::Text(span) => spans.push(span),
            Piece::Picture(p) => {
                flush(&mut spans, out, &mut first);
                out.push(Block::Picture(p));
                emitted = true;
            }
            Piece::PageBreak => {
                flush(&mut spans, out, &mut first);
                out.push(Block::PageBreak);
                emitted = true;
            }
            Piece::Blocks(blocks) => {
                flush(&mut spans, out, &mut first);
                out.extend(blocks);
                emitted = true;
            }
        }
    }
    if spans.iter().any(|s| !s.text.is_empty()) {
        flush(&mut spans, out, &mut first);
    } else if !emitted {
        // An empty paragraph is a blank line.
        out.push(Block::Paragraph(Paragraph {
            spans: Vec::new(),
            style,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Document {
        read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/report.docx")).unwrap()
    }

    fn paragraphs(doc: &Document) -> Vec<&Paragraph> {
        doc.blocks
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn reads_the_fixture() {
        let doc = fixture();
        let ps = paragraphs(&doc);
        assert_eq!(ps[0].text(), "Quarterly report");
        assert_eq!(ps[0].style.heading, 1);
        let styled = ps[1];
        assert_eq!(
            styled.text(),
            "Plain text, then bold, italic and red words."
        );
        assert!(styled.spans.iter().any(|s| s.bold && s.text == "bold"));
        assert!(styled.spans.iter().any(|s| s.italic && s.text == "italic"));
        assert!(
            styled
                .spans
                .iter()
                .any(|s| s.color == Some([0xC0, 0x10, 0x10]) && s.text == "red")
        );
        let item = |text: &str| ps.iter().find(|p| p.text() == text).copied().unwrap();
        assert_eq!(item("First point").style.list.as_deref(), Some("•"));
        assert_eq!(item("Step one").style.list.as_deref(), Some("1."));
        assert_eq!(item("Step two").style.list.as_deref(), Some("2."));
        assert_eq!(item("Centred line").style.align, Align::Center);
        let arabic = item("مرحبا بالعالم. هذا تقرير ربع سنوي.");
        assert_eq!(arabic.style.rtl, Some(true));

        let table = doc
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Table(t) => Some(t),
                _ => None,
            })
            .unwrap();
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[1][2].paragraphs[0].text(), "+4%");
        let picture = doc
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Picture(p) => Some(p),
                _ => None,
            })
            .unwrap();
        assert_eq!((picture.pixel_width, picture.pixel_height), (160, 120));
        assert!((picture.width - 113.4).abs() < 0.5, "{}", picture.width);
        assert!(doc.blocks.contains(&Block::PageBreak));
        assert!((doc.page.width - 612.0).abs() < 1.0, "{:?}", doc.page);
    }

    #[test]
    fn survives_a_round_trip() {
        let doc = fixture();
        let bytes = super::super::write::write(&doc).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("again.docx");
        std::fs::write(&path, bytes).unwrap();
        let again = read(&path).unwrap();
        let texts =
            |d: &Document| -> Vec<String> { paragraphs(d).iter().map(|p| p.text()).collect() };
        // List markers are written as text in front of the item.
        let expected: Vec<String> = paragraphs(&doc)
            .iter()
            .map(|p| match &p.style.list {
                Some(m) => format!("{m}\t{}", p.text()),
                None => p.text(),
            })
            .collect();
        assert_eq!(texts(&again), expected);
        assert_eq!(again.blocks.len(), doc.blocks.len());
        assert_eq!(paragraphs(&again)[0].style.heading, 1);
    }

    #[test]
    fn numbers_and_letters() {
        assert_eq!(format_number(3, "decimal"), "3");
        assert_eq!(format_number(3, "lowerLetter"), "c");
        assert_eq!(format_number(28, "upperLetter"), "BB");
        assert_eq!(format_number(14, "lowerRoman"), "xiv");
        assert_eq!(format_number(7, "decimalZero"), "07");
        assert_eq!(bullet("\u{F0B7}"), "•");
        assert_eq!(bullet("o"), "◦");
        assert_eq!(heading_level("heading 2"), Some(2));
        assert_eq!(heading_level("title"), Some(1));
        assert_eq!(heading_level("normal"), None);
        assert_eq!(parse_color("C01010"), Some([0xC0, 0x10, 0x10]));
        assert_eq!(parse_color("auto"), None);
    }
}
