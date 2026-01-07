use std::collections::HashMap;

use crate::{generator::Generator, names::Names, spec::*};

const URL: &str = "https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json";

mod generator;
mod names;
mod spec;

pub fn main() {
  let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

  let names = Names::from_spec(&spec);

  std::fs::write("src/lsp_version.txt", spec.meta_data.version).unwrap();

  generate_requests(&mut Generator::new("src/request.rs", &names), &spec.requests);
  generate_notifications(&mut Generator::new("src/notification.rs", &names), &spec.notifications);

  let mut g = Generator::new("src/types.rs", &names);
  g.writeln("use serde::{Deserialize, Serialize, de, ser};");
  g.writeln("use std::{collections::HashMap, fmt};");
  g.writeln("");
  g.writeln("use crate::*;");

  let structs =
    spec.structures.iter().map(|ty| (ty.name.as_str(), ty)).collect::<HashMap<&str, &Structure>>();

  for ty in &spec.structures {
    generate_struct(&mut g, ty, &structs);
  }

  for ty in &spec.enumerations {
    generate_enum(&mut g, ty);
  }

  for ty in &spec.type_aliases {
    generate_type_alias(&mut g, ty);
  }

  while g.has_types() {
    let types = g.drain_types();

    for (name, ty) in types {
      g.writeln("#[derive(Serialize, Deserialize, Default, Clone)]");
      g.writeln(format_args!("pub struct {} {{", name));
      generate_anon_struct_fields(&mut g, &ty, true, &name);
      g.writeln("}");
    }
  }
}

fn generate_struct(g: &mut Generator, ty: &Structure, structs: &HashMap<&str, &Structure>) {
  g.writeln("");
  g.write_doc(&ty.documentation);
  g.writeln("#[derive(Serialize, Deserialize, Default, Clone)]");
  g.writeln(format_args!("pub struct {} {{", ty.name));

  generate_struct_fields(g, ty, None, structs);

  g.writeln(format_args!("}}"));
}

fn generate_struct_fields(
  g: &mut Generator,
  ty: &Structure,
  parent: Option<&Structure>,
  _structs: &HashMap<&str, &Structure>,
) {
  for field in ty.properties.iter() {
    if let Some(p) = parent
      && p.properties.iter().any(|p| p.name == field.name)
    {
      continue;
    }

    g.write_doc(&field.documentation);
    if field.name == "type" {
      g.writeln("#[serde(rename = \"type\")]");
      g.write("ty: ");
    } else {
      if to_snake_case(&field.name) != field.name {
        g.writeln(format_args!("#[serde(rename = \"{}\")]", field.name));
      }
      g.write(format_args!("pub {}: ", to_snake_case(&field.name)));
    }

    let mut name_hints = vec![];

    // These two are functionally identical from all reasonable perspectives, except
    // for the inner array type of the one field they share. So we hardcode it.
    if ty.name == "DocumentSymbolClientCapabilities" && field.name == "tagSupport" {
      name_hints.push("DocumentSymbolTagSupportCapabilities".to_string());
    } else if ty.name == "PublishDiagnosticsClientCapabilities" && field.name == "tagSupport" {
      name_hints.push("PublishDiagnosticsTagSupportCapabilities".to_string());
    } else if ty.name == "WorkspaceSymbolClientCapabilities" && field.name == "tagSupport" {
      name_hints.push("WorkspaceSymbolTagSupportCapabilities".to_string());
    } else if ty.name == "InlayHintClientCapabilities" && field.name == "resolveSupport" {
      name_hints.push("InlayHintResolveSupportCapabilities".to_string());
    } else if ty.name == "CodeActionClientCapabilities" && field.name == "resolveSupport" {
      name_hints.push("CodeActionResolveSupportCapabilities".to_string());
    } else if ty.name == "WorkspaceSymbolClientCapabilities" && field.name == "resolveSupport" {
      name_hints.push("WorkspaceSymbolResolveSupportCapabilities".to_string());
    } else if ty.name.ends_with("Capabilities") {
      name_hints.push(format!("{}Capabilities", to_pascal_case(&field.name)));
    }

    name_hints.push(to_pascal_case(&field.name));

    if field.optional {
      if matches!(&field.ty, Type::Reference { name } if *name == ty.name) {
        g.write("Option<Box<");
        write_type(g, &field.ty, name_hints);
        g.write(">>");
      } else if let Type::Or { items } = &field.ty {
        let mut items = items.clone();
        if !items.iter().any(|item| item.is_null()) {
          items.push(Type::Base { name: BaseType::Null });
        }
        write_type(g, &Type::Or { items }, name_hints);
      } else {
        g.write("Option<");
        write_type(g, &field.ty, name_hints);
        g.write(">");
      }
    } else {
      write_type(g, &field.ty, name_hints);
    }

    g.writeln(",");
  }

  for mixin in &ty.mixins {
    g.writeln("#[serde(flatten)]");
    g.writeln(format_args!("pub {}: ", to_snake_case(&variant_name(mixin))));
    write_type(g, &mixin, vec![]);
    g.writeln(",");
  }

  for extends in &ty.extends {
    // NB: Structs can be inlined with this. I think they look better without
    // inlining? Need to experiment a bit.
    /*
    if let Type::Reference { name } = &extends
      && let Some(mixin) = structs.get(name.as_str())
    {
      generate_struct_fields(g, mixin, Some(ty), structs);
    } else {
    */

    g.writeln("#[serde(flatten)]");
    g.writeln(format_args!("pub {}: ", to_snake_case(&variant_name(extends))));
    write_type(g, &extends, vec![]);
    g.writeln(",");
  }
}

