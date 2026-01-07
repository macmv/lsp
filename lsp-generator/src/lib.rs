use crate::{generator::Generator, spec::*};

const URL: &str = "https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json";

mod generator;
mod spec;

pub fn generate() {
  let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

  generate_requests(&mut Generator::new("src/request.rs"), &spec.requests);
  generate_notifications(&mut Generator::new("src/notification.rs"), &spec.notifications);

  let mut g = Generator::new("src/lib.rs");
  g.writeln("use serde::{Deserialize, Serialize, de, ser};");
  g.writeln("use std::{collections::HashMap, fmt};");
  g.writeln("");
  g.writeln("pub mod request;");
  g.writeln("pub mod notification;");

  g.writeln("");
  g.writeln("#[derive(Serialize, Deserialize)]");
  g.writeln("#[serde(untagged)]");
  g.writeln("pub enum Or<A, B> {");
  g.writeln("A(A),");
  g.writeln("B(B),");
  g.writeln("}");

  for ty in &spec.structures {
    generate_struct(&mut g, ty);
  }

  for ty in &spec.enumerations {
    generate_enum(&mut g, ty);
  }

  while g.has_types() {
    let types = g.drain_types();

    for (name, ty) in types {
      g.writeln("#[derive(Serialize, Deserialize)]");
      g.writeln(format_args!("pub struct {} {{", name));
      for prop in &ty.properties {
        g.write_doc(&prop.documentation);
        if prop.name == "type" {
          g.write("#[serde(rename = \"type\")]");
          g.write("ty: ");
        } else {
          if to_snake_case(&prop.name) != prop.name {
            g.write(format_args!("#[serde(rename = \"{}\")]", prop.name));
          }
          g.write(format_args!("pub {}: ", to_snake_case(&prop.name)));
        }
        write_type(&mut g, &prop.ty);
        g.writeln(",");
      }
      g.writeln("}");
    }
  }
}

fn generate_struct(g: &mut Generator, ty: &Structure) {
  g.writeln("");
  g.write_doc(&ty.documentation);
  g.writeln("#[derive(Serialize, Deserialize)]");
  g.writeln(format_args!("pub struct {} {{", ty.name));
  for field in ty.properties.iter() {
    g.write_doc(&field.documentation);
    if field.name == "type" {
      g.write("#[serde(rename = \"type\")]");
      g.write("ty: ");
    } else {
      if to_snake_case(&field.name) != field.name {
        g.write(format_args!("#[serde(rename = \"{}\")]", field.name));
      }
      g.write(format_args!("pub {}: ", to_snake_case(&field.name)));
    }
    write_type(g, &field.ty);
    g.writeln(",");
  }
  g.writeln(format_args!("}}"));
}

