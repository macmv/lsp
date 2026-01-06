use crate::spec::Spec;

const URL: &str = "https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/metaModel/metaModel.json";

mod spec;

pub fn generate() {
  let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

  dbg!(&spec);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_spec() {
    let spec = ureq::get(URL).call().unwrap().into_body().read_json::<Spec>().unwrap();

    dbg!(&spec);
    panic!();
  }
}
