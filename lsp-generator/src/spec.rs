#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
  pub meta_data:     Metadata,
  pub requests:      Vec<Request>,
  pub notifications: Vec<Notification>,
  pub structures:    Vec<Structure>,
  pub enumerations:  Vec<Enumeration>,
  pub type_aliases:  Vec<TypeAlias>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
  pub version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  pub method:            String,
  pub result:            Type,
  pub message_direction: MessageDirection,
  pub params:            Option<Type>,
  pub partial_result:    Option<Type>,
  #[serde(default)]
  pub documentation:     String,
  #[serde(default)]
  pub since:             Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
  pub method:            String,
  pub message_direction: MessageDirection,
  pub params:            Option<Type>,
  #[serde(default)]
  pub documentation:     String,
  #[serde(default)]
  pub since:             Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Structure {
  pub name:          String,
  pub properties:    Vec<Property>,
  #[serde(default)]
  pub extends:       Vec<Type>,
  #[serde(default)]
  pub mixins:        Vec<Type>,
  #[serde(default)]
  pub documentation: String,
  #[serde(default)]
  pub since:         Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enumeration {
  pub name:                   String,
  #[serde(rename = "type")]
  pub ty:                     Type,
  #[serde(default)]
  pub values:                 Vec<EnumValue>,
  #[serde(default)]
  pub supports_custom_values: bool,
  #[serde(default)]
  pub documentation:          String,
  #[serde(default)]
  pub since:                  Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
  pub name:          String,
  pub value:         NumberOrString,
  #[serde(default)]
  pub documentation: String,
  #[serde(default)]
  pub since:         Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NumberOrString {
  Number(i64),
  String(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeAlias {
  pub name:          String,
  #[serde(rename = "type")]
  pub ty:            Type,
  #[serde(default)]
  pub documentation: String,
  #[serde(default)]
  pub since:         Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDirection {
  ClientToServer,
  ServerToClient,
  Both,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Property {
  pub name:          String,
  #[serde(default)]
  pub optional:      bool,
  #[serde(default)]
  pub documentation: String,
  #[serde(rename = "type")]
  pub ty:            Type,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Type {
  Base { name: BaseType },
  Reference { name: String },
  Or { items: Vec<Type> },
  Literal { value: Literal },
  StringLiteral { value: String },
  Map { key: Box<Type>, value: Box<Type> },
  Array { element: Box<Type> },
  Tuple { items: Vec<Type> },
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Literal {
  pub properties: Vec<Property>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum BaseType {
  #[serde(rename = "null")]
  Null,
  #[serde(rename = "string")]
  String,
  #[serde(rename = "uinteger")]
  Uinteger,
  #[serde(rename = "integer")]
  Integer,
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "decimal")]
  Decimal,
  #[serde(rename = "DocumentUri")]
  DocumentUri,
  #[serde(rename = "URI")]
  Uri,
}

impl Type {
  pub fn is_null(&self) -> bool {
    matches!(self, Type::Base { name: BaseType::Null })
      || matches!(self, Type::Literal { value: Literal { properties } } if properties.is_empty())
  }
}
