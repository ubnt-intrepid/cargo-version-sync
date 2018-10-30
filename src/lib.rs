#![doc(html_root_url = "https://docs.rs/cargo-version-sync/0.0.2")]

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
pub mod runner;
mod util;

pub use crate::assert::assert_version_sync;
