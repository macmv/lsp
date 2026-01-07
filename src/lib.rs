#![allow(rustdoc::redundant_explicit_links)] // explicit links are simpler
#![allow(deprecated)] // we need to use the deprecated types

pub mod notification;
pub mod request;

mod types;
pub use types::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Or2<A, B> {
  A(A),
  B(B),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Or3<A, B, C> {
  A(A),
  B(B),
  C(C),
}

#[derive(Serialize, Deserialize, Clone)]
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
