extern crate chrono;
extern crate failure;
extern crate regex;
extern crate serde;
extern crate toml;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

use failure::{format_err, Fallible};
use serde::Deserialize;

use crate::parse::{Manifest, Replacement};

fn cargo_manifest_dir() -> Fallible<PathBuf> {
    match env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => {
            let current_dir = std::env::current_dir()?;
            let mut current_dir: &Path = &current_dir;
            loop {
                if current_dir.join("Cargo.toml").is_file() {
                    return Ok(current_dir.to_owned());
                }
                current_dir = match current_dir.parent() {
                    Some(parent) => parent,
                    None => {
                        return Err(failure::format_err!("The cargo manifest file is not found"))
                    }
                }
            }
        }
    }
}

fn collect_replacements(manifest: &Manifest) -> Fallible<Vec<Replacement>> {
    use std::collections::HashMap;
    let mut collected: HashMap<PathBuf, Replacement> = HashMap::new();

    if let Some(ref replacements) = manifest
        .package
        .metadata
        .as_ref()
        .and_then(|meta| meta.version_sync.as_ref())
        .map(|version_sync| &version_sync.replacements)
    {
        for replace in replacements.iter() {
            let file = replace.file.canonicalize()?;
            collected
                .entry(file.clone())
                .or_insert_with(|| Replacement {
                    file,
                    patterns: vec![],
                }).patterns
                .extend(replace.patterns.clone());
        }
    }

    Ok(collected.into_iter().map(|(_k, v)| v).collect())
}

fn replace_all_in_place(re: &regex::Regex, text: &mut Cow<'_, str>, rep: impl regex::Replacer) {
    *text = match mem::replace(text, Cow::Borrowed("<dummy>")) {
        Cow::Borrowed(content) => re.replace_all(content, rep),
        Cow::Owned(owned_content) => match re.replace_all(&owned_content, rep) {
            Cow::Borrowed(..) => Cow::Owned(owned_content),
            Cow::Owned(replaced) => Cow::Owned(replaced),
        },
    }
}

#[derive(Debug)]
pub struct Runner {
    manifest: Manifest,
    replacements: Vec<Replacement>,
}

impl Runner {
    pub fn init() -> Fallible<Self> {
        let manifest_path = cargo_manifest_dir()?.join("Cargo.toml");
        let manifest = Manifest::from_file(manifest_path)?
            .ok_or_else(|| format_err!("missing Cargo manifest file"))?;

        let replacements = collect_replacements(&manifest)?;

        Ok(Self {
            manifest,
            replacements,
        })
    }
}

#[derive(Debug)]
pub struct Changeset {
    pub file: PathBuf,
    pub content: String,
    pub replaced: String,
    _priv: (),
}

impl Changeset {
    pub fn dump(&self) -> Fallible<()> {
        fs::write(&self.file, &self.replaced).map_err(Into::into)
    }
}

impl Runner {
    fn handle_replacement(&self, replace: &Replacement) -> Fallible<Option<Changeset>> {
        if !replace.file.is_file() {
            return Ok(None);
        }

        let content = fs::read_to_string(&replace.file)?;

        let replaced = {
            let mut replaced = Cow::Borrowed(content.as_str());
            for pattern in &replace.patterns {
                let re = pattern.build_regex()?;
                let rep = pattern.replaced_text(self.manifest.crate_version());
                replace_all_in_place(&re, &mut replaced, rep.as_str());
            }
            replaced.into_owned()
        };

        if content != replaced {
            Ok(Some(Changeset {
                file: replace.file.clone(),
                content,
                replaced,
                _priv: (),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn run(&self) -> Fallible<Vec<Changeset>> {
        let mut changeset = Vec::new();
        for replace in &self.replacements {
            changeset.extend(self.handle_replacement(replace)?);
        }
        Ok(changeset)
    }
}

mod parse {
    use failure::Fallible;
    use serde::Deserialize;
    use std::io;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Deserialize)]
    pub struct Manifest {
        pub package: Package,
    }

    impl Manifest {
        pub fn from_file(path: impl AsRef<Path>) -> Fallible<Option<Self>> {
            let content = match std::fs::read_to_string(path) {
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
        pub fn build_regex(&self) -> Fallible<regex::Regex> {
            regex::Regex::new(&self.search).map_err(Into::into)
        }

        pub fn replaced_text(&self, version: &str) -> String {
            let date = chrono::Local::now();
            self.replace
                .replace("{{version}}", version)
                .replace("{{date}}", &date.format("%Y-%m-%d").to_string())
        }
    }
}

pub fn assert_sync() {
    let manifest_dir = cargo_manifest_dir().unwrap();
    match Runner::init().and_then(|runner| runner.run()) {
        Ok(changeset) => {
            if !changeset.is_empty() {
                eprintln!("The version number(s) are not synced in the following files:\n");
                for change in changeset {
                    eprintln!(
                        "  - {}",
                        change.file.strip_prefix(&manifest_dir).unwrap().display()
                    );
                }
                eprintln!();
                panic!();
            }
        }
        Err(err) => panic!("error: {}", err),
    }
}
