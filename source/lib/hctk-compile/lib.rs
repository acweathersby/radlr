#![feature(const_format_args)]
#![feature(const_fmt_arguments_new)]
#![feature(box_patterns)]
#![allow(non_snake_case)]
#![feature(drain_filter)]
#![feature(btree_drain_filter)]
#![feature(hash_drain_filter)]

mod ascript;
mod builder;
mod llvm;
mod options;
mod source_types;

use std::fs::create_dir_all;
use std::path::PathBuf;

pub use builder::bytecode;
use hctk_core::types::HCErrorContainer;

pub use crate::builder::pipeline::BuildPipeline;
pub use source_types::*;

pub mod tasks {
  pub use crate::ascript::build_ast;
  pub use crate::builder::bytecode::build_bytecode_parser;
  pub use crate::builder::disassembly::build_bytecode_disassembly;
  pub use crate::builder::llvm::build_llvm_parser;
  pub use crate::builder::llvm::build_llvm_parser_interface;
}

/// Convenience function for building a bytecode based parser. Use this in
/// build scripts to output a parser source file to `{OUT_DIR}/hc_parser/{grammar_name}.rs`.
pub fn compile_bytecode_parser(grammar_source_path: &PathBuf, include_ascript: bool) -> bool {
  let mut out_dir = std::env::var("OUT_DIR").map(|d| PathBuf::from(&d)).unwrap();

  out_dir.push("./hc_parser/");

  create_dir_all(&out_dir).unwrap();

  let pipeline = BuildPipeline::from_source(&grammar_source_path, 0)
    .set_source_output_dir(&out_dir)
    .set_build_output_dir(&out_dir)
    .set_source_file_name("%.rs")
    .add_task(tasks::build_bytecode_parser(SourceType::Rust, include_ascript));

  if include_ascript { pipeline.add_task(tasks::build_ast(SourceType::Rust)) } else { pipeline }
    .add_task(tasks::build_bytecode_disassembly())
    .run(|errors| {
      for error in &errors {
        eprintln!("{}", error);
      }
    })
    .2
}

#[cfg(test)]
mod library_smoke_tests {

  use std::path::PathBuf;

  use hctk_core::types::GrammarStore;

  use crate::ascript::compile::compile_ascript_store;
  use crate::ascript::types::AScriptStore;
  use crate::builder::pipeline::BuildPipeline;
  use crate::tasks::build_ast;

  #[test]
  fn test_compile_pipeline() {
    BuildPipeline::from_string(
      "
    <> A >\\1 f:ast { { t_Banana, c_Mobius, value:u32($1), string:str($1), useful:true } } 
    | \\a \\b A f:ast { { t_Banana, value: u32($1), dd:u32($3), tok, useful:false } }
    ",
      &PathBuf::from("/tmp"),
    )
    .set_source_output_dir(&std::env::temp_dir())
    .add_task(build_ast(crate::SourceType::Rust))
    .run(|e| {});
  }

  #[test]
  fn test_output_rust_on_trivial_grammar() {
    let g = GrammarStore::from_str(
      "
        <> A >\\1 f:ast { { t_Banana, c_Mobius, value:u32($1), string:str($1), useful:true } } 
        | \\a \\b A f:ast { { t_Banana, value: u32($1), dd:u32($3), tok, useful:false } }
        ",
    )
    .unwrap();

    let mut ascript = AScriptStore::new(g).unwrap();

    assert_eq!(ascript.structs.len(), 1);
  }
}
