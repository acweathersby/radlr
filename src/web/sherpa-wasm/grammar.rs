use sherpa_bytecode::compile_bytecode;
use sherpa_core::{
  compile_grammar_from_str,
  remove_grammar_mut,
  GrammarIdentities,
  GrammarSoup,
  Journal,
  ParserDatabase,
  SherpaResult, build_compile_db, compile_parse_states, optimize, SherpaError, ParseState, ParseStatesMap, ParseStatesVec, proxy::Map, IString,
};
use sherpa_rust_build::build_rust;
use sherpa_rust_runtime::bytecode::{disassemble_bytecode, disassemble_parse_block};
use std::{path::PathBuf, sync::LockResult};
use wasm_bindgen::prelude::*;

use crate::error::PositionedErrors;

/// A Grammar Identity
#[wasm_bindgen]
pub struct JSGrammarIdentities(pub(crate) Box<GrammarIdentities>);

/// A Parser database derived from grammar defined in a JSSoup
#[wasm_bindgen]
pub struct JSParserDB(pub(crate) Box<ParserDatabase>);

/// Parser states generated from the compilation of parser db
#[wasm_bindgen]
pub struct JSParseStates(pub(crate) Box<ParseStatesVec>);

/// An arbitrary collection of grammars
#[wasm_bindgen]
pub struct JSSoup(pub(crate) Box<GrammarSoup>);

/// Bytecode produced from parse states
#[wasm_bindgen]
pub struct JSBytecode(pub(crate) Box<(Vec<u8>,  Map<IString, usize>)>);

#[wasm_bindgen]
impl JSSoup {
  /// Adds or replaces grammar in the soup, or throws an error
  /// if the grammar is invalid. Returns the grammar
  /// id if successful.
  pub fn add_grammar(
    &mut self,
    grammar: String,
    path: String,
  ) -> Result<JSGrammarIdentities, PositionedErrors> {
    let mut j = Journal::new(Default::default());

    j.set_active_report("grammar parse", sherpa_core::ReportType::Any);    

    if remove_grammar_mut((&PathBuf::from(&path)).into(), &mut self.0).is_faulty() {
      return Result::Err(convert_journal_errors(&mut j));
    }

    match compile_grammar_from_str(&mut j, grammar.as_str(), path.into(), &self.0) {
      SherpaResult::Ok(g_id) => Ok(JSGrammarIdentities(Box::new(g_id))),
      _ => {
        return Result::Err(convert_journal_errors(&mut j));
      },
    }
  }

  /// Adds a production targeting a specific grammar
  pub fn add_production(&mut self, grammar_name: String) -> Result<(), JsError> {
    Ok(())
  }
}

/// Creates an empty grammar soup object.
/// Use soup modifiers to add grammars and productions
///
/// Pass soup to parser compiler functions to create parsers, generate bytecode,
/// and construct ASCript AST and CST structures.
#[wasm_bindgen]
pub fn create_soup() -> Result<JSSoup, JsError> {
  Ok(JSSoup(Box::new(Default::default())))
}


/// Creates a parser db from a soup and a root grammar, or returns semantic errors.
#[wasm_bindgen]
pub fn create_parse_db(grammar_id: String, soup: &JSSoup) -> Result<JSParserDB, PositionedErrors> {
  let mut j = Journal::new(None);


  let db = if let LockResult::Ok(headers) = soup.0.grammar_headers.read() {

    match headers.get(&(&PathBuf::from(&grammar_id)).into()) {
      Some(g_id) => {
        j.set_active_report("ParserDB Compile", sherpa_core::ReportType::Any);

        let gs = soup.0.as_ref();
      
        let id = g_id.identity;
      
        let SherpaResult::Ok(db) = build_compile_db(j.transfer(), id, gs) else {
          
          j.flush_reports();

          let mut errors = PositionedErrors::default();

          if let Some(reports) = j.get_faulty_reports() {
            for report in &reports {
              errors.extend_from_refs(&report.errors());
            }
          }


          return Result::Err(errors);
  
        };

        db
      },
    _ =>{return Err(Default::default())}
    }
  } else {
    return Err(Default::default())
  };
    


  return Ok(JSParserDB(Box::new(db)))
}


/// Temporary simple AST output implementation.
#[wasm_bindgen]
pub fn create_rust_ast_output(js_db: &JSParserDB) -> Result<String, PositionedErrors> {
  let mut j = Journal::new(None);
  j.set_active_report("ast compile", sherpa_core::ReportType::Any);

  let db = &js_db.0;

  let SherpaResult::Ok(output) = build_rust(j.transfer(), db) else {
      return Result::Err(convert_journal_errors(&mut j));
  };

  Ok(output)
}


/// Temporary simple AST output implementation.
#[wasm_bindgen]
pub fn create_parser_states(js_db: &JSParserDB, optimize_states: bool) -> Result<JSParseStates, PositionedErrors> {
  let mut j = Journal::new(None);

  j.set_active_report("state construction", sherpa_core::ReportType::Any);

  let db = &js_db.0;

  let SherpaResult::Ok(states) =compile_parse_states(j.transfer(), &db)else {
      return Result::Err(convert_journal_errors(&mut j));
  };

  let states = if optimize_states {
    let SherpaResult::Ok(states) =optimize( &db, states)else {
      return Result::Err(convert_journal_errors(&mut j));
    };
    states
  }else {
    states.into_iter().collect()
  };

  Ok(JSParseStates(Box::new(states)))
}


fn convert_journal_errors(j:&mut Journal) -> PositionedErrors {
  let mut errors = PositionedErrors::default();
  j.flush_reports();
  if let Some(reports) = j.get_faulty_reports() {
    for report in &reports {
      errors.extend_from_refs(&report.errors());
    }
  }

  errors
}

/// Temporary simple disassembly implementation.
#[wasm_bindgen]
pub fn create_bytecode(js_db: &JSParserDB, states: &JSParseStates) -> Result<JSBytecode, PositionedErrors> {
  let mut j = Journal::new(None);

  j.set_active_report("bytecode compile", sherpa_core::ReportType::Any);
  
  let db = &js_db.0;


  let SherpaResult::Ok((bc, state_lu)) = compile_bytecode(&db, states.0.iter()) else {
    return Result::Err(convert_journal_errors(&mut j));
  };

  Ok(JSBytecode(Box::new((bc, state_lu))))
}

/// Temporary simple disassembly implementation.
#[wasm_bindgen]
pub fn create_bytecode_disassembly(bytecode: &JSBytecode) -> Result<String, PositionedErrors> {
  Ok(disassemble_bytecode(&bytecode.0.0))
}


/// Temporary simple disassembly of a single instruction
#[wasm_bindgen]
pub fn create_instruction_disassembly(address: u32, bytecode: &JSBytecode) -> String {
  disassemble_parse_block(Some((bytecode.0.0.as_slice(), address as usize).into()), false).0
}