fn generate_enum(g: &mut Generator, ty: &Enumeration) {
  g.writeln("");
  g.write_doc(&ty.documentation);
  match ty.ty {
    Type::Base { name: BaseType::String } => {
      g.writeln("#[derive(Serialize, Deserialize)]");
      g.writeln("#[serde(untagged)]");
    }
    Type::Base { name: BaseType::Integer | BaseType::Uinteger } => {
      g.writeln("#[derive(Clone, Copy)]");
    }

    _ => panic!("invalid enum type: {:#?}", ty.ty),
  }

  g.writeln(format_args!("pub enum {} {{", ty.name));
  for variant in ty.values.iter() {
    g.write_doc(&variant.documentation);

    match &variant.value {
      NumberOrString::Number(n) => {
        if ty.supports_custom_values {
          g.writeln(format_args!("{},", to_pascal_case(&variant.name)));
        } else {
          g.writeln(format_args!("{} = {},", to_pascal_case(&variant.name), n));
        }
      }
      NumberOrString::String(s) => {
        g.writeln(format_args!("#[serde(rename = \"{}\")]", s));
        g.writeln(format_args!("{},", to_pascal_case(&variant.name)));
      }
    }
  }

  if ty.supports_custom_values {
    match ty.ty {
      Type::Base { name: BaseType::String } => g.writeln("Custom(String),"),
      Type::Base { name: BaseType::Integer } => g.writeln("Custom(i32),"),
      Type::Base { name: BaseType::Uinteger } => g.writeln("Custom(u32),"),

      _ => unreachable!(),
    }
  }

  g.writeln(format_args!("}}"));

  match ty.ty {
    Type::Base { name: BaseType::Uinteger | BaseType::Integer } => {
      let signed = ty.ty == Type::Base { name: BaseType::Integer };
      let num = if signed { "i32" } else { "u32" };

      if ty.supports_custom_values {
        g.writeln(format_args!("impl {} {{", ty.name));
        g.writeln(format_args!("pub fn as_{num}(self) -> {num} {{",));
        g.writeln("match self {");
        for variant in &ty.values {
          match variant.value {
            NumberOrString::Number(n) => {
              g.writeln(format_args!("Self::{} => {n},", to_pascal_case(&variant.name)));
            }
            _ => unreachable!(),
          }
        }
        g.writeln("Self::Custom(value) => value,");
        g.writeln("}");
        g.writeln("}");
        g.writeln("}");

        g.writeln(format_args!("impl From<{num}> for {} {{", ty.name));
        g.writeln(format_args!("fn from(value: {num}) -> Self {{"));
        g.writeln("match value {");
        for variant in &ty.values {
          match variant.value {
            NumberOrString::Number(n) => {
              g.writeln(format_args!("{n} => Self::{},", to_pascal_case(&variant.name)));
            }
            _ => unreachable!(),
          }
        }
        g.writeln("_ => Self::Custom(value)");
        g.writeln("}");
        g.writeln("}");
        g.writeln("}");
      } else {
        g.writeln(format_args!("impl {} {{", ty.name));
        g.writeln(format_args!("pub fn as_{num}(self) -> {num} {{",));
        g.writeln(format_args!("self as {num}"));
        g.writeln("}");
        g.writeln("}");

        g.writeln(format_args!("impl TryFrom<{num}> for {} {{", ty.name));
        g.writeln("type Error = ();");
        g.writeln(format_args!("fn try_from(value: {num}) -> Result<Self, ()> {{"));
        g.writeln("match value {");
        for variant in &ty.values {
          match variant.value {
            NumberOrString::Number(n) => {
              g.writeln(format_args!("{n} => Ok(Self::{}),", to_pascal_case(&variant.name)));
            }
            _ => unreachable!(),
          }
        }
        g.writeln("_ => Err(())");
        g.writeln("}");
        g.writeln("}");
        g.writeln("}");
      }

      g.writeln(format_args!("impl Serialize for {} {{", ty.name));
      g.writeln("fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>");
      g.writeln("  where S: ser::Serializer {");
      g.writeln(format_args!("serializer.serialize_{num}(self.as_{num}())"));
      g.writeln("}");
      g.writeln("}");

      g.writeln(format_args!("impl<'de> Deserialize<'de> for {} {{", ty.name));
      g.writeln("fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>");
      g.writeln("  where D: de::Deserializer<'de> {");
      g.writeln("struct Visitor;");

      g.writeln("impl<'de> de::Visitor<'de> for Visitor {");
      g.writeln(format_args!("type Value = {};", ty.name));
      g.writeln("fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {");
      if signed {
        g.writeln("formatter.write_str(\"integer\")");
      } else {
        g.writeln("formatter.write_str(\"unsigned integer\")");
      }
      g.writeln("}");
      g.writeln(format_args!("fn visit_u64<E>(self, value: u64) -> Result<{}, E>", ty.name));
      g.writeln("  where E: de::Error {");
      if ty.supports_custom_values {
        g.writeln(format_args!("if let Ok(n) = {num}::try_from(value) {{"));
        g.writeln(format_args!("Ok({}::from(n))", ty.name));
      } else {
        g.writeln(format_args!("if let Ok(n) = {num}::try_from(value)"));
        g.writeln(format_args!("  && let Ok(v) = {}::try_from(n) {{", ty.name));
        g.writeln(format_args!("Ok(v)"));
      }
      g.writeln(format_args!("}} else {{"));
      g.writeln(format_args!(
        "Err(de::Error::invalid_value(de::Unexpected::Unsigned(value), &self))"
      ));
      g.writeln(format_args!("}}"));
      g.writeln(format_args!("}}"));
      g.writeln(format_args!("fn visit_i64<E>(self, value: i64) -> Result<{}, E>", ty.name));
      g.writeln("  where E: de::Error {");
      if ty.supports_custom_values {
        g.writeln(format_args!("if let Ok(n) = {num}::try_from(value) {{"));
        g.writeln(format_args!("Ok({}::from(n))", ty.name));
      } else {
        g.writeln(format_args!("if let Ok(n) = {num}::try_from(value)"));
        g.writeln(format_args!("  && let Ok(v) = {}::try_from(n) {{", ty.name));
        g.writeln(format_args!("Ok(v)"));
      }
      g.writeln(format_args!("}} else {{"));
      g.writeln(format_args!(
        "Err(de::Error::invalid_value(de::Unexpected::Signed(value), &self))"
      ));
      g.writeln(format_args!("}}"));
      g.writeln(format_args!("}}"));
      g.writeln("}");

      g.writeln(format_args!("let n = deserializer.deserialize_{num}(Visitor)?;",));
      g.writeln("Ok(Self::from(n))");

      g.writeln("}");
      g.writeln("}");
    }

    _ => {}
  }
}

