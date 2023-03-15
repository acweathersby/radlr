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
#![allow(bad_style)]
#![allow(non_snake_case)]
#![warn(missing_docs)]

mod ascript;
#[cfg(not(feature = "wasm-target"))]
mod build;
mod bytecode;
mod grammar;
mod journal;
mod llvm;
mod parser;
mod tasks;
mod types;
mod util;
mod writer;

#[cfg(all(test, not(feature = "wasm-target")))]
mod test;

/// Methods for debugging a grammar and it's artifacts.
pub mod debug;

pub use journal::{Config, Journal, Report, ReportType};

pub use types::{SherpaError, SherpaResult};

/// Methods compiling a parser from a grammar.
pub mod compile {
  #[cfg(feature = "wasm-target")]
  pub use crate::grammar::compile::parser::sherpa_bc::{bytecode, meta};
  pub use crate::{
    bytecode::{compile_bytecode, BytecodeOutput},
    parser::{compile_parse_states, optimize_parse_states},
    types::{
      GrammarId,
      GrammarRef,
      GrammarStore,
      ParseState,
      Production,
      ProductionId,
      Rule,
      ScannerStateId,
      Symbol,
      SymbolID,
    },
  };
}

/// Error objects
pub mod errors {
  pub use crate::types::{SherpaError, SherpaError::*, SherpaErrorSeverity};
}

#[cfg(not(feature = "wasm-target"))]
/// Create a build pipeline
pub mod pipeline {

  #[cfg(feature = "llvm")]
  pub use crate::build::pipeline::{
    compile_bytecode_parser,
    compile_llvm_parser,
  };
  pub use crate::build::pipeline::{BuildPipeline, SourceType};
  /// Tasks that can be added to a build pipeline
  pub mod tasks {
    #[cfg(feature = "llvm")]
    pub use crate::build::llvm::{
      build_llvm_parser,
      build_llvm_parser_interface,
    };
    pub use crate::build::{
      ascript::build_ascript_types_and_functions,
      bytecode::build_bytecode_parser,
      disassembly::build_bytecode_disassembly,
      rust_preamble::build_rust_preamble,
    };
  }
}
