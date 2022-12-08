//! # Sherpa
//! A parser compiler
//!
//! ## Examples:
//!
//! ### compile a grammar from a string
#![crate_type = "rlib"]
#![const_eval_limit = "0"]
#![feature(const_eval_limit)]
#![feature(box_patterns)]
#![feature(drain_filter)]
#![feature(btree_drain_filter)]
#![feature(try_trait_v2)]
#![feature(int_roundings)]
#![feature(map_try_insert)]
#![allow(bad_style)]
#![allow(non_snake_case)]

mod ascript;
mod build;
mod bytecode;
pub mod debug;
mod deprecated_runtime;
mod grammar;
mod intermediate;
mod journal;
mod llvm;
mod types;
mod util;
mod writer;

#[cfg(test)]
mod test;

pub use journal::{Config, Journal, Report, ReportType};

pub use types::{SherpaError, SherpaResult};

pub mod compile {
  pub use crate::{
    bytecode::{compile_bytecode, BytecodeOutput},
    grammar::parse::{compile_ascript_ast, compile_grammar_ast, compile_ir_ast},
    intermediate::{compile::*, optimize::*},
    types::{
      GrammarId,
      GrammarRef,
      GrammarStore,
      Production,
      ProductionId,
      Rule,
      ScannerStateId,
      Symbol,
      SymbolID,
    },
  };
}
pub mod errors {
  pub use crate::{
    intermediate::errors::*,
    types::{SherpaError, SherpaError::*, SherpaErrorSeverity},
  };
}

/// Create a build pipeline
pub mod pipeline {

  pub use crate::build::pipeline::{compile_bytecode_parser, BuildPipeline, SourceType};

  pub mod tasks {
    pub use crate::build::{
      ascript::build_ascript_types_and_functions,
      bytecode::build_bytecode_parser,
      disassembly::build_bytecode_disassembly,
      llvm::{build_llvm_parser, build_llvm_parser_interface},
    };
  }
}
