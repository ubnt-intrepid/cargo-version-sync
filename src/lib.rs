#![doc(html_root_url = "https://docs.rs/cargo-version-sync/0.0.5")]

extern crate chrono;
extern crate failure;
extern crate regex;
extern crate serde;
extern crate toml;

mod assert;
mod changeset;
mod manifest;
mod replacer;
pub mod runner;

pub use crate::assert::assert_version_sync;
