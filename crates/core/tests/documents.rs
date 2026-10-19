//! One test per document conversion path, and the tools on PDF and DOCX files.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::{Path, PathBuf};

use dropwheel_core::doc::Block;
use dropwheel_core::{
    Compress, Crop, FileKind, Options, Params, Redact, Region, actions, docx, imaging, pdf,
};

fn run(id: &str, input: &Path, params: Params) -> PathBuf {
    let before = std::fs::read(input).unwrap();
    let out = actions::find(id)
        .unwrap()
        .run(input, &Options::new(params), input.parent().unwrap())
        .unwrap_or_else(|e| panic!("{id} on {}: {e}", input.display()));
    assert_eq!(
        std::fs::read(input).unwrap(),
        before,
        "{id} changed its input"
    );
    out
}

fn convert(source: &str, target: &str) -> (tempfile::TempDir, PathBuf) {
    let (dir, input) = common::staged(source, None);
    let out = run(&format!("convert.{target}"), &input, Params::None);
    (dir, out)
}

fn colour_of_page(path: &Path, fx: f32, fy: f32) -> [u8; 3] {
    let img = pdf::render_page(path, 0, 400).unwrap();
    let (x, y) = (
        (img.width() as f32 * fx) as u32,
        (img.height() as f32 * fy) as u32,
    );
    img.get_pixel(x, y).0
}

macro_rules! image_to_pdf {
    ($($name:ident: $source:literal;)*) => {
        $(
            #[test]
            fn $name() {
                let (_dir, out) = convert($source, "pdf");
                assert!(out.ends_with(concat!(stringify!($name), "")) || out.extension().unwrap() == "pdf");
                assert_eq!(FileKind::detect(&out).unwrap(), FileKind::Pdf);
                if !common::pdfium() {
                    return;
                }
                let sizes = pdf::page_sizes(&out).unwrap();
                assert_eq!(sizes.len(), 1);
                let (w, h) = sizes[0];
                // A4, turned to suit the picture, which sits in the middle.
                assert!((w.max(h) - 841.9).abs() < 1.0 && (w.min(h) - 595.3).abs() < 1.0);
                assert_eq!(colour_of_page(&out, 0.02, 0.02), [255, 255, 255]);
                let centre = colour_of_page(&out, 0.5, 0.5);
                assert_ne!(centre, [255, 255, 255], "{}", $source);
            }
        )*
    };
}

image_to_pdf! {
    jpg_to_pdf: "photo.jpg";
    png_to_pdf: "alpha.png";
    webp_to_pdf: "alpha.webp";
    bmp_to_pdf: "photo.bmp";
    tiff_to_pdf: "photo.tiff";
    gif_to_pdf: "photo.gif";
    ico_to_pdf: "icon.ico";
}

#[test]
fn photos_are_upright_in_the_pdf() {
    if !common::pdfium() {
        return;
    }
    let (_dir, out) = convert("photo.jpg", "pdf");
    // The upright photo is landscape, so the page is too.
    let (w, h) = pdf::page_sizes(&out).unwrap()[0];
    assert!(w > h);
    let img = pdf::render_page(&out, 0, 842).unwrap();
    // The picture is 64x48 px at 96 dpi: 48x36 pt, centred on the page.
    let (cx, cy) = (img.width() / 2, img.height() / 2);
    let at = |dx: i32, dy: i32| {
        let p = img
            .get_pixel((cx as i32 + dx) as u32, (cy as i32 + dy) as u32)
            .0;
        common::quadrant(p)
    };
    assert_eq!(at(-12, -9), "red");
    assert_eq!(at(12, -9), "green");
    assert_eq!(at(-12, 9), "blue");
}

macro_rules! pdf_to_image {
    ($($name:ident: $target:literal;)*) => {
        $(
            #[test]
            fn $name() {
                if !common::pdfium() {
                    return;
                }
                // One page: one file.
                let (_dir, out) = convert("scan.pdf", $target);
                let kind = FileKind::detect(&out).unwrap();
                assert_eq!(kind.extension(), $target);
                let img = imaging::decode(&out, kind).unwrap().image;
                // The scan is 160x120 px at 36 dpi: 320x240 pt, so 889x667 px at 200 dpi.
                assert!((img.width() as i32 - 889).abs() <= 2, "{}", img.width());
                assert_eq!(common::quadrant(common::colour_at(&img, 100, 100)), "red");

                // Two pages: a folder with a file each.
                let (_dir, out) = convert("text.pdf", $target);
                assert!(out.is_dir());
                assert!(out.ends_with("text_pages"));
                let mut names: Vec<String> = std::fs::read_dir(&out)
                    .unwrap()
                    .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                    .collect();
                names.sort();
                assert_eq!(
                    names,
                    [format!("text_page1.{}", $target), format!("text_page2.{}", $target)]
                );
            }
        )*
    };
}

pdf_to_image! {
    pdf_to_jpg: "jpg";
    pdf_to_png: "png";
    pdf_to_webp: "webp";
    pdf_to_bmp: "bmp";
    pdf_to_tiff: "tiff";
}

