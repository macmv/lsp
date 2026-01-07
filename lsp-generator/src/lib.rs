use crate::{generator::Generator, spec::*};

const URL: &str = "https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json";

mod generator;
mod spec;

pub fn generate() {
  let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

  generate_requests(&mut Generator::new("src/request.rs"), &spec.requests);
  generate_notifications(&mut Generator::new("src/notification.rs"), &spec.notifications);

  let mut g = Generator::new("src/lib.rs");
  g.writeln("use serde::{Serialize, Deserialize};");
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

    g.write(format_args!("type Params ="));
    write_type(g, &n.params.as_ref().unwrap_or(&Type::Base { name: "null".into() }));
    g.writeln(format_args!(";"));

    g.write(format_args!("type Result ="));
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

    g.write(format_args!("type Params ="));
    write_type(g, &n.params.as_ref().unwrap_or(&Type::Base { name: "null".into() }));
    g.writeln(format_args!(";"));

    g.writeln(format_args!("}}"));
  }
}

fn write_type(g: &mut Generator, ty: &Type) {
  match ty {
    Type::Base { name } if name == "null" => g.write("()"),
    Type::Base { name } => g.write(name),
    Type::Reference { name } if name == "LSPAny" => g.write("serde_json::Value"),
    Type::Reference { name } => g.write(name),

    Type::Or { items } => {
      if items.len() == 1 {
        write_type(g, &items[0]);
      } else if items.iter().any(|item| item == &Type::Base { name: "null".into() }) {
        g.write("Option<");
        write_type(
          g,
          &Type::Or {
            items: items
              .iter()
              .filter(|item| *item != &Type::Base { name: "null".into() })
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
        panic!("unhandled or {items:?}");
      }
    }

    Type::Array { element } => {
      g.write("Vec<");
      write_type(g, element);
      g.write(">");
    }

    _ => {
      todo!("write type {ty:?}");
    }
  }
}

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