fn generate_anon_struct_fields(g: &mut Generator, lit: &Literal, public: bool, name_hint: &str) {
  for prop in &lit.properties {
    g.write_doc(&prop.documentation);
    if prop.name == "type" {
      g.writeln("#[serde(rename = \"type\")]");
      g.write("ty: ");
    } else {
      if to_snake_case(&prop.name) != prop.name {
        g.writeln(format_args!("#[serde(rename = \"{}\")]", prop.name));
      }
      if public {
        g.write("pub ");
      }
      g.write(format_args!("{}: ", to_snake_case(&prop.name)));
    }

    let mut name_hints = vec![];

    if name_hint == "AnonCompletionItemCapabilities" && prop.name == "tagSupport" {
      name_hints.push("AnonCompletionItemTagSupportCapabilities".to_string());
    } else {
      name_hints.push(format!("{}{}", name_hint, to_pascal_case(&prop.name)));
    }

    name_hints.push(to_pascal_case(&prop.name));

    if prop.optional {
      if let Type::Or { items } = &prop.ty {
        let mut items = items.clone();
        if !items.iter().any(|item| item.is_null()) {
          items.push(Type::Base { name: BaseType::Null });
        }
        write_type(g, &Type::Or { items }, name_hints);
      } else {
        g.write("Option<");
        write_type(g, &prop.ty, name_hints);
        g.write(">");
      }
    } else {
      write_type(g, &prop.ty, name_hints);
    }

    g.writeln(",");
  }
}

