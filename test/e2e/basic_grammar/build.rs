use std::{path::PathBuf, str::FromStr};

use sherpa_compile::compile_bytecode_files;

fn main() {
  if let Ok(cwd) = std::env::var("CARGO_MANIFEST_DIR").map(|d| PathBuf::from_str(&d).unwrap()) {
    if let Ok(input) = cwd.join("./grammar.hcg").canonicalize() {
      println!("{}", input.to_str().unwrap());
      println!("cargo:rerun-if-changed={}", input.to_str().unwrap());
      compile_bytecode_files(&input, &cwd, true);
    }
  }
}
