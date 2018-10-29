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

1. Add fields to `Cargo.toml` for specifying files to rewrite the version numbers by `cargo version-sync`:

```toml
[[package.metadata.version-sync.replacements]]
file = "README.md"
patterns = [
  { search = "https://deps.rs/crate/tsukuyomi/[a-z0-9\\.-]+", replace = "https://deps.rs/crate/tsukuyomi/{{version}}" },
]
```

2. Run `cargo-version-sync` to rewrite version numbers:

```shell-session
$ cargo version-sync [--verbose] [--dry-run]
```

## Working with custom Git hooks

```toml
[dev-dependencies.cargo-husky]
version = "1"
default-features = false
features = ["user-hooks"]
```

in `.cargo-husky/hooks/pre-commit`:

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

## Working with `cargo test`

in `tests/version_sync.rs`:

```rust
extern crate cargo_version_sync;

#[test]
fn test_version_sync() {
    cargo_version_sync::assert_sync();
}
```

## Alternatives

* https://github.com/mgeisler/version-sync

## License

[MIT license](./LICENSE)
