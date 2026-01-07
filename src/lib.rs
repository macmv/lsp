//! # Language Server Protocol (LSP)
//!
//! This crate provides types for the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
//! It makes no assumptions about the transport layer (e.g. TCP, Unix sockets,
//! etc.), or if the consumer is a client or server.
//!
//! This crate uses [`serde`] for all encodable types.
//!
//! # Version
//!
//! Almost all types in this crate are generated from the [LSP
//! model](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json).
//! This code was generated from LSP version
#![doc = concat!("**", include_str!("lsp_version.txt"), "**.")]
//!

#![allow(rustdoc::redundant_explicit_links)] // explicit links are simpler
#![allow(deprecated)] // we need to use the deprecated types

pub mod notification;
pub mod request;

mod types;
pub use types::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Or2<A, B> {
  A(A),
  B(B),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Or3<A, B, C> {
  A(A),
  B(B),
  C(C),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Or4<A, B, C, D> {
  A(A),
  B(B),
  C(C),
  D(D),
}

impl<A: Default, B> Default for Or2<A, B> {
  fn default() -> Self { Or2::A(A::default()) }
}

impl<A: Default, B, C> Default for Or3<A, B, C> {
  fn default() -> Self { Or3::A(A::default()) }
}

impl<A: Default, B, C, D> Default for Or4<A, B, C, D> {
  fn default() -> Self { Or4::A(A::default()) }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ser<T: Serialize>(t: &T) -> String { serde_json::to_string(t).unwrap() }

  #[test]
  fn string_enums_work() {
    assert_eq!(
      ser(&[
        PositionEncodingKind::Utf8,
        PositionEncodingKind::Utf16,
        PositionEncodingKind::Custom("foo".into())
      ]),
      r#"["utf-8","utf-16","foo"]"#
    );
  }
}
