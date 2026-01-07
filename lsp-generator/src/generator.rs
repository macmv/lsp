use std::{
  fmt::{Display, Write},
  path::{Path, PathBuf},
  process::Command,
};

pub struct Generator {
  output: String,
  path:   PathBuf,
}

impl Generator {
  pub fn new(path: impl AsRef<Path>) -> Self {
    Generator { output: String::new(), path: path.as_ref().to_path_buf() }
  }

  pub fn writeln(&mut self, text: impl Display) { writeln!(self.output, "{text}").unwrap(); }
  pub fn write(&mut self, text: impl Display) { write!(self.output, "{text}").unwrap(); }
  pub fn write_doc(&mut self, doc: &str) {
    for line in doc.lines() {
      writeln!(self.output, "/// {line}").unwrap();
    }
  }
}

impl Drop for Generator {
  fn drop(&mut self) {
    std::fs::write(&self.path, &self.output).unwrap();

    let cmd = Command::new("rustfmt").arg(&self.path).output().unwrap();
    if !cmd.status.success() {
      println!("rustfmt failed.\n");
      println!("{}", String::from_utf8_lossy(&cmd.stderr));

      panic!();
    }
  }
}
