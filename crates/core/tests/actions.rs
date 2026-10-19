//! Every image action on real files.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;

use dropwheel_core::actions::metadata;
use dropwheel_core::{
    Background, Compress, Crop, Edit, FileKind, Options, Params, actions, imaging,
};

fn run(id: &str, input: &Path, params: Params) -> std::path::PathBuf {
    let dir = input.parent().unwrap();
    let before = std::fs::read(input).unwrap();
    let out = actions::find(id)
        .unwrap()
        .run(input, &Options::new(params), dir)
        .unwrap_or_else(|e| panic!("{id} on {}: {e}", input.display()));
    assert_eq!(
        std::fs::read(input).unwrap(),
        before,
        "{id} changed its input"
    );
    assert_ne!(out, input);
    out
}

#[test]
fn compress_by_quality_makes_a_smaller_jpeg() {
    let (_dir, input) = common::staged("photo.jpg", None);
    let out = run(
        "compress",
        &input,
        Params::Compress(Compress::Quality { percent: 30 }),
    );
    assert!(out.ends_with("photo_compressed.jpg"));
    let (a, b) = (
        std::fs::metadata(&input).unwrap().len(),
        std::fs::metadata(&out).unwrap().len(),
    );
    assert!(b < a, "{b} >= {a}");
}

#[test]
fn compress_to_a_target_size() {
    for name in ["photo.jpg", "alpha.png", "alpha.webp", "photo.tiff"] {
        let (_dir, input) = common::staged(name, None);
        let target = std::fs::metadata(&input).unwrap().len() / 2;
        let out = run(
            "compress",
            &input,
            Params::Compress(Compress::Size { bytes: target }),
        );
        let size = std::fs::metadata(&out).unwrap().len();
        assert!(size <= target, "{name}: {size} > {target}");
        assert_eq!(
            FileKind::detect(&out).unwrap(),
            FileKind::detect(&input).unwrap()
        );
    }
}

#[test]
fn crop_cuts_the_rectangle_from_the_upright_image() {
    let (_dir, input) = common::staged("photo.jpg", None);
    // The top-left (red) quadrant of the upright 64x48 picture.
    let crop = Crop {
        x: 0,
        y: 0,
        width: 32,
        height: 24,
        ref_width: 64,
        ref_height: 48,
    };
    let out = run("crop", &input, Params::Crop(crop));
    assert!(out.ends_with("photo_cropped.jpg"));
    let img = common::decode(&out);
    assert_eq!((img.width(), img.height()), (32, 24));
    assert_eq!(common::quadrant(common::colour_at(&img, 13, 9)), "red");
}

#[test]
fn edit_rotates_and_flips() {
    let (_dir, input) = common::staged("alpha.png", None);
    let edit = Edit {
        rotate: 90,
        ..Edit::default()
    };
    let out = run("edit", &input, Params::Edit(edit));
    let img = common::decode(&out);
    assert_eq!((img.width(), img.height()), (48, 64));
    // Rotated clockwise, the bottom-left (blue) quadrant comes to the top left.
    assert_eq!(common::quadrant(common::colour_at(&img, 2, 2)), "blue");
    assert!(img.color().has_alpha());
}

#[test]
fn background_fills_transparency() {
    let (_dir, input) = common::staged("alpha.png", None);
    let out = run(
        "add_background",
        &input,
        Params::Background(Background { color: [0, 0, 0] }),
    );
    let img = common::decode(&out);
    assert!(!img.color().has_alpha());
    // The left edge was fully transparent: now black.
    assert_eq!(img.to_rgb8().get_pixel(0, 5).0, [0, 0, 0]);
}

#[test]
fn metadata_is_listed() {
    let fields = metadata::read(&common::fixture("photo.jpg")).unwrap();
    let has = |name: &str, value: &str| {
        fields
            .iter()
            .any(|f| f.name == name && f.value.contains(value))
    };
    assert!(has("Make", "Dropwheel Test"), "{fields:#?}");
    assert!(has("Artist", "Test Author"));
    assert!(fields.iter().any(|f| f.group == "Location"));
    assert!(fields.iter().any(|f| f.group == "XMP"));
    assert!(fields.iter().any(|f| f.group == "Comment"));

    let png = metadata::read(&common::fixture("alpha.png")).unwrap();
    assert!(
        png.iter()
            .any(|f| f.name == "Description" && f.value == "تجربة")
    );
}

#[test]
fn removing_metadata_keeps_the_picture() {
    for name in ["photo.jpg", "alpha.png", "alpha.webp", "photo.tiff"] {
        let (_dir, input) = common::staged(name, None);
        assert!(
            !metadata::read(&input).unwrap().is_empty(),
            "{name} has no metadata to start"
        );
        let out = run("metadata", &input, Params::None);
        assert!(
            out.to_str().unwrap().contains("_clean."),
            "{}",
            out.display()
        );
        let left = metadata::read(&out).unwrap();
        assert!(left.is_empty(), "{name}: {left:#?}");
        let kind = FileKind::detect(&out).unwrap();
        let img = imaging::decode(&out, kind).unwrap().image;
        assert_eq!(
            (img.width(), img.height()).0.max(img.height()),
            64,
            "{name}"
        );
    }
}

#[test]
fn removing_metadata_from_a_jpeg_does_not_re_encode_it() {
    let (_dir, input) = common::staged("photo.jpg", None);
    let out = run("metadata", &input, Params::None);
    let a = std::fs::metadata(&input).unwrap().len();
    let b = std::fs::metadata(&out).unwrap().len();
    // Only metadata went: the compressed picture data is byte for byte the same.
    assert!(b < a && a - b < 1200, "{a} -> {b}");
    let pixels = |p: &Path| common::decode(p).to_rgb8().into_raw();
    assert_eq!(pixels(&input), pixels(&out));
}

#[test]
fn arabic_names_and_long_folders() {
    let dir = tempfile::tempdir().unwrap();
    let mut deep = dir.path().to_path_buf();
    for i in 0..10 {
        deep.push(format!("مجلد طويل جدا رقم {i}"));
    }
    std::fs::create_dir_all(&deep).unwrap();
    let input = deep.join("صورة العائلة.jpg");
    std::fs::copy(common::fixture("photo.jpg"), &input).unwrap();
    assert!(input.as_os_str().len() > 260);

    let out = run("convert.png", &input, Params::None);
    assert_eq!(
        out.file_name().unwrap().to_str().unwrap(),
        "صورة العائلة_converted.png"
    );
    let again = run("convert.png", &input, Params::None);
    assert_eq!(
        again.file_name().unwrap().to_str().unwrap(),
        "صورة العائلة_converted (2).png"
    );
}

#[test]
fn wrong_settings_are_rejected_not_guessed() {
    let (_dir, input) = common::staged("photo.jpg", None);
    let err = actions::find("crop")
        .unwrap()
        .run(
            &input,
            &Options::new(Params::Edit(Edit::default())),
            input.parent().unwrap(),
        )
        .unwrap_err();
    assert_eq!(err.code(), "invalid_options");
}
