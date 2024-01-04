#![feature(box_patterns)]

#[cfg(test)]
mod test;

mod builder;

use crate::builder::write_rust_llvm_parser_file;
use builder::{
  add_ascript_functions_for_rust,
  create_rust_writer_utils,
  write_rust_ast,
  write_rust_ast2,
  write_rust_bytecode_parser_file,
};
use radlr_ascript::{output_base::AscriptWriter, types::AScriptStore};
use radlr_core::{CodeWriter, Journal, ParserStore, RadlrDatabase, RadlrResult};
use radlr_rust_runtime::types::BytecodeParserDB;

pub fn build_rust(mut j: Journal, db: &RadlrDatabase) -> RadlrResult<String> {
  j.set_active_report("Rust AST Compile", radlr_core::ReportType::Any);

  let store = AScriptStore::new(j.transfer(), db.get_internal())?;
  let u = create_rust_writer_utils(&store, db.get_internal());
  let w = AscriptWriter::new(&u, CodeWriter::new(vec![]));

  let writer = write_rust_ast2(w)?;

  String::from_utf8(writer.into_writer().into_output()).map_err(|e| e.into())
}

pub fn compile_rust_bytecode_parser<T: ParserStore>(store: &T, pkg: &BytecodeParserDB) -> RadlrResult<String> {
  let db = store.get_db();
  let mut j = store.get_journal().transfer();

  let store = AScriptStore::new(j.transfer(), &db);

  let store: AScriptStore = store?;

  j.flush_reports();

  assert!(!j.have_errors_of_type(radlr_core::RadlrErrorSeverity::Critical));

  let state_lookups = pkg.state_name_to_address.iter().map(|(name, offset)| (name.clone(), *offset as u32)).collect();

  let u = create_rust_writer_utils(&store, &db);

  let mut writer = AscriptWriter::new(&u, CodeWriter::new(vec![]));

  writer.stmt(
    r###"/// ### `radlr` Rust Parser
///
/// - **GENERATOR**: radlr 1.0.1-beta2
/// - **SOURCE**: /home/work/projects/lib_radlr/grammars/v2_0_0/grammar.sg
///
/// #### WARNING:
///
/// This is a generated file. Any changes to this file may be **overwritten
/// without notice**.
///
/// #### License:
/// Copyright (c) 2023 Anthony Weathersby
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to
/// deal in the Software without restriction, including without limitation the
/// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
/// sell copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in
/// all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
/// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
/// IN THE SOFTWARE.

// 
use std::hash::Hash;
use radlr_rust_runtime::{
  llvm_parser::*,
  types::{ast::*, Token, TokenRange}, deprecate::*,
};

"###
      .to_string(),
  )?;

  add_ascript_functions_for_rust(&mut writer, db)?;

  let writer = write_rust_ast(writer)?;

  let writer = write_rust_bytecode_parser_file(writer, &state_lookups, &pkg.bytecode)?;

  RadlrResult::Ok(writer.into_writer().to_string())
}

pub fn compile_rust_llvm_parser<T: ParserStore>(store: &T, grammar_name: &str, parser_name: &str) -> RadlrResult<String> {
  let db = store.get_db();
  let mut j = store.get_journal().transfer();

  let j2 = j.transfer();

  let store = AScriptStore::new(j2, &db);

  let store: AScriptStore = store?;

  j.flush_reports();

  assert!(!j.have_errors_of_type(radlr_core::RadlrErrorSeverity::Critical));

  let u = create_rust_writer_utils(&store, &db);

  let mut writer = AscriptWriter::new(&u, CodeWriter::new(vec![]));

  writer.stmt(
    r###"/// ### `radlr` Rust Parser
///
/// - **GENERATOR**: radlr 1.0.1-beta2
/// - **SOURCE**: /home/work/projects/lib_radlr/grammars/v2_0_0/grammar.sg
///
/// #### WARNING:
///
/// This is a generated file. Any changes to this file may be **overwritten
/// without notice**.
///
/// #### License:
/// Copyright (c) 2023 Anthony Weathersby
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to
/// deal in the Software without restriction, including without limitation the
/// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
/// sell copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in
/// all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
/// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
/// IN THE SOFTWARE.

// 
use std::hash::Hash;
use radlr_rust_runtime::{
  llvm_parser::*,
  types::{ast::*, *},
};
"###
      .to_string(),
  )?;

  add_ascript_functions_for_rust(&mut writer, db)?;

  let writer = write_rust_ast(writer)?;

  let writer = write_rust_llvm_parser_file(writer, grammar_name, parser_name)?;

  RadlrResult::Ok(writer.into_writer().to_string())
}
