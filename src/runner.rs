use failure::{format_err, Fallible};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::{Manifest, Replacement};

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
    replacements: Vec<Replacement>,
}

impl Runner {
    pub fn init() -> Fallible<Self> {
        let manifest_dir = crate::util::cargo_manifest_dir()?;
        let manifest_path = manifest_dir.join("Cargo.toml");
        let manifest = Manifest::from_file(manifest_path)?
            .ok_or_else(|| format_err!("missing Cargo manifest file"))?;

        let replacements = collect_replacements(&manifest)?;

        Ok(Self {
            manifest,
            manifest_dir,
            replacements,
        })
    }

    fn handle_replacement(&self, replace: &Replacement) -> Fallible<Option<Diff>> {
        if !replace.file.is_file() {
            return Ok(None);
        }

        let content = fs::read_to_string(&replace.file)?;

        let replaced = {
            let mut replaced = Cow::Borrowed(content.as_str());
            for pattern in &replace.patterns {
                let re = pattern.build_regex()?;
                let rep = pattern.replaced_text(self.manifest.crate_version());
                crate::util::replace_all_in_place(&re, &mut replaced, rep.as_str());
            }
            replaced.into_owned()
        };

        if content != replaced {
            Ok(Some(Diff {
                file: replace.file.clone(),
                content,
                replaced,
                _priv: (),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn collect_diffs(&self) -> Fallible<Vec<Diff>> {
        let mut changeset = Vec::new();
        for replace in &self.replacements {
            changeset.extend(self.handle_replacement(replace)?);
        }
        Ok(changeset)
    }

    pub fn manifest_dir(&self) -> &Path {
        &self.manifest_dir
    }
}

impl Runner {
    pub fn relative_file_path<'a>(&self, diff: &'a Diff) -> Fallible<&'a Path> {
        diff.file
            .strip_prefix(&self.manifest_dir)
            .map_err(Into::into)
    }
}

fn collect_replacements(manifest: &Manifest) -> Fallible<Vec<Replacement>> {
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
