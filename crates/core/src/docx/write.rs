//! Writing a [`Document`] as a DOCX that Word, LibreOffice and Google Docs open cleanly.
//!
//! Elements are written in the order the OOXML schema requires; Word reports a file as
//! damaged when they are not.

use std::fmt::Write as _;
use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::doc::{Align, Block, Document, Paragraph, Picture, Table};
use crate::error::{Error, Result};
use crate::kind::FileKind;

const TWIPS: f32 = 20.0;
const EMU: f32 = 12_700.0;

const NS: &str = concat!(
    r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture""#
);

/// Escapes text for XML, dropping characters XML cannot hold at all.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 || matches!(c, '\u{FFFE}' | '\u{FFFF}') => {}
            c => out.push(c),
        }
    }
    out
}

fn twips(points: f32) -> i64 {
    (points * TWIPS).round() as i64
}

struct Writer<'a> {
    doc: &'a Document,
    body: String,
    /// Pictures to store, in order; the n-th is `media/image{n+1}`.
    media: Vec<&'a Picture>,
}

/// Encodes `doc` as a DOCX file.
pub fn write(doc: &Document) -> Result<Vec<u8>> {
    let mut w = Writer {
        doc,
        body: String::new(),
        media: Vec::new(),
    };
    for block in &doc.blocks {
        w.block(block);
    }
    w.finish()
}

impl<'a> Writer<'a> {
    fn block(&mut self, block: &'a Block) {
        match block {
            Block::Paragraph(p) => self.paragraph(p),
            Block::Table(t) => self.table(t),
            Block::Picture(p) => self.picture(p),
            Block::PageBreak => self
                .body
                .push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#),
        }
    }

