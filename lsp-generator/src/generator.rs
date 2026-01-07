use std::{
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

  pub fn write_comment(&mut self, comment: &str) {
    self.output.push_str(&format!("// {}\n", comment));
  }
}

impl Drop for Generator {
  fn drop(&mut self) {
    std::fs::write(&self.path, &self.output).unwrap();

    let cmd = Command::new("rustfmt").arg(&self.path).output().unwrap();
    assert!(cmd.status.success());
  }
}
