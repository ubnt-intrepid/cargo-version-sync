use failure::format_err;
use failure::Fallible;
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;

pub use self::context::ReplacerContext;

mod context {
    use crate::manifest::Package;

    pub struct ReplacerContext<'a> {
        package: &'a Package,
        date: chrono::DateTime<chrono::Local>,
    }

    impl<'a> ReplacerContext<'a> {
        pub fn new(package: &'a Package) -> Self {
            Self {
                package,
                date: chrono::Local::now(),
            }
        }

        pub fn package_name(&self) -> &str {
            &self.package.name
        }

        pub fn package_version(&self) -> &str {
            &self.package.version
        }

        pub fn date(&self) -> chrono::DateTime<chrono::Local> {
            self.date
        }
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

    pub fn replace<'t>(
        &self,
        cx: &mut ReplacerContext<'_>,
        text: &'t str,
    ) -> Fallible<Cow<'t, str>> {
        match self {
            Replacer::Regex(re) => re.replace(cx, text),
            Replacer::Builtin(builtin) => builtin.replace(cx, text),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegexReplacer {
    search: String,
    replace: String,
}

impl RegexReplacer {
    fn replace<'t>(&self, cx: &mut ReplacerContext<'_>, text: &'t str) -> Fallible<Cow<'t, str>> {
        let mut text = Cow::Borrowed(text);
        self.replace_in_place(cx, &mut text)?;
        Ok(text)
    }

    fn replace_in_place(
        &self,
        cx: &mut ReplacerContext<'_>,
        text: &mut Cow<'_, str>,
    ) -> Fallible<()> {
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
        *text = match std::mem::replace(text, Cow::Borrowed("<dummy>")) {
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
    fn replace<'t>(&self, cx: &mut ReplacerContext<'_>, text: &'t str) -> Fallible<Cow<'t, str>> {
        match &*self.target {
            "markdown" => {
                // TODO: parse Markdown codeblock as TOML
                (RegexReplacer {
                    search: "{{name}} = \"[0-9a-z\\.-]+\"".into(),
                    replace: "{{name}} = \"{{version}}\"".into(),
                }).replace(cx, text)
            }
            "html-root-url" => (RegexReplacer {
                search: "https://docs.rs/{{name}}/[0-9a-z\\.-]+".into(),
                replace: "https://docs.rs/{{name}}/{{version}}".into(),
            }).replace(cx, text),
            s => Err(format_err!("unsupported target: {}", s)),
        }
    }
}
