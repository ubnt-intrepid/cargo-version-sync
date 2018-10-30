use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Local;
use failure::Fallible;
use regex::Regex;
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

    pub fn crate_version(&self) -> &str {
        &self.package.version
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
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
    pub patterns: Vec<Pattern>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Pattern {
    pub search: String,
    pub replace: String,
}

impl Pattern {
    pub fn build_regex(&self) -> Fallible<Regex> {
        Regex::new(&self.search).map_err(Into::into)
    }

    pub fn replaced_text(&self, version: &str) -> String {
        let date = Local::now();
        self.replace
            .replace("{{version}}", version)
            .replace("{{date}}", &date.format("%Y-%m-%d").to_string())
    }
}
