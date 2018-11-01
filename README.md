# `cargo-version-sync`

[![Crates.io](https://img.shields.io/crates/v/cargo-version-sync.svg)](https://crates.io/crates/cargo-version-sync)
[![Docs.rs](https://docs.rs/cargo-version-sync/badge.svg)](https://docs.rs/cargo-version-sync)
[![Build Status](https://travis-ci.org/ubnt-intrepid/cargo-version-sync.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/cargo-version-sync)
[![dependency status](https://deps.rs/crate/cargo-version-sync/0.0.2/status.svg)](https://deps.rs/crate/cargo-version-sync/0.0.2)

Cargo subcommand for keeping the version numbers in sync with `Cargo.toml`

## Status

Experimental

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
  { type = "regex", search = "https://deps.rs/crate/{{name}}/[a-z0-9\\.-]+", replace = "https://deps.rs/crate/{{name}}/{{version}}" },
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

## Using Preset

You can use the preset replacers by setting the key `use-preset`.

```toml
[package.metadata.version-sync]
use-preset = true
```

Currently, replacers are registered for the following files:

* `file = "README.md"`
  - `{ type = "builtin", target = "markdown" }`
  - `{ type = "regex", search = "https://deps.rs/crate/{{name}}/[0-9a-z\\.-]+", replace = "https://deps.rs/crate/{{name}}/{{version}}" }`
* `file = "src/lib.rs"`
  - `{ type = "builtin", target = "html-root-url" }`

## Integration test

`cargo-version-sync` can also be used as a library used in integration tests.
First, add the dependency to the member of `[dev-dependencies]`:

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

When there are some files that have not updated the version numbers,
the integration test fails as follows:

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

## Alternatives

* https://github.com/mgeisler/version-sync

## License

[MIT license](./LICENSE)

<!--
```toml
cargo-version-sync = "0.0.4"
```
-->
