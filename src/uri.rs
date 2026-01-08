use std::{
  fmt,
  path::{Path, PathBuf},
  str::FromStr,
};

use uriparse::Scheme;

// This enum is a little absurdly complex. Let's avoid exposing like 15 types
// for a parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriError(uriparse::URIError);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Uri {
  uri: uriparse::URI<'static>,
}

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
    let s = <&str>::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
  }
}

impl Uri {
  pub fn from_file_path(p: impl AsRef<Path>) -> Self {
    Uri {
      uri: uriparse::URI::builder()
        .with_scheme(Scheme::File)
        .with_authority(Some(uriparse::Authority::try_from("").unwrap()))
        .with_path(
          uriparse::Path::try_from(p.as_ref().as_os_str().to_str().unwrap()).unwrap().into_owned(),
        )
        .build()
        .unwrap(),
    }
  }

  pub fn to_file_path(&self) -> Option<PathBuf> {
    if *self.uri.scheme() == Scheme::File {
      Some(PathBuf::from_str(&self.uri.path().to_string()).unwrap())
    } else {
      None
    }
  }
}

impl FromStr for Uri {
  type Err = UriError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let uri = uriparse::URI::try_from(s).map_err(UriError)?;
    Ok(Uri { uri: uri.into_owned() })
  }
}

impl Default for Uri {
  fn default() -> Self { Uri { uri: uriparse::URI::builder().build().unwrap() } }
}

impl fmt::Debug for Uri {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("Uri").field(&self.to_string()).finish()
  }
}

impl fmt::Display for Uri {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.uri.fmt(f) }
}

impl fmt::Display for UriError {
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
  fn file_path_adds_leading_slash() {
    let uri = Uri::from_file_path("foo/bar.rs");
    assert_eq!(uri, "file:///foo/bar.rs");
    assert_eq!(uri.to_file_path().unwrap(), PathBuf::from("/foo/bar.rs"));
  }

  #[test]
  fn file_path_keeps_trailing() {
    let uri = Uri::from_file_path("/foo/bar");
    assert_eq!(uri, "file:///foo/bar");
    assert_eq!(uri.to_file_path().unwrap(), PathBuf::from("/foo/bar"));

    let uri = Uri::from_file_path("/foo/bar/");
    assert_eq!(uri, "file:///foo/bar/");
    assert_eq!(uri.to_file_path().unwrap(), PathBuf::from("/foo/bar/"));
  }
}
