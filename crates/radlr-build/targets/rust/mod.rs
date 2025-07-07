use super::common::{build_ast_source, build_parser_source, build_parser_states};
use crate::BuildConfig;
use radlr_core::*;
use std::{fs::*, io::Write};

const SCRIPT: &'static str = include_str!("rust_ast_script.atat");
const BC_SCRIPT: &'static str = include_str!("rust_bytecode_script.atat");

#[derive(Debug, Clone, Copy, Default)]
pub struct RustConfig {
  /// Adds a mod file to source directory
  pub add_mod:        bool,
  /// When used with `add_mod`, flattens the namespace of the module
  pub flat_namespace: bool,
}

pub fn build(db: &RadlrGrammarDatabase, build_config: BuildConfig, parser_config: ParserConfig) -> RadlrResult<()> {
  let (out_dir, lib_dir) = (build_config.source_out, build_config.lib_out);

  let (binary_path, parser_path, ast_path) = if let Some(name_prefix) = build_config.name_prefix {
    (
      lib_dir.join(format!("{name_prefix}_parser.bin")),
      out_dir.join(format!("{name_prefix}_parser.rs")),
      out_dir.join(format!("{name_prefix}_ast.rs")),
    )
  } else {
    (lib_dir.join("parser.bin"), out_dir.join("parser.rs"), out_dir.join("ast.rs"))
  };

  if build_config.build_parser {
    let parser = build_parser_states(db, parser_config)?;

    let bytecode = radlr_bytecode::compile_bytecode(&parser, false)?;

    {
      let mut parser_binary = OpenOptions::new().append(false).truncate(true).write(true).create(true).open(&binary_path)?;
      bytecode.write_binary(&mut parser_binary)?;
      parser_binary.flush()?;
    }

    build_parser_source(db, BC_SCRIPT, bytecode, binary_path, parser_path)?;
  }

  if build_config.build_ast {
    build_ast_source(db, SCRIPT, ast_path, build_config.ast_struct_name, &[("RUST_NODE_WRAPPER", "std::sync::Arc")])?;
  }

  if build_config.rust.add_mod {
    let mod_path = out_dir.join("mod.rs");

    let mut parser = OpenOptions::new().append(false).truncate(true).write(true).create(true).open(&mod_path)?;

    if build_config.rust.flat_namespace {
      parser.write_all(b"mod parser;\n")?;
      parser.write_all(b"pub use parser::*;\n")?;
      if build_config.build_ast {
        parser.write_all(b"mod ast;\n")?;
        parser.write_all(b"pub use ast::*;\n")?;
      }
    } else {
      parser.write_all(b"pub mod parser;\n")?;
      if build_config.build_ast {
        parser.write_all(b"pub mod ast;\n")?;
      }
    }
  }

  Ok(())
}
