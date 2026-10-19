//! The actions the radial menu offers, in menu order.

pub mod annotate;
mod background;
pub mod compress;
mod convert;
pub mod crop;
mod documents;
pub mod edit;
pub mod metadata;
pub mod redact;
mod support;

pub use convert::{Convert, targets};

use crate::action::Action;
use crate::kind::FileKind;

static ALL: [&dyn Action; 17] = [
    &convert::DOCX,
    &convert::PDF,
    &convert::TXT,
    &convert::JPG,
    &convert::PNG,
    &convert::WEBP,
    &convert::BMP,
    &convert::TIFF,
    &convert::AVIF,
    &convert::HEIC,
    &compress::COMPRESS,
    &crop::CROP,
    &edit::EDIT,
    &annotate::ANNOTATE,
    &redact::REDACT,
    &metadata::METADATA,
    &background::ADD_BACKGROUND,
];

pub fn all() -> &'static [&'static dyn Action] {
    &ALL
}

pub fn find(id: &str) -> Option<&'static dyn Action> {
    ALL.iter().copied().find(|a| a.id() == id)
}

/// The actions that apply to every one of `kinds`, so a multi-file drop only offers what
/// can be done to all of the files. Empty when `kinds` is.
pub fn for_kinds(kinds: &[FileKind]) -> Vec<&'static dyn Action> {
    if kinds.is_empty() {
        return Vec::new();
    }
    ALL.iter()
        .copied()
        .filter(|a| kinds.iter().all(|k| a.applicable(*k)))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::Path;

    use super::*;
    use crate::action::{ActionGroup, Options};
    use crate::imaging;
    use crate::kind::FileKind::*;

    /// HEIC is only offered where Microsoft's encoder is installed.
    fn with_heic(mut labels: Vec<&'static str>) -> Vec<&'static str> {
        if !imaging::can_encode(Heic) {
            labels.retain(|l| *l != "HEIC");
        }
        labels
    }

    fn labels(kinds: &[FileKind], group: ActionGroup) -> Vec<&'static str> {
        for_kinds(kinds)
            .into_iter()
            .filter(|a| a.group() == group)
            .map(|a| a.label())
            .collect()
    }

    fn converts(kinds: &[FileKind]) -> Vec<&'static str> {
        labels(kinds, ActionGroup::Convert)
    }

    fn tools(kinds: &[FileKind]) -> Vec<&'static str> {
        labels(kinds, ActionGroup::Tools)
    }

    #[test]
    fn ids_are_unique_and_findable() {
        let ids: HashSet<_> = all().iter().map(|a| a.id()).collect();
        assert_eq!(ids.len(), all().len());
        for a in all() {
            assert_eq!(find(a.id()).map(|f| f.id()), Some(a.id()));
        }
        assert!(find("convert.exe").is_none());
    }

    #[test]
    fn menu_order_follows_the_brief() {
        let order: Vec<_> = all().iter().map(|a| a.label()).collect();
        assert_eq!(
            order,
            [
                "DOCX",
                "PDF",
                "TXT",
                "JPG",
                "PNG",
                "WEBP",
                "BMP",
                "TIFF",
                "AVIF",
                "HEIC",
                "COMPRESS",
                "CROP",
                "EDIT",
                "ANNOTATE",
                "REDACT",
                "METADATA",
                "ADD BACKGROUND",
            ]
        );
    }

    #[test]
    fn images() {
        assert_eq!(
            converts(&[Jpeg]),
            with_heic(vec!["PDF", "PNG", "WEBP", "BMP", "TIFF", "AVIF", "HEIC"])
        );
        assert_eq!(
            tools(&[Jpeg]),
            ["COMPRESS", "CROP", "EDIT", "ANNOTATE", "REDACT", "METADATA"]
        );
        assert!(tools(&[Png]).contains(&"ADD BACKGROUND"));
        assert!(!converts(&[Png]).contains(&"PNG"));
        assert!(!converts(&[Heic]).contains(&"HEIC"));
        assert!(converts(&[Gif]).contains(&"JPG"));
    }

    #[test]
    fn documents() {
        assert_eq!(
            converts(&[Pdf]),
            ["DOCX", "TXT", "JPG", "PNG", "WEBP", "BMP", "TIFF"]
        );
        assert_eq!(tools(&[Pdf]), ["COMPRESS", "CROP", "REDACT", "METADATA"]);
        assert_eq!(converts(&[Docx]), ["PDF", "TXT"]);
        assert_eq!(tools(&[Docx]), ["METADATA"]);
        assert_eq!(converts(&[Txt]), ["DOCX", "PDF"]);
        assert!(tools(&[Txt]).is_empty());
    }

    #[test]
    fn unknown_files_get_nothing() {
        assert!(for_kinds(&[Unknown]).is_empty());
        assert!(for_kinds(&[]).is_empty());
        assert!(for_kinds(&[Png, Unknown]).is_empty());
    }

    #[test]
    fn mixed_drops_offer_the_common_actions() {
        assert_eq!(
            converts(&[Jpeg, Png]),
            with_heic(vec!["PDF", "WEBP", "BMP", "TIFF", "AVIF", "HEIC"])
        );
        assert!(!tools(&[Jpeg, Png]).contains(&"ADD BACKGROUND"));
        assert_eq!(converts(&[Png, Docx]), ["PDF"]);
        assert_eq!(tools(&[Png, Docx]), ["METADATA"]);
    }

    #[test]
    fn every_action_applies_to_something() {
        let kinds = [
            Jpeg, Png, Webp, Bmp, Tiff, Gif, Ico, Avif, Heic, Pdf, Docx, Txt,
        ];
        for a in all() {
            let heic_unavailable = a.id() == "convert.heic" && !imaging::can_encode(Heic);
            assert!(
                heic_unavailable || kinds.iter().any(|k| a.applicable(*k)),
                "{}",
                a.id()
            );
            assert!(!a.applicable(Unknown), "{}", a.id());
        }
    }

    #[test]
    fn missing_input_is_reported() {
        for a in all() {
            let err = a
                .run(
                    Path::new("/no/such/file.png"),
                    &Options::default(),
                    Path::new("."),
                )
                .unwrap_err();
            assert!(
                matches!(
                    err.code(),
                    "not_found" | "invalid_options" | "not_available"
                ),
                "{}: {err}",
                a.id()
            );
        }
    }
}
