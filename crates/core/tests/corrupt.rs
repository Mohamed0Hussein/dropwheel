//! Damaged and hostile files: every action must return an error, never panic.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::time::Instant;

use dropwheel_core::{
    Background, Compress, Crop, Edit, FileKind, Options, Params, actions, imaging,
};

const BROKEN: [&str; 6] = [
    "truncated.jpg",
    "truncated.png",
    "garbage.webp",
    "garbage.gif",
    "bomb.png",
    "empty.jpg",
];

fn params_for(id: &str) -> Params {
    match id {
        "compress" => Params::Compress(Compress::Quality { percent: 50 }),
        "crop" => Params::Crop(Crop {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
            ref_width: 64,
            ref_height: 48,
        }),
        "edit" => Params::Edit(Edit {
            rotate: 90,
            ..Edit::default()
        }),
        "add_background" => Params::Background(Background { color: [1, 2, 3] }),
        _ => Params::None,
    }
}

#[test]
fn every_action_refuses_broken_files() {
    for name in BROKEN {
        for action in actions::all() {
            let (dir, input) = common::staged(name, None);
            let result = action.run(&input, &Options::new(params_for(action.id())), dir.path());
            // Most actions do not even apply to these; the rest must fail cleanly, and none
            // may leave a partial file behind.
            if let Ok(out) = result {
                // A truncated JPEG can still decode its top half, and removing metadata
                // copies the bomb's chunks without decoding a single pixel. Both are fine.
                let partial = name.starts_with("truncated");
                let untouched_pixels = action.id() == "metadata" && name == "bomb.png";
                assert!(
                    partial || untouched_pixels,
                    "{} produced {} from {name}",
                    action.id(),
                    out.display()
                );
            }
            let leftovers: Vec<_> = std::fs::read_dir(dir.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
                .collect();
            assert!(leftovers.is_empty(), "{} left {leftovers:?}", action.id());
        }
    }
}

#[test]
fn decompression_bombs_are_refused_without_allocating() {
    let start = Instant::now();
    let err = imaging::decode(&common::fixture("bomb.png"), FileKind::Png)
        .err()
        .unwrap();
    assert_eq!(err.code(), "decode");
    assert!(err.to_string().contains("larger than"), "{err}");
    assert!(start.elapsed().as_secs() < 2);
}

#[test]
fn kinds_of_broken_files() {
    assert_eq!(
        FileKind::detect(&common::fixture("empty.jpg")).unwrap(),
        FileKind::Unknown
    );
    assert_eq!(
        FileKind::detect(&common::fixture("garbage.webp")).unwrap(),
        FileKind::Webp
    );
    assert!(imaging::decode(&common::fixture("garbage.webp"), FileKind::Webp).is_err());
    assert!(imaging::decode(&common::fixture("garbage.gif"), FileKind::Gif).is_err());
}