fn generate_enum(g: &mut Generator, ty: &Enumeration) {
  g.writeln("");
  g.write_doc(&ty.documentation);
  match ty.ty {
    Type::Base { name: BaseType::String } => {
      g.writeln("#[derive(Serialize, Deserialize, Default, Clone)]");
      g.writeln("#[serde(untagged)]");
    }
    Type::Base { name: BaseType::Integer | BaseType::Uinteger } => {
      g.writeln("#[derive(Clone, Copy, Default)]");
    }

    _ => panic!("invalid enum type: {:#?}", ty.ty),
  }

  g.writeln(format_args!("pub enum {} {{", ty.name));
  for (i, variant) in ty.values.iter().enumerate() {
    g.write_doc(&variant.documentation);

    match &variant.value {
      NumberOrString::Number(n) => {
        if i == 0 {
          g.writeln("#[default]");
        }
        if ty.supports_custom_values {
          g.writeln(format_args!("{},", to_pascal_case(&variant.name)));
        } else {
          g.writeln(format_args!("{} = {},", to_pascal_case(&variant.name), n));
        }
      }
      NumberOrString::String(s) => {
        g.writeln(format_args!("#[serde(rename = \"{}\")]", s));
        if i == 0 {
          g.writeln("#[default]");
        }
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

fn generate_type_alias(g: &mut Generator, ty: &TypeAlias) {
  g.writeln("");
  g.write_doc(&ty.documentation);
  match &ty.ty {
    Type::Or { items } => {
      let mut items = items.clone();
      // TextDocumentFilter and NotebookDocumentFilter have utter nonsense schema.
      items.dedup_by(|a, b| match (a, b) {
        (Type::Literal { value: a }, Type::Literal { value: b }) => {
          a.properties.iter().map(|p| &p.name).eq(b.properties.iter().map(|p| &p.name))
        }
        _ => false,
      });

      if items.len() == 1 {
        g.writeln(format_args!("pub type {} = ", ty.name));
        write_type(g, &items[0], vec![ty.name.clone()]);
        g.writeln(";");
      } else {
        g.writeln("#[derive(Serialize, Deserialize, Clone)]");
        g.writeln("#[serde(untagged)]");
        g.writeln(format_args!("pub enum {} {{", ty.name));
        for it in &items {
          g.write(format_args!("{}", variant_name(&it)));

          match it {
            Type::Literal { value } => {
              g.writeln("{");
              generate_anon_struct_fields(g, value, false, &ty.name);
              g.writeln("},");
            }
            _ => {
              g.write("(");
              write_type(g, it, vec![]);
              g.writeln("),");
            }
          }
        }
        g.writeln("}");

        // This struct in particular has only anonymous struct variants, so generating a
        // default impl is annoying.
        if ty.name != "TextDocumentContentChangeEvent" {
          g.writeln("");
          g.writeln(format_args!("impl Default for {} {{", ty.name));
          g.writeln("fn default() -> Self {");
          g.writeln(format_args!("{}::{}(Default::default())", ty.name, variant_name(&items[0])));
          g.writeln("}");
          g.writeln("}");
        }

        g.writeln("");
      }
    }

    _ => {
      g.writeln(format_args!("pub type {} = ", ty.name));
      write_type(g, &ty.ty, vec![ty.name.clone()]);
      g.writeln(";");
    }
  }
}

fn variant_name(ty: &Type) -> String {
  match ty {
    Type::Base { name: BaseType::Null } => "Null".into(),
    Type::Base { name: BaseType::String } => "String".into(),
    Type::Base { name: BaseType::Integer } => "Integer".into(),
    Type::Base { name: BaseType::Uinteger } => "Uinteger".into(),
    Type::Base { name: BaseType::Boolean } => "Boolean".into(),
    Type::Base { name: BaseType::Decimal } => "Decimal".into(),
    Type::Base { name: BaseType::DocumentUri } => "DocumentUri".into(),
    Type::Base { name: BaseType::Uri } => "Uri".into(),

    Type::Reference { name } => name.clone(),
    Type::Array { .. } => "Many".into(),
    Type::Literal { value } => anon_struct_name(value),

    _ => "T".to_string(),
  }
}

fn generate_requests(g: &mut Generator, requests: &[Request]) {
  g.writeln("//! LSP Requests.");
  g.writeln("");
  g.writeln("use crate::*;");
  g.writeln("");

  g.writeln("pub trait Request {");
  g.writeln("const METHOD: &'static str;");
  g.writeln("type Params: Send + serde::Serialize + serde::de::DeserializeOwned;");
  g.writeln("type Result: Send + serde::Serialize + serde::de::DeserializeOwned;");
  g.writeln("}");
  g.writeln("");

  for n in requests {
    g.write_doc(&n.documentation);
    let mut name = rpc_name(&n.method);
    // Avoid conflicts with the `WorkspaceSymbol` type.
    if name == "WorkspaceSymbol" {
      name = "WorkspaceSymbolRequest".into();
    }
    g.writeln(format_args!("pub enum {name} {{}}"));

    g.writeln(format_args!("impl Request for {name} {{"));
    g.writeln(format_args!("const METHOD: &'static str = \"{}\";", n.method));

    g.write(format_args!("type Params = "));
    write_type(
      g,
      &n.params.as_ref().unwrap_or(&Type::Base { name: BaseType::Null }),
      vec![format!("{name}Params")],
    );
    g.writeln(format_args!(";"));

    g.write(format_args!("type Result = "));
    write_type(g, &n.result, vec![format!("{name}Result")]);
    g.writeln(format_args!(";"));

    g.writeln(format_args!("}}"));
  }
}

fn generate_notifications(g: &mut Generator, notifications: &[Notification]) {
  g.writeln("//! LSP Notifications.");
  g.writeln("");
  g.writeln("use crate::*;");
  g.writeln("");

  g.writeln("pub trait Notification {");
  g.writeln("const METHOD: &'static str;");
  g.writeln("type Params: Send + serde::Serialize + serde::de::DeserializeOwned;");
  g.writeln("}");
  g.writeln("");

  for n in notifications {
    g.write_doc(&n.documentation);
    let name = rpc_name(&n.method);
    g.writeln(format_args!("pub enum {name} {{}}"));

    g.writeln(format_args!("impl Notification for {name} {{"));
    g.writeln(format_args!("const METHOD: &'static str = \"{}\";", n.method));

    g.write(format_args!("type Params = "));
    write_type(
      g,
      &n.params.as_ref().unwrap_or(&Type::Base { name: BaseType::Null }),
      vec![format!("{name}Params")],
    );
    g.writeln(format_args!(";"));

    g.writeln(format_args!("}}"));
  }
}

fn write_type(g: &mut Generator, ty: &Type, name_hint: Vec<String>) {
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
      let mut items = items.clone();
      items.dedup_by(|a, b| match (a, b) {
        (Type::Literal { value: a }, Type::Literal { value: b }) => {
          a.properties.iter().map(|p| &p.name).eq(b.properties.iter().map(|p| &p.name))
        }
        _ => false,
      });

      if items.len() == 1 {
        write_type(g, &items[0], name_hint);
      } else if items.iter().any(|item| item.is_null()) {
        g.write("Option<");
        write_type(
          g,
          &Type::Or { items: items.iter().filter(|item| !item.is_null()).cloned().collect() },
          name_hint,
        );
        g.write(">");
      } else if items.len() == 2 {
        g.write("Or2<");
        write_type(g, &items[0], name_hint.clone());
        g.write(", ");
        write_type(g, &items[1], name_hint);
        g.write(">");
      } else if items.len() == 3 {
        g.write("Or3<");
        write_type(g, &items[0], name_hint.clone());
        g.write(", ");
        write_type(g, &items[1], name_hint.clone());
        g.write(", ");
        write_type(g, &items[2], name_hint);
        g.write(">");
      } else if items.len() == 4 {
        g.write("Or4<");
        write_type(g, &items[0], name_hint.clone());
        g.write(", ");
        write_type(g, &items[1], name_hint.clone());
        g.write(", ");
        write_type(g, &items[2], name_hint.clone());
        g.write(", ");
        write_type(g, &items[3], name_hint);
        g.write(">");
      } else {
        panic!("union of length {}", items.len());
      }
    }

    Type::Array { element } => {
      g.write("Vec<");
      write_type(g, element, name_hint);
      g.write(">");
    }

    Type::Tuple { items } => {
      g.write("(");
      for (i, item) in items.iter().enumerate() {
        if i != 0 {
          g.write(", ");
        }
        write_type(g, item, vec![]);
      }
      g.write(")");
    }

    Type::Map { key, value } => {
      g.write("HashMap<");
      write_type(g, key, vec![]);
      g.write(", ");
      write_type(g, value, vec![]);
      g.write(">");
    }

    Type::StringLiteral { value } => {
      g.write(format_args!("String /* \"{}\" */", value));
    }

    Type::Literal { value } => {
      if value.properties.is_empty() {
        write_type(g, &Type::Base { name: BaseType::Null }, name_hint);
      } else {
        let mut name = match name_hint.iter().find_map(|n| {
          let name = if n.starts_with("Anon") { n.clone() } else { format!("Anon{n}") };
          if !g.contains_type(&name) { Some(name) } else { None }
        }) {
          Some(name) => name,
          None if name_hint.is_empty() => anon_struct_name(&value),
          None => format!("Anon{}", name_hint[0].clone()),
        };

        while g.contains_type(&name) {
          name.push('_');
        }

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
      if !snake_case.is_empty() {
        snake_case.push('_');
      }
      snake_case.push(ch.to_ascii_lowercase());
    } else {
      snake_case.push(ch);
    }
  }

  snake_case
}

fn to_pascal_case(method: &str) -> String {
  let mut name = String::new();
  let mut capitalize = true;
  for c in method.chars() {
    match c {
      'a'..='z' if capitalize => {
        name.push(c.to_ascii_uppercase());
        capitalize = false;
      }
      'a'..='z' | 'A'..='Z' | '0'..='9' => {
        name.push(c);
        capitalize = false;
      }
      _ => capitalize = true,
    }
  }

  name.replace("UTF", "Utf")
}

fn rpc_name(method: &str) -> String { to_pascal_case(method) }

fn anon_struct_name(value: &Literal) -> String {
  let name = value
    .properties
    .iter()
    .map(|p| format!("{}_{}", to_snake_case(&p.name), to_snake_case(&variant_name(&p.ty))))
    .collect::<Vec<_>>()
    .join("_");
  format!("Anon{}", to_pascal_case(&name))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_spec() { ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap(); }

  #[test]
  fn notification_name_works() {
    assert_eq!(to_pascal_case("textDocument/publishDiagnostics"), "TextDocumentPublishDiagnostics");
  }
}
