use crate::{generator::Generator, spec::*};

const URL: &str = "https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json";

mod generator;
mod spec;

pub fn generate() {
  let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

  generate_notifications(&mut Generator::new("src/notification.rs"), &spec.notifications);

  let mut g = Generator::new("src/lib.rs");
  g.write("pub mod notification;");
}

fn generate_notifications(g: &mut Generator, notifications: &[Notification]) {
  g.write("//! LSP Notifications.");
  g.write("");

  for n in notifications {
    g.write_doc(&n.documentation);
    g.write(&format!("pub const {}: &str = \"{}\";", notification_name(&n.method), n.method));
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
