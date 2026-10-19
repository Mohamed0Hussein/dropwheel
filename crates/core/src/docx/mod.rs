//! Word documents (DOCX): a zip of XML parts tied together by relationship files.

pub mod read;
pub mod write;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::error::{Error, Result};
use crate::kind::FileKind;
use crate::xml::{self, Element};

/// No single part of a document is read past this size, so a zip bomb cannot exhaust
/// memory.
const MAX_PART: u64 = 64 * 1024 * 1024;

pub const REL_DOCUMENT: &str = "officeDocument";
pub const REL_STYLES: &str = "styles";
pub const REL_NUMBERING: &str = "numbering";
pub const REL_IMAGE: &str = "image";
pub const REL_CORE: &str = "core-properties";
pub const REL_APP: &str = "extended-properties";
pub const REL_CUSTOM: &str = "custom-properties";

/// An opened DOCX file.
pub struct Package {
    path: PathBuf,
    zip: ZipArchive<BufReader<File>>,
}

/// A relationship: where it points and what kind of part it is, by the last segment of
/// its type URI.
#[derive(Debug, Clone)]
pub struct Rel {
    pub id: String,
    pub kind: String,
    /// Full part name inside the zip, without a leading slash.
    pub target: String,
    pub external: bool,
}

impl Package {
    pub fn open(path: &Path) -> Result<Package> {
        let file = File::open(path).map_err(|e| Error::io(path, e))?;
        let zip = ZipArchive::new(BufReader::new(file)).map_err(|e| Error::Decode {
            path: path.to_path_buf(),
            kind: FileKind::Docx,
            reason: e.to_string(),
        })?;
        Ok(Package {
            path: path.to_path_buf(),
            zip,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn error(&self, reason: impl Into<String>) -> Error {
        Error::Decode {
            path: self.path.clone(),
            kind: FileKind::Docx,
            reason: reason.into(),
        }
    }

    pub fn names(&self) -> Vec<String> {
        self.zip
            .file_names()
            .filter_map(|n| n.ok().map(|n| n.into_owned()))
            .collect()
    }

    /// The bytes of part `name`, if the package has it.
    pub fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>> {
        let reason = match self.zip.by_name(name) {
            Ok(file) => {
                let mut bytes = Vec::new();
                match file.take(MAX_PART + 1).read_to_end(&mut bytes) {
                    Ok(_) if bytes.len() as u64 > MAX_PART => format!("{name} is too large"),
                    Ok(_) => return Ok(Some(bytes)),
                    Err(e) => format!("{name}: {e}"),
                }
            }
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => e.to_string(),
        };
        Err(self.error(reason))
    }

    /// Part `name` parsed as XML.
    pub fn xml(&mut self, name: &str) -> Result<Option<Element>> {
        match self.read(name)? {
            Some(bytes) => xml::parse(&bytes)
                .map(Some)
                .map_err(|e| self.error(format!("{name}: {e}"))),
            None => Ok(None),
        }
    }

    /// The relationships of part `source` (`""` for the package itself).
    pub fn rels(&mut self, source: &str) -> Result<Vec<Rel>> {
        let (dir, file) = match source.rsplit_once('/') {
            Some((dir, file)) => (dir, file),
            None => ("", source),
        };
        let rels_name = if dir.is_empty() {
            format!("_rels/{file}.rels")
        } else {
            format!("{dir}/_rels/{file}.rels")
        };
        let Some(root) = self.xml(&rels_name)? else {
            return Ok(Vec::new());
        };
        Ok(root
            .children_named("Relationship")
            .filter_map(|r| {
                let target = r.attr("Target")?;
                let external = r.attr("TargetMode") == Some("External");
                Some(Rel {
                    id: r.attr("Id")?.to_string(),
                    kind: r.attr("Type")?.rsplit('/').next()?.to_string(),
                    target: if external {
                        target.to_string()
                    } else {
                        resolve(dir, target)
                    },
                    external,
                })
            })
            .collect())
    }

    /// The main document part's name, usually `word/document.xml`.
    pub fn main_part(&mut self) -> Result<String> {
        self.rels("")?
            .into_iter()
            .find(|r| r.kind == REL_DOCUMENT)
            .map(|r| r.target)
            .ok_or_else(|| self.error("it has no main document; it may not be a Word file"))
    }
}

/// Resolves `target` against the folder `dir` the way OPC does: absolute targets start
/// at the root, and `..` steps up.
pub fn resolve(dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = if target.starts_with('/') {
        Vec::new()
    } else {
        dir.split('/').filter(|p| !p.is_empty()).collect()
    };
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Relationships by id.
pub fn by_id(rels: &[Rel]) -> HashMap<&str, &Rel> {
    rels.iter().map(|r| (r.id.as_str(), r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_resolve() {
        assert_eq!(resolve("word", "media/a.png"), "word/media/a.png");
        assert_eq!(resolve("word", "../customXml/x.xml"), "customXml/x.xml");
        assert_eq!(resolve("word", "/word/styles.xml"), "word/styles.xml");
        assert_eq!(resolve("", "word/document.xml"), "word/document.xml");
        assert_eq!(resolve("word", "../../../../etc/passwd"), "etc/passwd");
    }
}
