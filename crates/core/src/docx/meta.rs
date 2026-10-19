//! A Word document's metadata: document properties, custom properties, the thumbnail,
//! and the names attached to comments and tracked changes.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::{Package, REL_APP, REL_CORE, REL_CUSTOM};
use crate::error::{Error, Result};
use crate::kind::FileKind;
use crate::xml::Element;

/// A group, a name and a value, as the Metadata dialog lists them.
pub type Row = (&'static str, String, String);

fn core_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "title" => "Title",
        "subject" => "Subject",
        "creator" => "Author",
        "keywords" => "Keywords",
        "description" => "Comments",
        "lastModifiedBy" => "Last saved by",
        "revision" => "Revision",
        "created" => "Created",
        "modified" => "Modified",
        "lastPrinted" => "Last printed",
        "category" => "Category",
        "contentStatus" => "Status",
        "language" => "Language",
        "identifier" => "Identifier",
        "version" => "Version",
        _ => return None,
    })
}

fn app_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "Application" => "Application",
        "AppVersion" => "Application version",
        "Company" => "Company",
        "Manager" => "Manager",
        "Template" => "Template",
        "TotalTime" => "Editing time (minutes)",
        "HyperlinkBase" => "Hyperlink base",
        _ => return None,
    })
}

/// Authors named on elements anywhere in `root`.
fn authors(root: &Element, out: &mut BTreeSet<String>) {
    for e in root.elements() {
        if let Some(a) = e.attr("author").filter(|a| !a.trim().is_empty()) {
            out.insert(a.to_string());
        }
        authors(e, out);
    }
}

/// The metadata of the DOCX at `path`.
pub fn read(path: &Path) -> Result<Vec<Row>> {
    let mut package = Package::open(path)?;
    let rels = package.rels("")?;
    let mut rows = Vec::new();
    let target = |kind: &str| {
        rels.iter()
            .find(|r| r.kind == kind)
            .map(|r| r.target.clone())
    };

    if let Some(core) = target(REL_CORE)
        && let Some(root) = package.xml(&core)?
    {
        for e in root.elements() {
            let value = e.text();
            if let Some(label) = core_label(&e.name)
                && !value.trim().is_empty()
            {
                rows.push(("Document", label.to_string(), value.trim().to_string()));
            }
        }
    }
    if let Some(app) = target(REL_APP)
        && let Some(root) = package.xml(&app)?
    {
        for e in root.elements() {
            let value = e.text();
            if let Some(label) = app_label(&e.name)
                && !value.trim().is_empty()
            {
                rows.push(("Application", label.to_string(), value.trim().to_string()));
            }
        }
    }
    if let Some(custom) = target(REL_CUSTOM)
        && let Some(root) = package.xml(&custom)?
    {
        for p in root.children_named("property") {
            let name = p.attr("name").unwrap_or("Property").to_string();
            let value: String = p.elements().map(Element::text).collect();
            rows.push(("Custom", name, value));
        }
    }
    if rels.iter().any(|r| r.kind == "thumbnail") {
        rows.push((
            "Document",
            "Thumbnail".into(),
            "A preview picture of the first page".into(),
        ));
    }

    let main = package.main_part()?;
    let mut names = BTreeSet::new();
    let mut parts = vec![main.clone()];
    parts.extend(
        package
            .rels(&main)?
            .into_iter()
            .filter(|r| {
                matches!(
                    r.kind.as_str(),
                    "comments" | "people" | "footnotes" | "endnotes"
                )
            })
            .map(|r| r.target),
    );
    for part in parts {
        if let Some(root) = package.xml(&part)? {
            authors(&root, &mut names);
        }
    }
    if !names.is_empty() {
        rows.push((
            "People",
            "Comments and changes by".into(),
            names.into_iter().collect::<Vec<_>>().join(", "),
        ));
    }
    Ok(rows)
}

/// Blanks the values of attributes named `names` (with their prefixes) in `xml`.
fn blank_attributes(xml: &str, names: &[&str]) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    'outer: loop {
        let mut next: Option<(usize, usize)> = None;
        for name in names {
            let needle = format!(" {name}=\"");
            if let Some(i) = rest.find(&needle)
                && next.is_none_or(|(j, _)| i < j)
            {
                next = Some((i, needle.len()));
            }
        }
        let Some((i, len)) = next else {
            out.push_str(rest);
            break 'outer;
        };
        let value_start = i + len;
        let Some(end) = rest[value_start..].find('"') else {
            out.push_str(rest);
            break 'outer;
        };
        out.push_str(&rest[..value_start]);
        rest = &rest[value_start + end..];
    }
    out
}

