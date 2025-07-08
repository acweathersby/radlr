use js_sys::{Array, ArrayBuffer, JsString, Number, Uint8Array};
use radlr_ascript::AscriptDatabase;
use radlr_bytecode::compile_bytecode;
use radlr_core::{worker_pool::SingleThreadPool, RadlrGrammar, RadlrGrammarDatabase, *};
use radlr_rust_runtime::{
  kernel::{disassemble_bytecode, disassemble_parse_block},
  types::{
    bytecode::{ByteCodeIterator, Instruction, Opcode},
    BytecodeParserDB,
    RuntimeDatabase,
    TableHeaderData,
  },
};
use std::{collections::HashMap, io::BufWriter, path::PathBuf, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::{
  error::{ErrorOrigin, PositionedErrors},
  JSParserClassification,
  JSParserConfig,
};

/// A Grammar Identity
#[allow(unused)]
#[wasm_bindgen]
pub struct JSGrammarIdentities(pub(crate) Box<GrammarIdentities>);

/// A Parser database derived from grammar defined in a JSSoup
#[wasm_bindgen]
pub struct JSGrammarDB(pub(crate) Box<RadlrGrammarDatabase>);

#[wasm_bindgen]
impl JSGrammarDB {
  /// Maps a rule_id (integer) to a [begin(int), end(int)] pair marking
  /// the substring of the rule's definition in the grammar  source
  pub fn rule_id_to_grammar_offsets(&self, rule_id: u32) -> JsValue {
    let rule_id: usize = rule_id as usize;
    let db = self.0.get_internal();
    if rule_id < db.rules().len() {
      let rule = db.rule(rule_id.into());

      let start = rule.tok.get_start() as u32;
      let end = rule.tok.get_end() as u32;

      let array = js_sys::Array::default();
      array.push(&Number::from(start).into());
      array.push(&Number::from(end).into());
      array.into()
    } else {
      js_sys::Array::default().into()
    }
  }

  /// Maps a nt_id (integer) to the name assigned to nonterminal
  /// Returns an empty string if the nonterminal cannot be found.
  pub fn nonterm_id_to_name(&self, nt_id: u32) -> String {
    let nt_id: usize = nt_id as usize;
    let db = self.0.get_internal();
    if nt_id < db.parser_nonterms().len() {
      db.nonterm_friendly_name(nt_id.into()).to_string(db.string_store())
    } else {
      Default::default()
    }
  }

  /// Returns the ast generating JSON script for the grammar or any errors
  /// encountered while generating the script.
  pub fn get_ast_generating_script(&self) -> Result<String, PositionedErrors> {
    let atat_json_script = include_str!("../../atat_scripts/ast/json.atat");

    let ast_db: AscriptDatabase = (self.0.as_ref()).into();

    if let Some(errors) = ast_db.get_errors() {
      Err(PositionedErrors::from((&errors.to_vec(), ErrorOrigin::Grammar)))
    } else {
      let buf = BufWriter::new(Vec::new());

      let Ok(buf) = ast_db.format(atat_json_script, buf, 100, "", &[]) else {
        return Err(to_err(RadlrError::StaticText("Failed to create ast"), ErrorOrigin::Grammar));
      };

      let Ok(inner_buf) = buf.into_inner() else {
        return Err(to_err(RadlrError::StaticText("Could not read buffer bytes"), ErrorOrigin::Grammar));
      };

      match String::from_utf8(inner_buf) {
        Ok(result) => Ok(result),
        Err(..) => Err(to_err(RadlrError::StaticText("Could buffer bytes could not be read as UTF8"), ErrorOrigin::Grammar)),
      }
    }
  }
}

impl AsRef<RadlrGrammarDatabase> for JSGrammarDB {
  fn as_ref(&self) -> &RadlrGrammarDatabase {
    &self.0
  }
}

/// Parser states generated from the compilation of parser db
#[wasm_bindgen]
pub struct JSIRParser {
  pub(crate) states: Box<RadlrIRParser>,
}

#[wasm_bindgen]
impl JSIRParser {
  #[wasm_bindgen(getter)]
  pub fn classification(&self) -> JSParserClassification {
    self.states.get_classification().into()
  }

  #[wasm_bindgen(getter)]
  pub fn num_of_states(&self) -> JsValue {
    (self.states.get_metrics().num_of_states as u32).into()
  }

  #[wasm_bindgen(getter)]
  pub fn optimized(&self) -> JsValue {
    (self.states.get_metrics().optimized).into()
  }
}

impl AsRef<RadlrIRParser> for JSIRParser {
  fn as_ref(&self) -> &RadlrIRParser {
    &self.states
  }
}

/// An arbitrary collection of grammars
#[wasm_bindgen]
pub struct JSRadlrGrammar(pub(crate) Box<RadlrGrammar>);

impl AsRef<RadlrGrammar> for JSRadlrGrammar {
  fn as_ref(&self) -> &RadlrGrammar {
    &self.0
  }
}

impl AsMut<RadlrGrammar> for JSRadlrGrammar {
  fn as_mut(&mut self) -> &mut RadlrGrammar {
    &mut self.0
  }
}

/// Bytecode produced from parse states
#[allow(unused)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct JSBytecodeParserDB(pub(crate) Rc<BytecodeParserDB>, pub(crate) HashMap<u32, String>);

impl AsRef<[u8]> for JSBytecodeParserDB {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref().as_ref()
  }
}

