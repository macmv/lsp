use std::{
  borrow::Cow,
  collections::HashMap,
  fmt::{Display, Write},
  path::{Path, PathBuf},
  process::Command,
  sync::LazyLock,
};

use regex::Regex;

use crate::{names::Names, spec::Literal};

pub struct Generator<'a> {
  output: String,
  path:   PathBuf,

  type_map: HashMap<String, Literal>,
  types:    Vec<(String, Literal)>,

  names: &'a Names,
}

const LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{@link\s+(\S+)\}").unwrap());
const LINK_NAMED_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\{@link\s+(\S+)\s([^}]+)\}").unwrap());
const SAMPLE_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\n@sample ([^`]+)`([^`]+)`").unwrap());

impl<'a> Generator<'a> {
  pub fn new(path: impl AsRef<Path>, names: &'a Names) -> Self {
    Generator {
      output: String::new(),
      path: path.as_ref().to_path_buf(),
      type_map: HashMap::new(),
      types: vec![],
      names,
    }
  }

  pub fn writeln(&mut self, text: impl Display) { writeln!(self.output, "{text}").unwrap(); }
  pub fn write(&mut self, text: impl Display) { write!(self.output, "{text}").unwrap(); }
  pub fn write_doc(&mut self, doc: &str) {
    let mut doc = Cow::Borrowed(doc);

    let deprecated = if let Some(start) = doc.find("@deprecated") {
      let len = doc[start..].find('\n').unwrap_or(doc[start..].len());
      let reason = doc[start + "@deprecated ".len()..start + len].to_string();
      let mut d = doc.to_string();
      d.replace_range(start..start + len, "");
      doc = d.into();

      Some(reason)
    } else {
      None
    };

    let doc = LINK_REGEX.replace_all(&doc, |caps: &regex::Captures| {
      let ty = caps.get(1).unwrap().as_str();
      format!("[`{}`]({})", ty, self.names.resolve(ty))
    });
    let doc = LINK_NAMED_REGEX.replace_all(&doc, |caps: &regex::Captures| {
      let ty = self.names.resolve(caps.get(1).unwrap().as_str());
      let mut text = Cow::Borrowed(caps.get(2).unwrap().as_str());

      if text.ends_with("[]") {
        text = format!("Vec<{}>", text[0..text.len() - 2].trim()).into();
      }

      format!("[`{text}`]({ty})")
    });
    let doc = SAMPLE_REGEX.replace_all(&doc, |caps: &regex::Captures| {
      format!(
        "\n# Sample\n\n{}\n\n```ts\n{}\n```\n",
        caps.get(1).unwrap().as_str(),
        caps.get(2).unwrap().as_str()
      )
    });

    for line in doc.lines() {
      writeln!(self.output, "/// {line}").unwrap();
    }

    if let Some(deprecated) = deprecated {
      self.writeln(format_args!("#[deprecated = \"{deprecated}\"]"));
    }
  }

  pub fn has_types(&self) -> bool { !self.types.is_empty() }
  pub fn contains_type(&self, name: &str) -> bool { self.type_map.contains_key(name) }
  pub fn add_type(&mut self, name: String, ty: Literal) {
    if let Some(prev) = self.type_map.insert(name.clone(), ty.clone()) {
      if prev != ty {
        panic!("type mismatch for {name}\n{prev:?}\n{ty:?}");
      }
    } else {
      self.types.push((name, ty));
    }
  }
  pub fn drain_types(&mut self) -> Vec<(String, Literal)> { std::mem::take(&mut self.types) }
}

impl Drop for Generator<'_> {
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