/// Parts whose names reveal who wrote or reviewed the document.
fn names_people(name: &str) -> bool {
    name.starts_with("word/")
        && name.ends_with(".xml")
        && [
            "document",
            "comments",
            "footnotes",
            "endnotes",
            "header",
            "footer",
            "people",
        ]
        .iter()
        .any(|p| name["word/".len()..].starts_with(p))
}

/// The DOCX at `path` without its properties, custom properties, thumbnail, or the
/// names on comments and tracked changes. The text and its formatting are untouched.
pub fn strip(path: &Path) -> Result<Vec<u8>> {
    // Only Word documents: any other zip is refused, as it is everywhere else.
    Package::open(path)?.main_part()?;
    let fail = |reason: String| Error::Decode {
        path: path.to_path_buf(),
        kind: FileKind::Docx,
        reason,
    };
    let file = File::open(path).map_err(|e| Error::io(path, e))?;
    let mut zip = ZipArchive::new(BufReader::new(file)).map_err(|e| fail(e.to_string()))?;
    let mut out = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let write_err = |e: &dyn std::fmt::Display| Error::Encode {
        kind: FileKind::Docx,
        reason: e.to_string(),
    };

    for i in 0..zip.len() {
        let name = {
            let entry = zip.by_index_raw(i).map_err(|e| fail(e.to_string()))?;
            entry.name().map_err(|e| fail(e.to_string()))?.into_owned()
        };
        let replacement: Option<Vec<u8>> = match name.as_str() {
            "docProps/core.xml" => Some(super::write::CORE.as_bytes().to_vec()),
            "docProps/app.xml" => Some(super::write::APP.as_bytes().to_vec()),
            n if n == "docProps/custom.xml" || n.starts_with("docProps/thumbnail") => continue,
            "_rels/.rels" | "[Content_Types].xml" => {
                let text = read_text(&mut zip, i).map_err(fail)?;
                Some(drop_entries(&text).into_bytes())
            }
            n if names_people(n) => {
                let text = read_text(&mut zip, i).map_err(fail)?;
                Some(
                    blank_attributes(
                        &text,
                        &["w:author", "w:initials", "w15:author", "w15:userId"],
                    )
                    .into_bytes(),
                )
            }
            _ => None,
        };
        match replacement {
            Some(bytes) => {
                out.start_file(name, options).map_err(|e| write_err(&e))?;
                out.write_all(&bytes).map_err(|e| write_err(&e))?;
            }
            None => {
                let raw = zip.by_index_raw(i).map_err(|e| fail(e.to_string()))?;
                out.raw_copy_file(raw).map_err(|e| write_err(&e))?;
            }
        }
    }
    Ok(out.finish().map_err(|e| write_err(&e))?.into_inner())
}

fn read_text<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
    i: usize,
) -> std::result::Result<String, String> {
    let file = zip.by_index(i).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(64 * 1024 * 1024)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Ok(text)
}

/// Removes relationships and content types of the parts [`strip`] drops.
fn drop_entries(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(len) = rest[start..].find('>') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let tag = &rest[start..=start + len];
        let dropped = (tag.starts_with("<Relationship ") || tag.starts_with("<Override "))
            && (tag.contains("custom-properties")
                || tag.contains("/thumbnail\"")
                || tag.contains("docProps/custom.xml")
                || tag.contains("docProps/thumbnail"));
        if !dropped {
            out.push_str(tag);
        }
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/report.docx")
    }

    #[test]
    fn lists_properties() {
        let rows = read(&fixture()).unwrap();
        let has = |name: &str, value: &str| rows.iter().any(|(_, n, v)| n == name && v == value);
        assert!(has("Title", "Quarterly report"), "{rows:#?}");
        assert!(has("Author", "Test Author"));
        assert!(has("Last saved by", "Someone Else"));
        assert!(rows.iter().any(|(_, n, _)| n == "Thumbnail"));
    }

    #[test]
    fn strips_them() {
        let bytes = strip(&fixture()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clean.docx");
        std::fs::write(&path, &bytes).unwrap();
        let rows = read(&path).unwrap();
        assert!(
            rows.iter()
                .all(|(g, _, _)| *g != "Document" && *g != "People"),
            "{rows:#?}"
        );
        // Still a readable document with the same text.
        let doc = super::super::read::read(&path).unwrap();
        assert!(doc.to_text().contains("Quarterly report"));
        let mut zip = ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert!(zip.by_name("docProps/thumbnail.jpeg").is_err());
    }

    #[test]
    fn blanks_names() {
        let xml = r#"<w:ins w:id="1" w:author="Jane Doe" w:date="2026"><w:comment w:initials="JD" w:author="Ann">"#;
        assert_eq!(
            blank_attributes(xml, &["w:author", "w:initials"]),
            r#"<w:ins w:id="1" w:author="" w:date="2026"><w:comment w:initials="" w:author="">"#
        );
    }
}
