use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use failure::Fallible;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub package: Package,
}

impl Manifest {
    pub fn from_file(path: impl AsRef<Path>) -> Fallible<Option<Self>> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        toml::from_str(&content).map(Some).map_err(Into::into)
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(rename = "version-sync")]
    pub version_sync: Option<Config>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub replacements: Vec<Replacement>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Replacement {
    pub file: PathBuf,
    pub replacers: Vec<Replacer>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Replacer {
    #[serde(rename = "regex")]
    Regex(RegexReplacer),
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegexReplacer {
    pub search: String,
    pub replace: String,
}