#[test]
fn pdf_to_txt() {
    if !common::pdfium() {
        return;
    }
    let (_dir, out) = convert("text.pdf", "txt");
    assert!(out.ends_with("text_converted.txt"));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Quarterly report\r\n"), "{text:?}");
    assert!(text.contains("Page two has a picture below."));
    let (_dir, out) = convert("arabic.pdf", "txt");
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("مرحبا بالعالم")
    );
}

#[test]
fn pdf_to_docx() {
    if !common::pdfium() {
        return;
    }
    let (_dir, out) = convert("text.pdf", "docx");
    assert!(out.ends_with("text_converted.docx"));
    let doc = docx::read::read(&out).unwrap();
    let text = doc.to_text();
    assert!(text.contains("Quarterly report"), "{text}");
    assert!(
        text.contains("Dropwheel turns this page into text. Second line of the first paragraph.")
    );
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::Picture(_))));
    assert!(doc.blocks.contains(&Block::PageBreak));

    // A scan becomes a picture of the page.
    let (_dir, out) = convert("scan.pdf", "docx");
    let doc = docx::read::read(&out).unwrap();
    assert!(
        matches!(doc.blocks.as_slice(), [Block::Picture(_)]),
        "{:?}",
        doc.blocks.len()
    );
}

#[test]
fn docx_to_pdf() {
    let (_dir, out) = convert("report.docx", "pdf");
    assert!(out.ends_with("report_converted.pdf"));
    if !common::pdfium() {
        return;
    }
    // The page break in the document starts a second page.
    assert_eq!(pdf::page_sizes(&out).unwrap().len(), 2);
    let text = common::pdf_text(&out);
    for expected in [
        "Quarterly report",
        "Plain text, then bold, italic and red words.",
        "• First point",
        "1. Step one",
        "Region",
        "+4%",
        "مرحبا بالعالم. هذا تقرير ربع سنوي.",
    ] {
        assert!(
            text.contains(expected),
            "{expected:?} missing from {text:?}"
        );
    }
}

#[test]
fn docx_to_txt() {
    let (_dir, out) = convert("report.docx", "txt");
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Quarterly report\r\n"));
    assert!(text.contains("• First point\r\n"));
    assert!(text.contains("2. Step two\r\n"));
    assert!(text.contains("Region\tSales\tChange\r\n"));
    assert!(text.contains("مرحبا بالعالم"));
}

#[test]
fn txt_to_docx() {
    for (source, expected) in [
        ("notes.txt", "ملاحظة: لا تنسَ الخبز"),
        ("legacy.txt", "مرحبا بالعالم"),
        ("utf16.txt", "Ünïcode in UTF-16"),
    ] {
        let (_dir, out) = convert(source, "docx");
        let doc = docx::read::read(&out).unwrap();
        let text = doc.to_text();
        assert!(text.contains(expected), "{source}: {text:?}");
    }
    let (_dir, out) = convert("notes.txt", "docx");
    let text = docx::read::read(&out).unwrap().to_text();
    assert!(text.starts_with("Shopping list\r\n\tmilk\r\n"), "{text:?}");
}

#[test]
fn txt_to_pdf() {
    let (_dir, out) = convert("notes.txt", "pdf");
    if !common::pdfium() {
        return;
    }
    let text = common::pdf_text(&out);
    assert!(text.contains("Shopping list"), "{text:?}");
    assert!(text.contains("ملاحظة: لا تنسَ الخبز"), "{text:?}");
    let (_dir, out) = convert("legacy.txt", "pdf");
    assert!(common::pdf_text(&out).contains("هذا ملف قديم"));
}

#[test]
fn compress_a_pdf() {
    if !common::pdfium() {
        return;
    }
    let (_dir, input) = common::staged("text.pdf", None);
    let out = run(
        "compress",
        &input,
        Params::Compress(Compress::Quality { percent: 50 }),
    );
    assert!(out.ends_with("text_compressed.pdf"));
    assert_eq!(pdf::page_sizes(&out).unwrap().len(), 2);
    assert!(common::pdf_text(&out).contains("Quarterly report"));
}

#[test]
fn crop_a_pdf() {
    if !common::pdfium() {
        return;
    }
    let (_dir, input) = common::staged("text.pdf", None);
    // The top half of the page, drawn on a 595 x 842 preview.
    let crop = Crop {
        x: 0,
        y: 0,
        width: 595,
        height: 421,
        ref_width: 595,
        ref_height: 842,
    };
    let out = run("crop", &input, Params::Crop(crop));
    assert!(out.ends_with("text_cropped.pdf"));
    let sizes = pdf::page_sizes(&out).unwrap();
    assert!((sizes[0].1 - 421.0).abs() < 2.0, "{sizes:?}");
    assert!(common::pdf_text(&out).contains("Quarterly report"));

    // A rotated page is cropped as it is shown: its left half.
    let (_dir, input) = common::staged("rotated.pdf", None);
    let crop = Crop {
        x: 0,
        y: 0,
        width: 150,
        height: 200,
        ref_width: 300,
        ref_height: 200,
    };
    let out = run("crop", &input, Params::Crop(crop));
    assert_eq!(pdf::page_sizes(&out).unwrap(), [(150.0, 200.0)]);
    let img = pdf::render_page(&out, 0, 150).unwrap();
    assert_eq!(img.get_pixel(10, 10).0, [255, 0, 0]);
}