    fn paragraph(&mut self, p: &Paragraph) {
        let s = &p.style;
        let rtl = p.is_rtl();
        let mut ppr = String::new();
        if (1..=6).contains(&s.heading) {
            let _ = write!(ppr, r#"<w:pStyle w:val="Heading{}"/>"#, s.heading);
        }
        if s.page_break_before {
            ppr.push_str("<w:pageBreakBefore/>");
        }
        if rtl {
            ppr.push_str("<w:bidi/>");
        }
        let _ = write!(
            ppr,
            r#"<w:spacing w:before="{}" w:after="{}"/>"#,
            twips(s.space_before),
            twips(s.space_after)
        );
        if s.list.is_some() {
            let indent = s.indent.max(18.0);
            let _ = write!(
                ppr,
                r#"<w:ind w:start="{}" w:hanging="{}"/>"#,
                twips(indent),
                twips(18.0)
            );
        } else if s.indent > 0.0 {
            let _ = write!(ppr, r#"<w:ind w:start="{}"/>"#, twips(s.indent));
        }
        let jc = match s.align {
            Align::Start => None,
            Align::Center => Some("center"),
            Align::End => Some("end"),
            Align::Justify => Some("both"),
        };
        if let Some(jc) = jc {
            let _ = write!(ppr, r#"<w:jc w:val="{jc}"/>"#);
        }
        self.body.push_str("<w:p><w:pPr>");
        self.body.push_str(&ppr);
        self.body.push_str("</w:pPr>");
        if let Some(marker) = &s.list {
            run(
                &mut self.body,
                &format!("{marker}\t"),
                &Default::default(),
                s.size,
            );
        }
        for span in &p.spans {
            run(&mut self.body, &span.text, span, s.size);
        }
        self.body.push_str("</w:p>");
    }

    fn table(&mut self, t: &'a Table) {
        let columns = t
            .rows
            .iter()
            .map(|r| r.iter().map(|c| usize::from(c.span.max(1))).sum::<usize>())
            .max()
            .unwrap_or(1)
            .max(1);
        let total = self.doc.page.content_width();
        let given: f32 = t.widths.iter().take(columns).sum();
        let widths: Vec<f32> = if t.widths.len() >= columns && given > 1.0 {
            t.widths
                .iter()
                .take(columns)
                .map(|w| w / given * total)
                .collect()
        } else {
            vec![total / columns as f32; columns]
        };
        self.body
            .push_str(r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblBorders>"#);
        for side in ["top", "left", "bottom", "right", "insideH", "insideV"] {
            let _ = write!(
                self.body,
                r#"<w:{side} w:val="single" w:sz="4" w:space="0" w:color="808080"/>"#
            );
        }
        self.body
            .push_str(r#"</w:tblBorders><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid>"#);
        for w in &widths {
            let _ = write!(self.body, r#"<w:gridCol w:w="{}"/>"#, twips(*w));
        }
        self.body.push_str("</w:tblGrid>");
        for row in &t.rows {
            self.body.push_str("<w:tr>");
            let mut column = 0usize;
            for cell in row {
                let span = usize::from(cell.span.max(1)).min(columns.saturating_sub(column).max(1));
                let width: f32 = widths.iter().skip(column).take(span).sum();
                column += span;
                let _ = write!(
                    self.body,
                    r#"<w:tc><w:tcPr><w:tcW w:w="{}" w:type="dxa"/>"#,
                    twips(width)
                );
                if span > 1 {
                    let _ = write!(self.body, r#"<w:gridSpan w:val="{span}"/>"#);
                }
                self.body.push_str("</w:tcPr>");
                if cell.paragraphs.is_empty() {
                    self.body.push_str("<w:p/>");
                }
                for p in &cell.paragraphs {
                    self.paragraph(p);
                }
                self.body.push_str("</w:tc>");
            }
            self.body.push_str("</w:tr>");
        }
        self.body.push_str("</w:tbl>");
    }

    fn picture(&mut self, p: &'a Picture) {
        self.media.push(p);
        let n = self.media.len();
        let page = &self.doc.page;
        let fit = (page.content_width() / p.width)
            .min(page.content_height() / p.height)
            .min(1.0);
        let cx = ((p.width * fit * EMU).round() as i64).max(1);
        let cy = ((p.height * fit * EMU).round() as i64).max(1);
        let _ = write!(
            self.body,
            concat!(
                r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0">"#,
                r#"<wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="{n}" name="Picture {n}"/>"#,
                r#"<wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr>"#,
                r#"<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">"#,
                r#"<pic:pic><pic:nvPicPr><pic:cNvPr id="{n}" name="image{n}.{ext}"/><pic:cNvPicPr/></pic:nvPicPr>"#,
                r#"<pic:blipFill><a:blip r:embed="rIdImage{n}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>"#,
                r#"<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>"#,
                r#"<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>"#,
                r#"</a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
            ),
            cx = cx,
            cy = cy,
            n = n,
            ext = p.format.extension()
        );
    }

    fn finish(self) -> Result<Vec<u8>> {
        let page = &self.doc.page;
        let landscape = if page.width > page.height {
            r#" w:orient="landscape""#
        } else {
            ""
        };
        let document = format!(
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
                "\n<w:document {ns}><w:body>{body}",
                r#"<w:sectPr><w:pgSz w:w="{w}" w:h="{h}"{landscape}/>"#,
                r#"<w:pgMar w:top="{top}" w:right="{right}" w:bottom="{bottom}" w:left="{left}" w:header="708" w:footer="708" w:gutter="0"/>"#,
                "</w:sectPr></w:body></w:document>"
            ),
            ns = NS,
            body = self.body,
            w = twips(page.width),
            h = twips(page.height),
            landscape = landscape,
            top = twips(page.margin_top),
            right = twips(page.margin_right),
            bottom = twips(page.margin_bottom),
            left = twips(page.margin_left),
        );

        let mut rels = String::from(concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            "\n",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            r#"<Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#
        ));
        for (i, p) in self.media.iter().enumerate() {
            let _ = write!(
                rels,
                r#"<Relationship Id="rIdImage{n}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image{n}.{ext}"/>"#,
                n = i + 1,
                ext = p.format.extension()
            );
        }
        rels.push_str("</Relationships>");

        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let mut parts: Vec<(String, &[u8], SimpleFileOptions)> = vec![
            (
                "[Content_Types].xml".into(),
                CONTENT_TYPES.as_bytes(),
                options,
            ),
            ("_rels/.rels".into(), PACKAGE_RELS.as_bytes(), options),
            ("docProps/core.xml".into(), CORE.as_bytes(), options),
            ("docProps/app.xml".into(), APP.as_bytes(), options),
            ("word/document.xml".into(), document.as_bytes(), options),
            (
                "word/_rels/document.xml.rels".into(),
                rels.as_bytes(),
                options,
            ),
            ("word/styles.xml".into(), STYLES.as_bytes(), options),
        ];
        for (i, p) in self.media.iter().enumerate() {
            // Pictures are compressed already.
            parts.push((
                format!("word/media/image{}.{}", i + 1, p.format.extension()),
                &p.data,
                stored,
            ));
        }
        let fail = |e: &dyn std::fmt::Display| Error::Encode {
            kind: FileKind::Docx,
            reason: e.to_string(),
        };
        for (name, data, options) in parts {
            zip.start_file(name, options).map_err(|e| fail(&e))?;
            zip.write_all(data).map_err(|e| fail(&e))?;
        }
        Ok(zip.finish().map_err(|e| fail(&e))?.into_inner())
    }
}

fn has_rtl(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
    })
}

/// Writes one run, splitting tabs and line breaks into their own elements. Runs holding
/// right-to-left letters are marked as such, so Word lays out their digits and
/// punctuation the right way.
fn run(out: &mut String, text: &str, span: &crate::doc::Span, paragraph_size: f32) {
    if text.is_empty() {
        return;
    }
    let mut rpr = String::new();
    if span.bold {
        rpr.push_str("<w:b/><w:bCs/>");
    }
    if span.italic {
        rpr.push_str("<w:i/><w:iCs/>");
    }
    if span.strike {
        rpr.push_str("<w:strike/>");
    }
    if let Some([r, g, b]) = span.color {
        let _ = write!(rpr, r#"<w:color w:val="{r:02X}{g:02X}{b:02X}"/>"#);
    }
    let size = span.size.unwrap_or(paragraph_size);
    let half_points = (size * 2.0).round().clamp(2.0, 3276.0) as i64;
    if half_points != 22 {
        let _ = write!(
            rpr,
            r#"<w:sz w:val="{half_points}"/><w:szCs w:val="{half_points}"/>"#
        );
    }
    if span.underline {
        rpr.push_str(r#"<w:u w:val="single"/>"#);
    }
    if has_rtl(text) {
        rpr.push_str("<w:rtl/>");
    }
    out.push_str("<w:r>");
    if !rpr.is_empty() {
        out.push_str("<w:rPr>");
        out.push_str(&rpr);
        out.push_str("</w:rPr>");
    }
    let mut piece = String::new();
    let flush = |out: &mut String, piece: &mut String| {
        if !piece.is_empty() {
            let _ = write!(out, r#"<w:t xml:space="preserve">{}</w:t>"#, escape(piece));
            piece.clear();
        }
    };
    for c in text.chars() {
        match c {
            '\t' => {
                flush(out, &mut piece);
                out.push_str("<w:tab/>");
            }
            '\n' => {
                flush(out, &mut piece);
                out.push_str("<w:br/>");
            }
            '\r' => {}
            c => piece.push(c),
        }
    }
    flush(out, &mut piece);
    out.push_str("</w:r>");
}

const CONTENT_TYPES: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    "\n",
    r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
    r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
    r#"<Default Extension="xml" ContentType="application/xml"/>"#,
    r#"<Default Extension="png" ContentType="image/png"/>"#,
    r#"<Default Extension="jpeg" ContentType="image/jpeg"/>"#,
    r#"<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
    r#"<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#,
    r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#,
    r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#,
    "</Types>"
);

const PACKAGE_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    "\n",
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>"#,
    r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>"#,
    r#"<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>"#,
    "</Relationships>"
);

/// Deliberately empty: a converted file carries no author, title or dates.
pub const CORE: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    "\n",
    r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" "#,
    r#"xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" "#,
    r#"xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"/>"#
);

pub const APP: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    "\n",
    r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" "#,
    r#"xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"/>"#
);

const STYLES: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    "\n",
    r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    r#"<w:docDefaults><w:rPrDefault><w:rPr>"#,
    r#"<w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Arial"/>"#,
    r#"<w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>"#,
    r#"</w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="0" w:line="264" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>"#,
    r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="40"/><w:szCs w:val="40"/></w:rPr></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style>"#,
    r#"<w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:b/><w:bCs/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style>"#,
    "</w:styles>"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{Cell, ParagraphStyle, Span};
    use crate::xml;

    fn sample() -> Document {
        let mut heading = Paragraph::plain("Report & <summary>");
        heading.style = ParagraphStyle::heading(1);
        let mut item = Paragraph::plain("first");
        item.style.list = Some("1.".into());
        item.style.indent = 36.0;
        let mixed = Paragraph {
            spans: vec![
                Span::plain("plain\ttabbed\nnext line "),
                Span {
                    bold: true,
                    color: Some([200, 0, 0]),
                    ..Span::plain("bold red")
                },
            ],
            style: ParagraphStyle::default(),
        };
        Document {
            blocks: vec![
                Block::Paragraph(heading),
                Block::Paragraph(mixed),
                Block::Paragraph(item),
                Block::Table(Table {
                    rows: vec![vec![
                        Cell {
                            paragraphs: vec![Paragraph::plain("a")],
                            span: 2,
                        },
                        Cell::default(),
                    ]],
                    widths: vec![],
                }),
                Block::PageBreak,
                Block::Paragraph(Paragraph::plain("مرحبا")),
            ],
            ..Document::default()
        }
    }

    #[test]
    fn writes_a_well_formed_package() {
        let bytes = write(&sample()).unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        for name in [
            "[Content_Types].xml",
            "_rels/.rels",
            "word/document.xml",
            "word/_rels/document.xml.rels",
            "word/styles.xml",
            "docProps/core.xml",
            "docProps/app.xml",
        ] {
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut zip.by_name(name).unwrap(), &mut data).unwrap();
            xml::parse(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        let mut data = String::new();
        std::io::Read::read_to_string(&mut zip.by_name("word/document.xml").unwrap(), &mut data)
            .unwrap();
        assert!(data.contains("Report &amp; &lt;summary&gt;"));
        assert!(data.contains(r#"<w:pStyle w:val="Heading1"/>"#));
        assert!(data.contains("<w:tab/>"));
        assert!(data.contains("<w:br/>"));
        assert!(data.contains(r#"<w:color w:val="C80000"/>"#));
        assert!(data.contains(r#"<w:gridSpan w:val="2"/>"#));
        assert!(data.contains("<w:bidi/>"));
        assert!(data.contains(r#"<w:br w:type="page"/>"#));
    }

    #[test]
    fn escapes_and_drops_control_characters() {
        assert_eq!(escape("a<b>&\"c\u{1}\u{FFFF}"), "a&lt;b&gt;&amp;&quot;c");
    }
}