fn to_err(e: RadlrError, origin: ErrorOrigin) -> PositionedErrors {
  (&vec![e], origin).into()
}

#[wasm_bindgen]
impl JSBytecodeParserDB {
  /// Returns the bytecode of the parser as bytes.
  #[wasm_bindgen(getter)]
  pub fn bytecode(&self) -> Uint8Array {
    let data = &self.0.bytecode;

    let buffer = Uint8Array::new_with_length(data.len() as u32);

    buffer.copy_from(data);

    buffer
  }

  /// A list of enterble non-terminal names and their respective bytecode entry
  /// point address address
  #[wasm_bindgen(getter)]
  pub fn entry_points(&self) -> JsValue {
    let entry_points = Array::new();

    for (name, address) in self.0.entrypoints() {
      let pair = Array::new();
      pair.push(&JsString::from(name).into());
      pair.push(&Number::from(address).into());
      entry_points.push(&pair.into());
    }

    entry_points.into()
  }
}

#[wasm_bindgen]
impl JSRadlrGrammar {
  /// Adds or replaces grammar in the soup, or throws an error
  /// if the grammar is invalid. Returns the grammar
  /// id if successful.
  pub fn add_grammar(&mut self, grammar_source: String, path: String) -> Result<JSGrammarIdentities, PositionedErrors> {
    let path = &PathBuf::from(&path);
    let grammar = self.as_mut();

    grammar.add_source_from_string(&grammar_source, &path, true).map_err(|e| to_err(e, ErrorOrigin::Grammar))?;

    let id: GrammarIdentities = grammar.path_to_id(path);

    Ok(JSGrammarIdentities(Box::new(id)))
  }

  /// Adds a non-terminal targeting a specific grammar
  pub fn add_nonterminal(&mut self, _grammar_name: String) -> Result<(), JsError> {
    Ok(())
  }
}

/// Creates an empty grammar soup object.
/// Use soup modifiers to add grammars and nonterminals
///
/// Pass soup to parser compiler functions to create parsers, generate bytecode,
/// and construct ASCript AST and CST structures.
#[wasm_bindgen]
pub fn create_soup() -> Result<JSRadlrGrammar, JsError> {
  Ok(JSRadlrGrammar(Box::new(RadlrGrammar::new())))
}

/// Creates a parser db from a soup and a root grammar, or returns semantic
/// errors.
#[wasm_bindgen]
pub fn create_grammar_db(
  grammar_id: String,
  soup: &JSRadlrGrammar,
  config: &JSParserConfig,
) -> Result<JSGrammarDB, PositionedErrors> {
  let grammar = soup.as_ref();

  let parser_db = grammar.build_db(&PathBuf::from(grammar_id), (*config).into()).map_err(|e| to_err(e, ErrorOrigin::Grammar))?;

  Ok(JSGrammarDB(Box::new(parser_db)))
}

/// Import a bytecode database from a JS ArrayBuffer
#[wasm_bindgen]
pub fn import_bytecode_db(buffer: ArrayBuffer) -> Result<JSBytecodeParserDB, PositionedErrors> {
  let buffer: Vec<u8> = Uint8Array::new(&buffer).to_vec();

  match radlr_lab::serialize::bytecode_db::import_bytecode_db(&buffer) {
    Ok(db) => {
      let nonterm_id_to_name = db.nonterm_name_to_id.iter().map(|(name, id)| (*id, name.clone())).collect();
      Ok(JSBytecodeParserDB(Rc::new(db), nonterm_id_to_name))
    }
    Err(err) => Err((RadlrError::from(err), ErrorOrigin::BytecodeImport).into()),
  }
}

