#![doc(html_root_url = "https://docs.rs/cargo-version-sync/0.0.3")]

extern crate chrono;
extern crate failure;
extern crate regex;
extern crate serde;
extern crate toml;

#[cfg(feature = "show-diff")]
extern crate bytecount;
#[cfg(feature = "show-diff")]
extern crate colored;
#[cfg(feature = "show-diff")]
extern crate difference;

mod assert;
mod manifest;
mod replacer;
pub mod runner;

pub use crate::assert::assert_version_sync;