fn generate_requests(g: &mut Generator, requests: &[Request]) {
  g.writeln("//! LSP Requests.");
  g.writeln("");
  g.writeln("use super::*;");
  g.writeln("");

  g.writeln("pub trait Request {");
  g.writeln("const METHOD: &'static str;");
  g.writeln("type Params;");
  g.writeln("type Result;");
  g.writeln("}");
  g.writeln("");

  for n in requests {
    g.write_doc(&n.documentation);
    let name = notification_name(&n.method);
    g.writeln(format_args!("pub enum {name} {{}}"));

    g.writeln(format_args!("impl Request for {name} {{"));
    g.writeln(format_args!("const METHOD: &'static str = \"{}\";", n.method));

    g.write(format_args!("type Params = "));
    write_type(g, &n.params.as_ref().unwrap_or(&Type::Base { name: BaseType::Null }));
    g.writeln(format_args!(";"));

    g.write(format_args!("type Result = "));
    write_type(g, &n.result);
    g.writeln(format_args!(";"));

    g.writeln(format_args!("}}"));
  }
}

fn generate_notifications(g: &mut Generator, notifications: &[Notification]) {
  g.writeln("//! LSP Notifications.");
  g.writeln("");
  g.writeln("use super::*;");
  g.writeln("");

  g.writeln("pub trait Notification {");
  g.writeln("const METHOD: &'static str;");
  g.writeln("type Params;");
  g.writeln("}");
  g.writeln("");

  for n in notifications {
    g.write_doc(&n.documentation);
    let name = notification_name(&n.method);
    g.writeln(format_args!("pub enum {name} {{}}"));

    g.writeln(format_args!("impl Notification for {name} {{"));
    g.writeln(format_args!("const METHOD: &'static str = \"{}\";", n.method));

    g.write(format_args!("type Params = "));
    write_type(g, &n.params.as_ref().unwrap_or(&Type::Base { name: BaseType::Null }));
    g.writeln(format_args!(";"));

    g.writeln(format_args!("}}"));
  }
}

fn write_type(g: &mut Generator, ty: &Type) {
  match ty {
    Type::Base { name } => match name {
      BaseType::Null => g.write("()"),
      BaseType::Boolean => g.write("bool"),
      BaseType::Integer => g.write("i32"),
      BaseType::Uinteger => g.write("u32"),
      BaseType::String => g.write("String"),
      BaseType::Decimal => g.write("f64"),
      BaseType::DocumentUri => g.write("String"),
      BaseType::Uri => g.write("String"),
    },
    Type::Reference { name } if name == "LSPAny" => g.write("serde_json::Value"),
    Type::Reference { name } => g.write(name),

    Type::Or { items } => {
      if items.len() == 1 {
        write_type(g, &items[0]);
      } else if items.iter().any(|item| item == &Type::Base { name: BaseType::Null }) {
        g.write("Option<");
        write_type(
          g,
          &Type::Or {
            items: items
              .iter()
              .filter(|item| *item != &Type::Base { name: BaseType::Null })
              .cloned()
              .collect(),
          },
        );
        g.write(">");
      } else if items.len() == 2 {
        g.write("Or<");
        for (i, item) in items.iter().enumerate() {
          if i != 0 {
            g.write(", ");
          }
          write_type(g, item);
        }
        g.write(">");
      } else {
        g.write("/* TODO */");
        g.write("Or<");
        for (i, item) in items.iter().enumerate() {
          if i != 0 {
            g.write(", ");
          }
          write_type(g, item);
        }
        g.write(">");
      }
    }

    Type::Array { element } => {
      g.write("Vec<");
      write_type(g, element);
      g.write(">");
    }

    Type::Tuple { items } => {
      g.write("(");
      for (i, item) in items.iter().enumerate() {
        if i != 0 {
          g.write(", ");
        }
        write_type(g, item);
      }
      g.write(")");
    }

    Type::Map { key, value } => {
      g.write("HashMap<");
      write_type(g, key);
      g.write(", ");
      write_type(g, value);
      g.write(">");
    }

    Type::StringLiteral { value } => {
      g.write(format_args!("String /* \"{}\" */", value));
    }

    Type::Literal { value } => {
      if value.properties.is_empty() {
        write_type(g, &Type::Base { name: BaseType::Null });
      } else {
        let name = value.properties.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join("");

        g.write(&name);
        g.add_type(name, value.clone());
      }
    }
  }
}

fn to_snake_case(name: &str) -> String {
  let mut snake_case = String::new();

  for ch in name.chars() {
    if ch.is_ascii_uppercase() {
      snake_case.push('_');
      snake_case.push(ch.to_ascii_lowercase());
    } else {
      snake_case.push(ch);
    }
  }

  snake_case
}

fn to_pascal_case(name: &str) -> String { name[0..1].to_ascii_uppercase() + &name[1..] }

fn notification_name(method: &str) -> String {
  let mut name = String::new();
  let mut capitalize = true;
  for c in method.chars() {
    match c {
      'a'..='z' if capitalize => {
        name.push(c.to_ascii_uppercase());
        capitalize = false;
      }
      'a'..='z' => name.push(c),
      'A'..='Z' => name.push(c),
      _ => capitalize = true,
    }
  }

  name
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_spec() { ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap(); }

  #[test]
  fn notification_name_works() {
    assert_eq!(
      notification_name("textDocument/publishDiagnostics"),
      "TextDocumentPublishDiagnostics"
    );
  }
}
