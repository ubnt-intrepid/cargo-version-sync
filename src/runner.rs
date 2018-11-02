use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use failure::{format_err, Fallible};

use crate::manifest::{Config, Manifest};
use crate::replacer::{Replacer, ReplacerContext};

#[derive(Debug)]
pub struct Diff {
    pub file: PathBuf,
    pub content: String,
    pub replaced: String,
    _priv: (),
}

#[derive(Debug)]
pub struct Runner {
    manifest: Manifest,
    manifest_dir: PathBuf,
    replacements: HashMap<PathBuf, Vec<Replacer>>,
}

impl Runner {
    pub fn init() -> Fallible<Self> {
        let manifest_dir = cargo_manifest_dir()?;
        let manifest_path = manifest_dir.join("Cargo.toml");
        let manifest = Manifest::from_file(manifest_path)?
            .ok_or_else(|| format_err!("missing Cargo manifest file"))?;

        let mut runner = Self {
            manifest,
            manifest_dir,
            replacements: HashMap::new(),
        };

        if runner.config().map_or(false, |config| config.use_preset) {
            println!("[cargo-version-sync] use preset");
            runner.build_preset()?;
        } else {
            runner.collect_replacements()?;
        }

        Ok(runner)
    }

    pub fn manifest_dir(&self) -> &Path {
        &self.manifest_dir
    }

    fn config(&self) -> Option<&Config> {
        self.manifest
            .package
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.version_sync.as_ref())
    }

    fn build_preset(&mut self) -> Fallible<()> {
        let file = self.manifest_dir.join("README.md").canonicalize()?;
        self.replacements
            .entry(file.clone())
            .or_insert_with(Default::default)
            .extend(vec![
                Replacer::builtin("markdown")?,
                Replacer::regex(
                    "https://deps.rs/crate/{{name}}/[0-9a-z\\.-]+",
                    "https://deps.rs/crate/{{name}}/{{version}}",
                ),
            ]);

        let file = self.manifest_dir.join("src/lib.rs").canonicalize()?;
        self.replacements
            .entry(file.clone())
            .or_insert_with(Default::default)
            .extend(vec![Replacer::builtin("html-root-url")?]);

        Ok(())
    }

    fn collect_replacements(&mut self) -> Fallible<()> {
        if let Some(ref parsed_replacements) = self
            .manifest
            .package
            .metadata
            .as_ref()
            .and_then(|meta| meta.version_sync.as_ref())
            .map(|version_sync| &version_sync.replacements)
        {
            for replace in parsed_replacements.iter() {
                let file = self.manifest_dir.join(&replace.file).canonicalize()?;
                self.replacements
                    .entry(file.clone())
                    .or_insert_with(Default::default)
                    .extend(replace.replacers.clone());
            }
        }

        Ok(())
    }

    pub fn collect_diffs(&self) -> Fallible<Vec<Diff>> {
        let mut diffs = Vec::new();

        for (path, replacers) in &self.replacements {
            if !path.is_file() {
                continue;
            }

            let content = fs::read_to_string(&path)?;

            let replaced = {
                let mut cx = ReplacerContext::new(content.as_str(), &self.manifest);
                for replacer in replacers {
                    replacer.replace(&mut cx)?;
                }
                cx.finish().into_owned()
            };

            if content != replaced {
                diffs.push(Diff {
                    file: path.clone(),
                    content,
                    replaced,
                    _priv: (),
                });
            }
        }

        Ok(diffs)
    }

    pub fn relative_file_path<'a>(&self, diff: &'a Diff) -> Fallible<&'a Path> {
        diff.file
            .strip_prefix(&self.manifest_dir)
            .map_err(Into::into)
    }
}

#[cfg(feature = "show-diff")]
mod show_diff {
    use super::*;

    use colored::Colorize;
    use difference::{Changeset, Difference};

    impl super::Runner {
        pub fn show_diff(&self, diff: &Diff) -> Fallible<()> {
            let file = diff.file.strip_prefix(&self.manifest_dir)?;
            let changeset = Changeset::new(&diff.content, &diff.replaced, "\n");

            println!("{}:", file.display());

            let mut line = 1;
            for diff in &changeset.diffs {
                match diff {
                    Difference::Same(ref x) => line += bytecount::count(x.as_ref(), b'\n') + 1,
                    Difference::Rem(ref x) => {
                        println!("  - ({:>4}) {}", line, x.to_string().red());
                        line += 1;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
    }
}

fn cargo_manifest_dir() -> Fallible<PathBuf> {
    match std::env::var_os("CARGO_MANIFEST_DIR") {
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
