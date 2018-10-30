use failure::Fallible;
use std::fmt;
use std::path::PathBuf;

use crate::runner::Runner;

pub fn assert_version_sync() {
    match inner() {
        Ok(Some(changeset)) => {
            eprintln!("{}", changeset);
            panic!();
        }
        Ok(None) => {}
        Err(e) => panic!("error during checking version sync: {}", e),
    }
}

fn inner() -> Fallible<Option<Changeset>> {
    let runner = Runner::init().unwrap();
    let diffs = runner.collect_diffs()?;
    if diffs.is_empty() {
        return Ok(None);
    }

    Ok(Some(Changeset {
        files: diffs
            .iter()
            .map(|diff| {
                diff.file
                    .strip_prefix(runner.manifest_dir())
                    .map(ToOwned::to_owned)
            }).collect::<Result<_, _>>()?,
    }))
}

struct Changeset {
    files: Vec<PathBuf>,
}

impl fmt::Display for Changeset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "The version number(s) are not synced in the following files:"
        )?;
        writeln!(f)?;
        for file in &self.files {
            writeln!(f, "  - {}", file.display())?;
        }
        writeln!(f)
    }
}