/// Export a bytecode database into a JS ArrayBuffer
#[wasm_bindgen]
pub fn export_bytecode_db(states: &JSIRParser) -> Result<ArrayBuffer, PositionedErrors> {
  let bc = match compile_bytecode(states.states.as_ref(), true) {
    Err(errors) => return Result::Err(convert_journal_errors(errors)),
    Ok(pkg) => pkg,
  };

  let buffer = radlr_lab::serialize::bytecode_db::export_bytecode_db(&bc);

  let array_buffer = ArrayBuffer::new(buffer.len() as u32);

  let output = Uint8Array::new(&array_buffer);

  output.copy_from(&buffer);

  Ok(array_buffer)
}

#[wasm_bindgen]
pub fn create_parser_states(
  js_db: &JSGrammarDB,
  optimize_states: bool,
  config: &JSParserConfig,
) -> Result<JSIRParser, PositionedErrors> {
  let pool = SingleThreadPool {};

  let db = js_db.as_ref();

  let parser = db.build_states((*config).into(), &pool).map_err(|e| to_err(e, ErrorOrigin::StatesCreation))?;

  let parser =
    (if optimize_states { parser.build_ir_parser(true, true, &pool) } else { parser.build_ir_parser(false, true, &pool) })
      .map_err(|e| to_err(e, ErrorOrigin::StatesCreation))?;

  Ok(JSIRParser { states: Box::new(parser) })
}

fn convert_journal_errors(in_errors: RadlrError) -> PositionedErrors {
  let mut errors = PositionedErrors::default();

  errors.extend_from_refs(&in_errors.flatten(), ErrorOrigin::StatesCreation);

  errors
}

/// Temporary simple disassembly implementation.
#[wasm_bindgen]
pub fn create_bytecode_disassembly(pkg: &JSBytecodeParserDB) -> Result<String, PositionedErrors> {
  Ok(disassemble_bytecode(&pkg.0.bytecode))
}

/// Temporary simple disassembly of a single instruction
#[wasm_bindgen]
pub fn create_instruction_disassembly(address: u32, pkg: &JSBytecodeParserDB) -> String {
  disassemble_parse_block(Some((pkg.0.bytecode.as_slice(), address as usize).into()), false).0
}

/// Return a list of symbols ids if the opcode of the instruction is
/// HashTable or VecTable
#[wasm_bindgen]
pub fn get_debug_symbol_ids(address: u32, pkg: &JSBytecodeParserDB) -> JsValue {
  let bc = pkg.0.bytecode.as_slice();
  let i: Instruction = (bc, address as usize).into();

  if i.get_opcode() == Opcode::HashBranch {
    let TableHeaderData { table_start, table_length, .. } = i.into();
    let vals = (0..table_length as usize).map(|offset| {
      let mut iter: ByteCodeIterator = (bc, table_start + offset * 4).into();
      let cell = iter.next_u32_le().unwrap();
      let value = cell & 0x7FF;
      JsValue::from_f64(value as f64)
    });
    Array::from_iter(vals).into()
  } else {
    Default::default()
  }
}

#[wasm_bindgen]
pub fn get_debug_state_name(address: u32, pkg: &JSBytecodeParserDB, _db: &JSGrammarDB) -> JsValue {
  let bc_db = pkg.0.clone();

  if let Some(name) = bc_db.address_to_state_name.get(&(address)) {
    name.to_string().into()
  } else {
    "".into()
  }
}