#[test]
fn redact_a_pdf() {
    if !common::pdfium() {
        return;
    }
    let (_dir, input) = common::staged("text.pdf", None);
    // The top fifth of page one, where the title and first lines are.
    let redact = Redact {
        regions: vec![Region {
            page: 0,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 0.2,
        }],
    };
    let out = run("redact", &input, Params::Redact(redact));
    assert!(out.ends_with("text_redacted.pdf"));
    let sizes = pdf::page_sizes(&out).unwrap();
    assert_eq!(sizes.len(), 2);
    assert!((sizes[0].0 - 595.3).abs() < 1.0);
    // The pages are pictures now: no text at all, and black where the box was.
    assert!(common::pdf_text(&out).trim().is_empty());
    // The pages are JPEG pictures, so allow for a little noise.
    let near = |c: [u8; 3], v: u8| c.iter().all(|x| x.abs_diff(v) < 12);
    assert!(near(colour_of_page(&out, 0.2, 0.1), 0));
    assert!(near(colour_of_page(&out, 0.5, 0.5), 255));
    // Page two had no box.
    let page_two = pdf::render_page(&out, 1, 200).unwrap();
    assert!(near(page_two.get_pixel(20, 20).0, 255));
}

#[test]
fn pdf_and_docx_metadata() {
    use dropwheel_core::actions::metadata;
    for (name, field) in [("text.pdf", "Author"), ("report.docx", "Author")] {
        let (_dir, input) = common::staged(name, None);
        let fields = metadata::read(&input).unwrap();
        assert!(
            fields
                .iter()
                .any(|f| f.name == field && f.value == "Test Author"),
            "{name}: {fields:#?}"
        );
        let out = run("metadata", &input, Params::None);
        assert!(
            out.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("_clean.")
        );
        let fields = metadata::read(&out).unwrap();
        assert!(
            !fields.iter().any(|f| f.name == field),
            "{name}: {fields:#?}"
        );
    }
}

#[test]
fn protected_and_broken_pdfs_fail_cleanly() {
    let pdfium = common::pdfium();
    for name in ["protected.pdf", "broken.pdf"] {
        for action in actions::all() {
            let kind = FileKind::Pdf;
            if !action.applicable(kind) || (!pdfium && action.id().starts_with("convert.")) {
                continue;
            }
            let params = match action.id() {
                "compress" => Params::Compress(Compress::Quality { percent: 50 }),
                "crop" => Params::Crop(Crop {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 10,
                    ref_width: 100,
                    ref_height: 100,
                }),
                "redact" => Params::Redact(Redact {
                    regions: vec![Region {
                        page: 0,
                        x: 0.0,
                        y: 0.0,
                        width: 0.5,
                        height: 0.5,
                    }],
                }),
                _ => Params::None,
            };
            if action.id() == "redact" && !pdfium {
                continue;
            }
            let (dir, input) = common::staged(name, None);
            let err = action
                .run(&input, &Options::new(params), dir.path())
                .expect_err(&format!("{} on {name}", action.id()));
            let expected = if name == "protected.pdf" {
                "protected"
            } else {
                "decode"
            };
            assert_eq!(err.code(), expected, "{} on {name}: {err}", action.id());
            let leftovers: Vec<_> = std::fs::read_dir(dir.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() != name)
                .collect();
            assert!(leftovers.is_empty(), "{} left {leftovers:?}", action.id());
        }
    }
}

#[test]
fn broken_word_files_fail_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    // A zip that is not a Word document, and a Word document cut short.
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("hello.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    std::io::Write::write_all(&mut zip, b"hi").unwrap();
    let not_word = dir.path().join("not word.docx");
    std::fs::write(&not_word, zip.finish().unwrap().into_inner()).unwrap();
    let report = std::fs::read(common::fixture("report.docx")).unwrap();
    let cut = dir.path().join("cut.docx");
    std::fs::write(&cut, &report[..report.len() / 2]).unwrap();

    for input in [&not_word, &cut] {
        let kind = FileKind::detect(input).unwrap();
        for action in actions::all().iter().filter(|a| a.applicable(kind)) {
            let err = action
                .run(input, &Options::default(), dir.path())
                .expect_err(&format!("{} on {}", action.id(), input.display()));
            assert_eq!(
                err.code(),
                "decode",
                "{} on {}: {err}",
                action.id(),
                input.display()
            );
        }
        let read = dropwheel_core::actions::metadata::read(input);
        assert!(read.is_err() || read.is_ok_and(|f| f.is_empty()));
    }
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    assert_eq!(leftovers.len(), 2, "{leftovers:?}");
}
