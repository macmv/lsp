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

mod uri;
pub use uri::{Uri, UriError};

use serde::{Deserialize, Serialize};

/// Represents a union of two types. The first variant will take priority when
/// deserializing (using `#[serde(untagged)]` behavior). Additionally, the first
/// variant is the default.
///
/// This relies on the LSP spec putting the "default" variant first. Ordering is
/// maintained between unions in the LSP spec and the order of generic arguments
/// in generated code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Or2<A, B> {
  A(A),
  B(B),
}

/// Represents a union of three types. The first variant will take priority when
/// deserializing (using `#[serde(untagged)]` behavior). Additionally, the first
/// variant is the default.
///
/// This relies on the LSP spec putting the "default" variant first. Ordering is
/// maintained between unions in the LSP spec and the order of generic arguments
/// in generated code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Or3<A, B, C> {
  A(A),
  B(B),
  C(C),
}

impl<A: Default, B> Default for Or2<A, B> {
  fn default() -> Self { Or2::A(A::default()) }
}

impl<A: Default, B, C> Default for Or3<A, B, C> {
  fn default() -> Self { Or3::A(A::default()) }
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum WorkDoneProgress {
  Begin(WorkDoneProgressBegin),
  Report(WorkDoneProgressReport),
  End(WorkDoneProgressEnd),
}

#[cfg(test)]
mod tests {
  use serde::de::DeserializeOwned;

  use super::*;

  macro_rules! assert_serde {
    ($value:expr, $expected:literal as $ty:ty) => {
      assert_eq!(ser($value), $expected);
      // This is dumb but it works.
      assert_eq!(format!("{:?}", de::<$ty>($expected)), format!("{:?}", $value));
    };
  }

  fn ser<T: Serialize>(t: &T) -> String { serde_json::to_string(t).unwrap() }
  fn de<T: DeserializeOwned>(t: &str) -> T { serde_json::from_str(t).unwrap() }

  #[test]
  fn string_enums_work() {
    assert_serde!(
      &[
        PositionEncodingKind::Utf8,
        PositionEncodingKind::Utf16,
        PositionEncodingKind::Custom("foo".into())
      ],
      r#"["utf-8","utf-16","foo"]"# as Vec<PositionEncodingKind>
    );
  }

  #[test]
  fn or2_works() {
    assert_serde!(
      &TextDocumentSyncOptions { save: Some(Or2::A(true)), ..Default::default() },
      r#"{"save":true}"# as TextDocumentSyncOptions
    );

    assert_serde!(
      &TextDocumentSyncOptions { save: Some(Or2::B(SaveOptions::default())), ..Default::default() },
      r#"{"save":{}}"# as TextDocumentSyncOptions
    );

    assert_serde!(
      &TextDocumentSyncOptions {
        save: Some(Or2::B(SaveOptions { include_text: Some(true), ..Default::default() })),
        ..Default::default()
      },
      r#"{"save":{"includeText":true}}"# as TextDocumentSyncOptions
    );
  }
}
