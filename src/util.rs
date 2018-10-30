use failure::Fallible;
use std::borrow::Cow;
use std::env;
use std::mem;
use std::path::{Path, PathBuf};

pub fn cargo_manifest_dir() -> Fallible<PathBuf> {
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

pub fn replace_all_in_place(re: &regex::Regex, text: &mut Cow<'_, str>, rep: impl regex::Replacer) {
    *text = match mem::replace(text, Cow::Borrowed("<dummy>")) {
        Cow::Borrowed(content) => re.replace_all(content, rep),
        Cow::Owned(owned_content) => match re.replace_all(&owned_content, rep) {
            Cow::Borrowed(..) => Cow::Owned(owned_content),
            Cow::Owned(replaced) => Cow::Owned(replaced),
        },
    }
}
