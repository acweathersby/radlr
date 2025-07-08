//! Radlr Core
//!
//! A Parser Compiler Framework
//!
//! This library contains all functions and types needed to create
//! and update Radlr grammars, generate IR parsers, and
//! optimize to create the best parsers possible.
//!
//! Visit [Radlr Docs](https://localhost:1313) to learn more about Radlr.

#![allow(non_snake_case)]
#![feature(box_patterns)]

mod builder;
mod compile;
mod grammar;
pub mod parser;
mod types;
mod utils;
mod writer;

pub use types::{
  o_to_r,
  proxy,
  worker_pool,
  ASTToken,
  CachedString,
  DBEntryPoint,
  DBNonTermKey,
  DBRule,
  DBRuleKey,
  DBTermKey,
  DBTokenData,
  ErrorClass,
  GrammarDatabase,
  GrammarHeader,
  GrammarId,
  GrammarIdentities,
  GrammarSoup,
  IString,
  IStringStore,
  Item,
  NonTermId,
  NonTerminal,
  ParseState,
  ParseStatesMap,
  ParseStatesVec,
  ParserClassification,
  ParserConfig,
  ParserMetrics,
  RadlrError,
  RadlrErrorSeverity,
  RadlrResult,
  ReductionType,
  Rule,
  SubNonTermType,
  SubNonTerminal,
  SymbolId,
  SymbolRef,
};

pub use builder::*;
pub use utils::{create_u64_hash as hash_id_value_u64, hash_group_btreemap};
pub use writer::code_writer::CodeWriter;


pub mod test;
