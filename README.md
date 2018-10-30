# `cargo-version-sync`

[![Crates.io](https://img.shields.io/crates/v/cargo-version-sync.svg)](https://crates.io/crates/cargo-version-sync)
[![Docs.rs](https://docs.rs/cargo-version-sync/badge.svg)](https://docs.rs/cargo-version-sync)
[![Build Status](https://travis-ci.org/ubnt-intrepid/cargo-version-sync.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/cargo-version-sync)

Cargo subcommand for keeping the version numbers in sync with `Cargo.toml`

## Status

Exeperimental

## Installation

```shell-session
$ cargo install cargo-version-sync
```

## Usage

At first, add fields to `Cargo.toml` for specifying files to rewrite the version numbers by `cargo version-sync`:

```toml
[[package.metadata.version-sync.replacements]]
file = "README.md"
replacers = [
  { type = "builtin", target = "markdown" },
  { type = "regex", search = "https://deps.rs/crate/{{name}}/[a-z0-9\\.-]+", replace = "https://deps.rs/crate//{{name}}/{{version}}" },
]

[[package.metadata.version-sync.replacements]]
file = "src/lib.rs"
replacers = [
  { type = "builtin", target = "html-root-url" },
]
```

Then run the command `cargo version-sync` to rewrite version numbers:

```shell-session
$ cargo version-sync
```

## Integration test


```toml
[dev-dependencies]
cargo-version-sync = { version = "0.0.2", default-features = false }
```

Then, add a test case in your integration test as follows:

```rust
extern crate cargo_version_sync;

#[test]
fn test_version_sync() {
    cargo_version_sync::assert_version_sync();
}
```

```command
$ cargo test
...
running 1 test
test test_version_sync ... FAILED

failures:

---- test_version_sync stdout ----
The version number(s) are not synced in the following files:

  - README.md
...
```

## Pre-commit check

in `.git/hooks/pre-commit`:

```sh
#!/bin/bash

set -e

if cargo fmt --version >/dev/null 2>&1; then
    (set -x; cargo fmt -- --check)
fi

if cargo version-sync --version >/dev/null 2>&1; then
    (set -x; cargo version-sync --check)
fi
```

You can automatically install the custom Git hooks by using [`cargo-husky`].

```toml
[dev-dependencies.cargo-husky]
version = "1"
default-features = false
features = ["user-hooks"]
```

[`cargo-husky`]: https://github.com/rhysd/cargo-husky.git

## Alternatives

* https://github.com/mgeisler/version-sync

## License

[MIT license](./LICENSE)

<!--
```toml
cargo-version-sync = "0.0.2"
```
-->
