//! One test per image conversion path: every readable fixture to every writable format.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use dropwheel_core::{FileKind, Options, actions, imaging};

fn convert(source: &str, target: &str, expected_size: (u32, u32)) {
    let (dir, input) = common::staged(source, None);
    let before = std::fs::read(&input).unwrap();
    let action = actions::find(&format!("convert.{target}")).unwrap();
    let out = action
        .run(&input, &Options::default(), dir.path())
        .unwrap_or_else(|e| panic!("{source} -> {target}: {e}"));

    assert_eq!(
        std::fs::read(&input).unwrap(),
        before,
        "the original changed"
    );
    let kind = FileKind::detect(&out).unwrap();
    assert_eq!(kind.extension(), target_ext(target));
    assert!(
        out.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("_converted.")
    );
    if imaging::can_decode(kind) && kind != FileKind::Avif {
        let img = imaging::decode(&out, kind).unwrap().image;
        assert_eq!(
            (img.width(), img.height()),
            expected_size,
            "{source} -> {target}"
        );
    }
}

fn target_ext(target: &str) -> &str {
    match target {
        "jpg" => "jpg",
        other => other,
    }
}

macro_rules! paths {
    ($($name:ident: $source:literal -> $target:literal, $w:literal x $h:literal;)*) => {
        $(
            #[test]
            fn $name() {
                convert($source, $target, ($w, $h));
            }
        )*
    };
}

paths! {
    jpg_to_png: "photo.jpg" -> "png", 64 x 48;
    jpg_to_webp: "photo.jpg" -> "webp", 64 x 48;
    jpg_to_bmp: "photo.jpg" -> "bmp", 64 x 48;
    jpg_to_tiff: "photo.jpg" -> "tiff", 64 x 48;
    jpg_to_avif: "photo.jpg" -> "avif", 64 x 48;

    png_to_jpg: "alpha.png" -> "jpg", 64 x 48;
    png_to_webp: "alpha.png" -> "webp", 64 x 48;
    png_to_bmp: "alpha.png" -> "bmp", 64 x 48;
    png_to_tiff: "alpha.png" -> "tiff", 64 x 48;
    png_to_avif: "alpha.png" -> "avif", 64 x 48;

    webp_to_jpg: "alpha.webp" -> "jpg", 64 x 48;
    webp_to_png: "alpha.webp" -> "png", 64 x 48;
    webp_to_bmp: "alpha.webp" -> "bmp", 64 x 48;
    webp_to_tiff: "alpha.webp" -> "tiff", 64 x 48;
    webp_to_avif: "alpha.webp" -> "avif", 64 x 48;

    bmp_to_jpg: "photo.bmp" -> "jpg", 64 x 48;
    bmp_to_png: "photo.bmp" -> "png", 64 x 48;
    bmp_to_webp: "photo.bmp" -> "webp", 64 x 48;
    bmp_to_tiff: "photo.bmp" -> "tiff", 64 x 48;
    bmp_to_avif: "photo.bmp" -> "avif", 64 x 48;

    tiff_to_jpg: "photo.tiff" -> "jpg", 64 x 48;
    tiff_to_png: "photo.tiff" -> "png", 64 x 48;
    tiff_to_webp: "photo.tiff" -> "webp", 64 x 48;
    tiff_to_bmp: "photo.tiff" -> "bmp", 64 x 48;
    tiff_to_avif: "photo.tiff" -> "avif", 64 x 48;

    gif_to_jpg: "photo.gif" -> "jpg", 64 x 48;
    gif_to_png: "photo.gif" -> "png", 64 x 48;
    gif_to_webp: "photo.gif" -> "webp", 64 x 48;
    gif_to_bmp: "photo.gif" -> "bmp", 64 x 48;
    gif_to_tiff: "photo.gif" -> "tiff", 64 x 48;
    gif_to_avif: "photo.gif" -> "avif", 64 x 48;

    ico_to_jpg: "icon.ico" -> "jpg", 32 x 32;
    ico_to_png: "icon.ico" -> "png", 32 x 32;
    ico_to_webp: "icon.ico" -> "webp", 32 x 32;
    ico_to_bmp: "icon.ico" -> "bmp", 32 x 32;
    ico_to_tiff: "icon.ico" -> "tiff", 32 x 32;
}

/// Writing HEIC needs Microsoft's HEVC encoder; where it is missing the wedge is hidden.
#[test]
fn heic_target_follows_the_encoder() {
    let action = actions::find("convert.heic").unwrap();
    assert_eq!(
        action.applicable(FileKind::Jpeg),
        imaging::can_encode(FileKind::Heic)
    );
    if imaging::can_encode(FileKind::Heic) {
        convert("photo.jpg", "heic", (64, 48));
    }
}

#[test]
fn conversions_turn_photos_upright() {
    let (dir, input) = common::staged("photo.jpg", None);
    let out = actions::find("convert.png")
        .unwrap()
        .run(&input, &Options::default(), dir.path())
        .unwrap();
    let img = common::decode(&out);
    assert_eq!(common::quadrant(common::colour_at(&img, 2, 2)), "red");
    assert_eq!(common::quadrant(common::colour_at(&img, 56, 2)), "green");
    assert_eq!(common::quadrant(common::colour_at(&img, 2, 40)), "blue");
}

#[test]
fn transparency_survives_where_the_format_has_it() {
    let (dir, input) = common::staged("alpha.png", None);
    let out = actions::find("convert.webp")
        .unwrap()
        .run(&input, &Options::default(), dir.path())
        .unwrap();
    let img = common::decode(&out).to_rgba8();
    assert!(img.get_pixel(0, 10).0[3] < 10);
    assert!(img.get_pixel(63, 10).0[3] > 245);
}