#[wasm_bindgen]
pub fn is_instruction_transitory(address: u32, pkg: &JSBytecodeParserDB) -> bool {
  let i = Instruction::from((pkg.0.bytecode.as_slice(), address as usize));
  let opcode = i.get_opcode();
  match opcode {
    Opcode::ShiftToken
    | Opcode::ShiftChar
    | Opcode::ShiftTokenScanless
    | Opcode::SkipToken
    | Opcode::SkipTokenScanless
    | Opcode::PeekReset
    | Opcode::PeekSkipToken
    | Opcode::PeekTokenScanless
    | Opcode::PeekSkipTokenScanless
    | Opcode::PeekToken => true,
    _ => false,
  }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct TokenOffsets {
  pub start: u32,
  pub end:   u32,
}

/// Return a list of symbols ids if the opcode of the instruction is
/// Op::DebugExpectedSymbols
#[wasm_bindgen]
pub fn get_debug_tok_offsets(address: u32, pkg: &JSBytecodeParserDB) -> JsValue {
  if let Some(token) = pkg.0.ir_token_lookup.get(&(address)) {
    (TokenOffsets { start: token.get_start() as u32, end: token.get_end() as u32 }).into()
  } else {
    Default::default()
  }
}

#[wasm_bindgen]
pub fn get_state_source_string(name: String, states: &JSIRParser) -> JsValue {
  let lu_name: IString = name.to_token();

  let parser = states.states.as_ref();

  let code = parser.get_states().find(|f| f.0 == lu_name).map(|f| f.1.print(parser.get_db(), true).unwrap_or_default());

  code.into()
}

/// Givin an symbol index, returns the symbol's friendly name.
#[wasm_bindgen]
pub fn get_symbol_name_from_id(id: u32, db: &JSGrammarDB) -> JsValue {
  let db = db.as_ref().get_internal();
  db.token(id.into()).name.to_string(db.string_store()).into()
}

/// Returns a list of entrypoint names
#[wasm_bindgen]
pub fn get_entry_names(db: &JSGrammarDB) -> JsValue {
  let db = db.as_ref().get_internal();
  db.entry_points().iter().map(|ep| JsValue::from(ep.entry_name.to_string(db.string_store()))).collect::<Array>().into()
}

#[wasm_bindgen]
pub fn get_nonterminal_name_from_id(id: u32, db: &JSBytecodeParserDB) -> String {
  db.0.nonterm_id_to_name.get(&id).cloned().unwrap_or_default()
}

/// Returns a diagram of a grammar rule  
#[wasm_bindgen]
pub fn get_rule_expression_string(rule_id: u32, db: &JSBytecodeParserDB) -> String {
  db.0.rule_diagram.get(&rule_id).cloned().unwrap_or_default()
}

/// Returns the offset and length of a token rule.
#[wasm_bindgen]
pub fn get_rule_location(rule_id: u32, db: &JSBytecodeParserDB) -> JsValue {
  match db.0.rule_offsets.get(&rule_id) {
    Some((start, end)) => {
      let array = Array::new();

      array.push(&JsValue::from_f64(*start as f64));
      array.push(&JsValue::from_f64(*end as f64));

      array.into()
    }
    _ => Array::new().into(),
  }
}

#[wasm_bindgen]
pub fn get_nonterminal_names_from_db(db: &JSGrammarDB) -> JsValue {
  let db = db.as_ref().get_internal();

  let array = Array::new();

  for non_term_key in db.parser_nonterms() {
    let non_term = db.nonterm_friendly_name(non_term_key);
    array.push(&non_term.to_str(db.string_store()).as_str().into());
  }

  return array.into();
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub enum JSReductionType {
  /// Any reduction resulting in the execution of a some kind of semantic
  /// action. At this point only `:ast` semantic actions are available.
  SemanticAction,
  /// A reduction of a terminal symbol to a nonterminal
  SingleTerminal,
  /// A reduction of single nonterminal symbol to another nonterminal
  SingleNonTerminal,
  /// A reduction of a left-recursive rule
  LeftRecursive,
  /// A reduction of more than one symbol to a nonterminal
  #[default]
  Mixed,
}

impl From<ReductionType> for JSReductionType {
  fn from(value: ReductionType) -> Self {
    match value {
      ReductionType::SemanticAction => JSReductionType::SemanticAction,
      ReductionType::SingleTerminal => JSReductionType::SingleTerminal,
      ReductionType::SingleNonTerminal => JSReductionType::SingleNonTerminal,
      ReductionType::LeftRecursive => JSReductionType::LeftRecursive,
      ReductionType::Mixed => JSReductionType::Mixed,
    }
  }
}

#[wasm_bindgen]
pub fn get_rule_reduce_type(id: u32, db: &JSGrammarDB) -> JSReductionType {
  let db = db.as_ref().get_internal();
  if (id as usize) < db.rules().len() {
    JSReductionType::from(db.get_reduce_type(DBRuleKey::from(id)))
  } else {
    JSReductionType::default()
  }
}
