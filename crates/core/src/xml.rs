//! A minimal XML tree for the Office files Dropwheel reads. Names are kept without their
//! namespace prefix, which is how OOXML is matched in practice (`w:p` is `p`).

use std::borrow::Cow;

use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

/// Deeper nesting than this is refused rather than risking the stack.
const MAX_DEPTH: usize = 256;
/// Nor more elements than this in one part.
const MAX_ELEMENTS: usize = 4_000_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

impl Element {
    /// The value of the attribute named `local`, whatever its prefix.
    pub fn attr(&self, local: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == local)
            .map(|(_, v)| v.as_str())
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    pub fn child(&self, local: &str) -> Option<&Element> {
        self.elements().find(|e| e.name == local)
    }

    pub fn children_named<'a>(&'a self, local: &'a str) -> impl Iterator<Item = &'a Element> {
        self.elements().filter(move |e| e.name == local)
    }

    /// The first element named `local` anywhere below this one.
    pub fn find(&self, local: &str) -> Option<&Element> {
        for e in self.elements() {
            if e.name == local {
                return Some(e);
            }
            if let Some(found) = e.find(local) {
                return Some(found);
            }
        }
        None
    }

    /// Every element named `local` below this one, in document order.
    pub fn find_all<'a>(&'a self, local: &str, out: &mut Vec<&'a Element>) {
        for e in self.elements() {
            if e.name == local {
                out.push(e);
            } else {
                e.find_all(local, out);
            }
        }
    }

    /// The text directly inside this element.
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|n| match n {
                Node::Text(t) => Some(t.as_str()),
                Node::Element(_) => None,
            })
            .collect()
    }

    /// A child's `val` attribute, as in `<w:jc w:val="center"/>`.
    pub fn val(&self, child: &str) -> Option<&str> {
        self.child(child).and_then(|c| c.attr("val"))
    }

    /// An OOXML on/off property: present means on, unless its value says otherwise.
    pub fn flag(&self, child: &str) -> Option<bool> {
        self.child(child)
            .map(|c| !matches!(c.attr("val"), Some("0" | "false" | "off" | "none")))
    }
}

fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or(name).to_owned()
}

fn element(start: &BytesStart) -> Element {
    let attrs = start
        .attributes()
        .with_checks(false)
        .flatten()
        .map(|a| {
            let value = a
                .normalized_value(XmlVersion::Implicit1_0)
                .map(Cow::into_owned)
                .unwrap_or_else(|_| a.value.clone().into_owned());
            (local(a.key.as_ref()), value)
        })
        .collect();
    Element {
        name: local(start.name().as_ref()),
        attrs,
        children: Vec::new(),
    }
}

fn entity(name: &str) -> Option<char> {
    match name {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "apos" => Some('\''),
        "quot" => Some('"'),
        _ => None,
    }
}

/// Parses `data` into its root element.
pub fn parse(data: &[u8]) -> Result<Element, String> {
    let mut reader = Reader::from_reader(data);
    let mut stack: Vec<Element> = vec![Element::default()];
    let mut count = 0usize;
    let push_text = |stack: &mut Vec<Element>, text: Cow<str>| {
        if let Some(top) = stack.last_mut() {
            match top.children.last_mut() {
                Some(Node::Text(t)) => t.push_str(&text),
                _ => top.children.push(Node::Text(text.into_owned())),
            }
        }
    };
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(start) => {
                count += 1;
                if stack.len() > MAX_DEPTH || count > MAX_ELEMENTS {
                    return Err("the XML is nested too deeply or is too large".into());
                }
                stack.push(element(&start));
            }
            Event::Empty(start) => {
                count += 1;
                if count > MAX_ELEMENTS {
                    return Err("the XML is too large".into());
                }
                let e = element(&start);
                if let Some(top) = stack.last_mut() {
                    top.children.push(Node::Element(e));
                }
            }
            Event::End(_) => {
                if stack.len() < 2 {
                    return Err("the XML closes more elements than it opens".into());
                }
                if let Some(done) = stack.pop()
                    && let Some(parent) = stack.last_mut()
                {
                    parent.children.push(Node::Element(done));
                }
            }
            Event::Text(t) => push_text(&mut stack, t.xml_content(XmlVersion::Implicit1_0)),
            Event::CData(t) => push_text(&mut stack, t.into_inner()),
            Event::GeneralRef(r) => {
                let name = r.xml_content(XmlVersion::Implicit1_0);
                let c = r
                    .resolve_char_ref()
                    .ok()
                    .flatten()
                    .or_else(|| entity(&name));
                if let Some(c) = c {
                    push_text(&mut stack, c.to_string().into());
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let mut document = stack.into_iter().next().unwrap_or_default();
    document.children.retain(|n| matches!(n, Node::Element(_)));
    match document.children.pop() {
        Some(Node::Element(root)) => Ok(root),
        _ => Err("the XML has no root element".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ooxml() {
        let xml = br#"<?xml version="1.0"?>
            <w:document xmlns:w="x"><w:body><w:p><w:pPr><w:jc w:val="center"/><w:bidi/><w:b w:val="0"/></w:pPr>
            <w:r><w:t xml:space="preserve">a &amp; b &#x645; </w:t></w:r></w:p></w:body></w:document>"#;
        let root = parse(xml).unwrap();
        assert_eq!(root.name, "document");
        let p = root.find("p").unwrap();
        let ppr = p.child("pPr").unwrap();
        assert_eq!(ppr.val("jc"), Some("center"));
        assert_eq!(ppr.flag("bidi"), Some(true));
        assert_eq!(ppr.flag("b"), Some(false));
        assert_eq!(ppr.flag("i"), None);
        assert_eq!(p.find("t").unwrap().text(), "a & b م ");
    }

    #[test]
    fn hostile_xml_is_refused() {
        let deep = "<a>".repeat(MAX_DEPTH + 10);
        assert!(parse(deep.as_bytes()).is_err());
        assert!(parse(b"</a>").is_err());
        assert!(parse(b"").is_err());
        assert!(parse(b"<a><b></a>").is_err());
    }
}
