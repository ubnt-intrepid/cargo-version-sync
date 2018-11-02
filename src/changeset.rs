use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Changeset<'a> {
    pub diffs: Vec<Diff>,
    pub manifest_dir: &'a Path,
}

impl<'a> fmt::Display for Changeset<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "The version number(s) are not synced in the following files:"
        )?;
        writeln!(f)?;
        for diff in &self.diffs {
            let file = diff
                .file
                .strip_prefix(self.manifest_dir)
                .expect("should be a valid relative path");
            writeln!(f, "  - {}", file.display())?;
        }
        writeln!(f)
    }
}

#[derive(Debug)]
pub struct Diff {
    pub file: PathBuf,
    pub content: String,
    pub replaced: String,
}
