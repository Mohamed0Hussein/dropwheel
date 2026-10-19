//! Every file operation Dropwheel can perform, with no UI and no Tauri.
//!
//! The desktop app is a thin layer over this crate: it detects the [`FileKind`] of each
//! dropped file, offers the [`Action`]s that apply to it, and runs the chosen one with an
//! output directory. Actions never overwrite their input or any existing file; see
//! [`output`].

#![forbid(unsafe_code)]

pub mod action;
pub mod actions;
pub mod error;
pub mod imaging;
pub mod kind;
pub mod output;
pub mod paths;

pub use action::{
    Action, ActionGroup, Background, CancelFlag, Compress, Crop, Edit, Options, Params,
};
pub use error::{Error, Result};
/// Re-exported so the app uses the same version for decoded images.
pub use image;
pub use kind::FileKind;
