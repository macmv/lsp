# Language Server Protocol for Rust

This crate exposes all types defined in the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specification) for [Rust](https://www.rust-lang.org/).

Most of the code here is generated from the LSP [meta model](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json). The generator is written in the `lsp-generator` crate.

## Building

The generator isn't part of a buildscript, so you need to run it manually. This is intentional, as downstream crates shouldn't regenerate the schema.

```rs
cargo run -p lsp-generator --release
```
