use std::{
  collections::HashSet,
  fmt::{Display, Write},
  path::{Path, PathBuf},
  process::Command,
};

use crate::spec::Literal;

pub struct Generator {
  output: String,
  path:   PathBuf,

  type_names: HashSet<String>,
  types:      Vec<(String, Literal)>,
}

impl Generator {
  pub fn new(path: impl AsRef<Path>) -> Self {
    Generator {
      output:     String::new(),
      path:       path.as_ref().to_path_buf(),
      type_names: HashSet::new(),
      types:      vec![],
    }
  }

  pub fn writeln(&mut self, text: impl Display) { writeln!(self.output, "{text}").unwrap(); }
  pub fn write(&mut self, text: impl Display) { write!(self.output, "{text}").unwrap(); }
  pub fn write_doc(&mut self, doc: &str) {
    for line in doc.lines() {
      writeln!(self.output, "/// {line}").unwrap();
    }
  }

  pub fn has_types(&self) -> bool { !self.types.is_empty() }
  pub fn add_type(&mut self, name: String, ty: Literal) {
    if self.type_names.insert(name.clone()) {
      self.types.push((name, ty));
    }
  }
  pub fn drain_types(&mut self) -> Vec<(String, Literal)> { std::mem::take(&mut self.types) }
}

impl Drop for Generator {
  fn drop(&mut self) {
    if std::thread::panicking() {
      return;
    }

    if self.has_types() {
      panic!("leftover types: {:#?}", self.types);
    }

    std::fs::write(&self.path, &self.output).unwrap();

    let cmd = Command::new("rustfmt").arg("--edition=2024").arg(&self.path).output().unwrap();
    if !cmd.status.success() {
      println!("rustfmt failed.\n");
      println!("{}", String::from_utf8_lossy(&cmd.stderr));

      panic!();
    }
  }
}
