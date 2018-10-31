use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use failure::{format_err, Fallible};

use crate::manifest::{Manifest, Replacement};
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
    replacements: Vec<Replacement>,
}

impl Runner {
    pub fn init() -> Fallible<Self> {
        let manifest_dir = cargo_manifest_dir()?;
        let manifest_path = manifest_dir.join("Cargo.toml");
        let manifest = Manifest::from_file(manifest_path)?
            .ok_or_else(|| format_err!("missing Cargo manifest file"))?;

        let replacements = if manifest
            .package
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.version_sync.as_ref())
            .map_or(false, |v| v.use_preset)
        {
            println!("[cargo-version-sync] use preset");
            build_replacement_preset(&manifest_dir)?
        } else {
            collect_replacements(&manifest, &manifest_dir)?
        };

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
            let mut content = Cow::Borrowed(content.as_str());
            let mut cx = ReplacerContext::new(&self.manifest.package);
            for replacer in &replace.replacers {
                content = match std::mem::replace(&mut content, Cow::Borrowed("<dummy>")) {
                    Cow::Borrowed(b) => replacer.replace(&mut cx, b)?,
                    Cow::Owned(o) => match replacer.replace(&mut cx, &content)? {
                        Cow::Borrowed(..) => Cow::Owned(o),
                        Cow::Owned(o) => Cow::Owned(o),
                    },
                };
            }
            content.into_owned()
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

fn build_replacement_preset(manifest_dir: impl AsRef<Path>) -> Fallible<Vec<Replacement>> {
    let mut preset = vec![];

    preset.push(Replacement {
        file: manifest_dir.as_ref().join("README.md").canonicalize()?,
        replacers: vec![
            Replacer::builtin("markdown")?,
            Replacer::regex(
                "https://deps.rs/crate/{{name}}/[0-9a-z\\.-]+",
                "https://deps.rs/crate/{{name}}/{{version}}",
            ),
        ],
    });

    preset.push(Replacement {
        file: manifest_dir.as_ref().join("src/lib.rs").canonicalize()?,
        replacers: vec![Replacer::builtin("html-root-url")?],
    });

    Ok(preset)
}

fn collect_replacements(
    manifest: &Manifest,
    manifest_dir: impl AsRef<Path>,
) -> Fallible<Vec<Replacement>> {
    let mut collected: HashMap<PathBuf, Replacement> = HashMap::new();

    if let Some(ref replacements) = manifest
        .package
        .metadata
        .as_ref()
        .and_then(|meta| meta.version_sync.as_ref())
        .map(|version_sync| &version_sync.replacements)
    {
        for replace in replacements.iter() {
            let file = manifest_dir.as_ref().join(&replace.file).canonicalize()?;
            collected
                .entry(file.clone())
                .or_insert_with(|| Replacement {
                    file,
                    replacers: vec![],
                }).replacers
                .extend(replace.replacers.clone());
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
