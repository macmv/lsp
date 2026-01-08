use std::{
  borrow::Cow,
  fmt,
  path::{Path, PathBuf},
};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, percent_encode};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Uri(pub String);

const FILE_PATH_ENCODE_SET: &AsciiSet =
  &CONTROLS.add(b' ').add(b'#').add(b'?').add(b'%').add(b'[').add(b']').add(b'/');

impl serde::Serialize for Uri {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> serde::Deserialize<'de> for Uri {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    Ok(Uri(<String>::deserialize(deserializer)?))
  }
}

impl Uri {
  pub fn from_file_path(p: impl AsRef<Path>) -> Self {
    let mut p = p.as_ref().as_os_str().to_str().expect("cannot encode non-utf8 path");

    while let Some(stripped) = p.strip_prefix('/') {
      p = stripped;
    }

    let mut buf = String::from("file:///");
    for (i, segment) in p.split_inclusive('/').enumerate() {
      if segment.is_empty() && i != 0 {
        continue;
      }

      let trailing_slash = segment.ends_with('/');
      buf.push_str(
        &percent_encode(
          segment[..segment.len() - trailing_slash as usize].as_bytes(),
          FILE_PATH_ENCODE_SET,
        )
        .to_string(),
      );

      if trailing_slash {
        buf.push('/');
      }
    }

    Uri(buf)
  }

  pub fn to_file_path(&self) -> Option<PathBuf> {
    if let Some(mut path) = self.0.strip_prefix("file://") {
      let mut buf = PathBuf::new();

      // `file://foo` is not valid.
      path = path.strip_prefix('/')?;
      match path.chars().next() {
        Some('a'..='z' | 'A'..='Z')
          if path.chars().nth(1) == Some(':') && path.chars().nth(2) == Some('/') =>
        {
          // This is a prefix.
          buf.push(&path[..3]);
          path = &path[3..];
        }
        _ => buf.push("/"),
      }

      for segment in path.split('/') {
        buf.push(&*String::from_utf8_lossy(&Cow::from(percent_decode_str(segment))));
      }

      Some(buf)
    } else {
      None
    }
  }
}

impl fmt::Debug for Uri {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Uri").field(&self.to_string()).finish()
  }
}

impl fmt::Display for Uri {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl PartialEq<str> for Uri {
  fn eq(&self, other: &str) -> bool { self.to_string() == other }
}

impl PartialEq<&str> for Uri {
  fn eq(&self, other: &&str) -> bool { *self == **other }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn file_path_conversion() {
    let uri = Uri::from_file_path("/foo/bar.rs");
    assert_eq!(uri, "file:///foo/bar.rs");
    assert_eq!(uri.to_file_path().unwrap(), PathBuf::from("/foo/bar.rs"));
  }

  #[test]
  fn from_path_adds_leading_slash() {
    assert_eq!(Uri::from_file_path("foo/bar.rs"), "file:///foo/bar.rs");
  }

  #[test]
  fn from_path_keeps_trailing() {
    assert_eq!(Uri::from_file_path("/foo/bar"), "file:///foo/bar");
    assert_eq!(Uri::from_file_path("/foo/bar/"), "file:///foo/bar/");
  }

  #[test]
  fn to_path_removes_trailing() {
    assert_eq!(Uri("file:///foo/bar/".into()).to_file_path().unwrap(), Path::new("/foo/bar"));
  }

  #[test]
  fn from_path_encodes() {
    assert_eq!(Uri::from_file_path("/foo bar"), "file:///foo%20bar");
    assert_eq!(Uri::from_file_path("/foo#bar"), "file:///foo%23bar");
    assert_eq!(Uri::from_file_path("/foo?bar"), "file:///foo%3Fbar");
    assert_eq!(Uri::from_file_path("/foo%bar"), "file:///foo%25bar");
    assert_eq!(Uri::from_file_path("/foo[bar"), "file:///foo%5Bbar");
    assert_eq!(Uri::from_file_path("/foo]bar"), "file:///foo%5Dbar");
  }

  #[test]
  fn to_path_decodes() {
    assert_eq!(Uri("file:///foo%20bar".into()).to_file_path().unwrap(), Path::new("/foo bar"));
    assert_eq!(Uri("file:///foo%23bar".into()).to_file_path().unwrap(), Path::new("/foo#bar"));
    assert_eq!(Uri("file:///foo%3Fbar".into()).to_file_path().unwrap(), Path::new("/foo?bar"));
    assert_eq!(Uri("file:///foo%25bar".into()).to_file_path().unwrap(), Path::new("/foo%bar"));
    assert_eq!(Uri("file:///foo%5Bbar".into()).to_file_path().unwrap(), Path::new("/foo[bar"));
    assert_eq!(Uri("file:///foo%5Dbar".into()).to_file_path().unwrap(), Path::new("/foo]bar"));
  }

  #[test]
  fn from_path_handles_prefix() {
    // This is technically ambiguous. Because we want `Uri` to be portable, we
    // assume any path whose first component matches `[a-z]:` is a prefix.
    assert_eq!(Uri::from_file_path("C:/foo/bar.txt"), Uri("file:///C:/foo/bar.txt".into()),);
  }

  #[test]
  fn to_path_handles_prefix() {
    // This is technically ambiguous. Because we want `Uri` to be portable, we
    // assume any path whose first component matches `[a-z]:` is a prefix.
    assert_eq!(
      Uri("file:///C:/foo/bar.txt".into()).to_file_path().unwrap(),
      Path::new("C:/foo/bar.txt")
    );
  }
}
