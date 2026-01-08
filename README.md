# Language Server Protocol for Rust

[![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/lsp.svg
[crates.io]: https://crates.io/crates/lsp

This crate exposes all types defined in the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification).

Most of the code here is generated from the LSP [meta model](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json). The generator is written in the `lsp-generator` crate.

## Features

- `raw_value`: Enables `serde_json/raw_value`, and uses a `Box<RawValue>` for `LspAny` instead of a `serde_json::Value`.

## Building

The generator isn't part of a buildscript, so you need to run it manually. This is intentional, as downstream crates shouldn't regenerate the schema.

```rs
cargo run -p lsp-generator --release
```
