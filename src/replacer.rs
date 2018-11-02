use std::borrow::Cow;
use std::mem;

use failure::format_err;
use failure::Fallible;
use regex::Regex;
use serde::Deserialize;

use crate::manifest::Manifest;

pub struct ReplacerContext<'a> {
    manifest: &'a Manifest,
    date: chrono::DateTime<chrono::Local>,
    text: Cow<'a, str>,
}

impl<'a> ReplacerContext<'a> {
    pub fn new(text: &'a str, manifest: &'a Manifest) -> Self {
        Self {
            manifest,
            date: chrono::Local::now(),
            text: Cow::Borrowed(text),
        }
    }

    pub fn package_name(&self) -> &str {
        &self.manifest.package.name
    }

    pub fn package_version(&self) -> &str {
        &self.manifest.package.version
    }

    pub fn date(&self) -> chrono::DateTime<chrono::Local> {
        self.date
    }

    pub fn text(&mut self) -> &mut Cow<'a, str> {
        &mut self.text
    }

    pub fn finish(self) -> Cow<'a, str> {
        self.text
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Replacer {
    #[serde(rename = "regex")]
    Regex(RegexReplacer),
    #[serde(rename = "builtin")]
    Builtin(BuiltinReplacer),
}

impl Replacer {
    pub fn builtin(target: &str) -> Fallible<Replacer> {
        match target {
            "markdown" => Ok(Replacer::Builtin(BuiltinReplacer {
                target: "markdown".into(),
            })),
            "html-root-url" => Ok(Replacer::Builtin(BuiltinReplacer {
                target: "html-root-url".into(),
            })),
            target => Err(failure::format_err!("invalid builtin target: {}", target)),
        }
    }

    pub fn regex(search: &str, replace: &str) -> Replacer {
        Replacer::Regex(RegexReplacer {
            search: search.into(),
            replace: replace.into(),
        })
    }

    pub fn replace(&self, cx: &mut ReplacerContext) -> Fallible<()> {
        match self {
            Replacer::Regex(re) => re.replace(cx),
            Replacer::Builtin(builtin) => builtin.replace(cx),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegexReplacer {
    search: String,
    replace: String,
}

impl RegexReplacer {
    fn replace(&self, cx: &mut ReplacerContext<'_>) -> Fallible<()> {
        let re = {
            let search = self.search.replace("{{name}}", &cx.package_name());
            Regex::new(&search)?
        };

        let rep = {
            self.replace
                .replace("{{name}}", cx.package_name())
                .replace("{{version}}", cx.package_version())
                .replace("{{date}}", &cx.date().format("%Y-%m-%d").to_string())
        };

        *cx.text() = match mem::replace(cx.text(), Cow::Borrowed("<dummy>")) {
            Cow::Borrowed(content) => re.replace_all(content, rep.as_str()),
            Cow::Owned(owned_content) => match re.replace_all(&owned_content, rep.as_str()) {
                Cow::Borrowed(..) => Cow::Owned(owned_content),
                Cow::Owned(replaced) => Cow::Owned(replaced),
            },
        };
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuiltinReplacer {
    target: String,
}

impl BuiltinReplacer {
    fn replace(&self, cx: &mut ReplacerContext<'_>) -> Fallible<()> {
        let replacer = match &*self.target {
            "markdown" => {
                // TODO: parse Markdown codeblock as TOML
                RegexReplacer {
                    search: "{{name}} = \"[0-9a-z\\.-]+\"".into(),
                    replace: "{{name}} = \"{{version}}\"".into(),
                }
            }
            "html-root-url" => RegexReplacer {
                search: "https://docs.rs/{{name}}/[0-9a-z\\.-]+".into(),
                replace: "https://docs.rs/{{name}}/{{version}}".into(),
            },
            s => return Err(format_err!("unsupported target: {}", s)),
        };

        replacer.replace(cx)
    }
}
