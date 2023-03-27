#![allow(unused)]

use std::{hash::Hash, path::PathBuf, sync::Arc};

use sherpa_runtime::{
  types::{Token, TokenRange},
  utf8::lookup_table::CodePointClass,
};

use crate::{
  compile::ParseState,
  grammar::hash_id_value_u64,
  types::DEFAULT_SYM_ID,
  writer::code_writer::CodeWriter,
};

use super::{
  super::parser::State,
  Array,
  CachedString,
  IString,
  IStringStore,
  Map,
  Set,
  SymbolId,
};

/// A globally unique identifier for a single production.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ProductionId {
  /// Productions directly defined within a grammar.
  Standard(u64, ProductionSubType),
  /// Productions derived from grammar symbols such as the
  /// group `(...)` symbol. All sub productions belong to
  /// only one "Standard" production
  Sub(u64, u32, ProductionSubType),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ProductionSubType {
  Parser,
  Scanner,
  ScannerToken,
  ScannerSym,
}

impl From<(GrammarId, &str)> for ProductionId {
  fn from(value: (GrammarId, &str)) -> Self {
    ProductionId::Standard(hash_id_value_u64(value), ProductionSubType::Parser)
  }
}

impl From<(ProductionId, usize)> for ProductionId {
  fn from((prod_id, index): (ProductionId, usize)) -> Self {
    if let ProductionId::Standard(id, sub_type) = prod_id {
      ProductionId::Sub(id, index as u32, sub_type)
    } else {
      unreachable!()
    }
  }
}

impl ProductionId {
  pub fn as_sym(&self) -> SymbolId {
    SymbolId::NonTerminal { id: self.as_parse_prod() }
  }

  pub fn as_tok_sym(&self) -> SymbolId {
    SymbolId::NonTerminalToken {
      id:         self.as_scan_prod(),
      precedence: 0,
    }
  }

  pub fn as_parse_prod(&self) -> ProductionId {
    match self {
      ProductionId::Standard(id, _) => {
        ProductionId::Standard(*id, ProductionSubType::Parser)
      }
      ProductionId::Sub(id, index, _) => {
        ProductionId::Sub(*id, *index, ProductionSubType::Parser)
      }
    }
  }

  pub fn as_scan_prod(&self) -> ProductionId {
    match self {
      ProductionId::Standard(id, _) => {
        ProductionId::Standard(*id, ProductionSubType::Scanner)
      }
      ProductionId::Sub(id, index, _) => {
        ProductionId::Sub(*id, *index, ProductionSubType::Scanner)
      }
    }
  }

  pub fn set_index(&mut self, index: usize) {
    match self {
      ProductionId::Standard(id, ..) | ProductionId::Sub(id, ..) => {
        *id = index as u64;
      }
    }
  }
}

/// A globally unique identifier for a single grammar file.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct GrammarId(u64);

impl From<&PathBuf> for GrammarId {
  fn from(value: &PathBuf) -> Self {
    GrammarId(hash_id_value_u64(&value))
  }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RuleId(u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TokenSymbol {
  pub type_: SymbolType,
  pub val:   IString,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ProductionRef(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TokenProductionRef(u32);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Rule {
  /// A list of [SymbolId]s and their position within the source grammar
  pub symbols: Array<(SymbolId, usize)>,
  pub skipped: Array<SymbolId>,
  pub ast:     Option<ASTToken>,
}

/// A reference to some Ascript AST data that is either automatically generated
/// depending on the reference type, or is stored on a Production node.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ASTToken {
  /// Represents the ast expression `:ast [ $1 ]`.
  ///
  ///
  /// Automatically generated when when a list production (` A(+) | A(*) `) is
  /// processed.
  ListEntry(TokenRange),
  /// Represents the ast expression `:ast [ $1, $--last-- ]`, where `--last--`
  /// represents the last symbol in a rule.
  ///
  /// Automatically generated when when a list production (` A(+) | A(*) `) is
  /// processed.
  ListIterate(TokenRange),
  /// An AST expression defined within a grammar. `0` Is the production id
  /// in which a copy if the AST expressions is stored. `1` is the index
  /// into the Productions's `asts` array for that stored production.
  Defined(ProductionId, usize),
}

/// A custom parse state defined within a grammar e.g `state_name => ...`
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CustomState {
  pub id:      ProductionId,
  pub g_id:    GrammarId,
  pub name:    IString,
  pub symbols: Set<SymbolId>,
  pub state:   Box<State>,
  pub tok:     Token,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Production {
  /// The unique identifier of this production.
  pub id: ProductionId,

  /// The unique identifier of the owning GrammarHEader.
  pub g_id: GrammarId,

  /// All symbols that are referenced by the rules of the
  /// production and its sub-productions.
  pub symbols: Set<SymbolId>,

  /// All rules that reduce to this production
  pub rules: Array<Rule>,

  /// Productions generated from the expansion of "production" type
  /// symbols such as groups & lists. These productions are only referenced
  /// by the rules defined by this production.
  pub sub_prods: Array<Box<SubProduction>>,

  /// Productions derived from `tk:` invocations of normal productions.
  /// These productions have the special characteristic where none of
  /// their rules contain left recursions
  pub tok_prods: Array<Box<SubProduction>>,

  /// The type of this production
  pub type_: ProductionType,

  /// The name of the production as it is found in the source grammar.
  pub name: IString,

  pub tok: Token,

  pub asts: Array<Box<super::super::parser::Ascript>>,
}

/// Productions generated from the expansion of "production" type
/// symbols such as groups & lists. These productions are only referenced
/// by the rules defined by this production.
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SubProduction {
  pub id: ProductionId,

  pub g_id: GrammarId,

  pub name: IString,

  pub rules: Array<Rule>,

  pub type_: SubProductionType,
}

/// Types of [SubProduction]s that may be derived from rule symbols.
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SubProductionType {
  /// List sub productions are left recursive productions
  /// that are derived from `list` symbols e.g: `A(+) | A(*) | A(+sym) |
  /// A(*sym)` .
  List,
  /// Group productions are derived from group symbols e.g `(...)` and are
  /// created when they are present in rules that have AST definitions to
  /// maintain expected behaviors when referencing symbols in an ast
  /// expression.
  Group,
}

impl SubProductionType {
  pub fn to_string(&self) -> String {
    match self {
      SubProductionType::Group => "group".into(),
      SubProductionType::List => "list".into(),
    }
  }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct GrammarHeader {
  pub identity:  GrammarIdentity,
  /// Productions that are accessible as entry points to this
  /// grammar. Contains the global id of the public production
  /// and its export name.
  pub pub_prods: Map<IString, ProductionId>,

  pub imports: Array<GrammarId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SymbolType {
  /// A single token string
  Token,
  /// A single, tokenized, production
  TokenProduction(TokenProductionRef),
  Production(ProductionRef),
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ProductionType {
  ContextFree,
  Pratt,
  Peg,
  ParseSTate,
}

/// Identifiers for a Grammar
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct GrammarIdentity {
  /// A globally unique identifier for this GrammarStore instance. Derived
  /// from the source path
  pub guid: GrammarId,

  /// A name defined by the grammar author. This is either the value of the
  /// `@NAME` preamble, or the original file name stem if this preamble is
  /// not present.
  pub name: IString,

  /// The absolute path of the grammar's source file. This may be empty if the
  /// source code was passed in as a string, as with the case of grammars
  /// compiled with
  /// [compile_grammar_from_string](sherpa_core::grammar::compile_grammar_from_string)).
  pub path: IString,
}

impl GrammarIdentity {
  pub fn from_path(
    grammar_source_path: &PathBuf,
    string_store: &IStringStore,
  ) -> Self {
    Self {
      guid: grammar_source_path.into(),
      path: grammar_source_path.intern(string_store),
      ..Default::default()
    }
  }
}

use super::ParserDatabase;
#[cfg(debug_assertions)]
impl SymbolId {
  pub fn debug_string(&self, db: &ParserDatabase) -> String {
    use SymbolId::*;

    match *self {
      Undefined => "Undefine".into(),
      Default => "Default".into(),
      EndOfFile { .. } => "{EOF}".into(),
      ClassSpace { .. } => "c:sp".into(),
      ClassHorizontalTab { .. } => "c:tab".into(),
      ClassNewLine { .. } => "c:nl".into(),
      ClassIdentifier { .. } => "c:id".into(),
      ClassNumber { .. } => "c:num".into(),
      ClassSymbol { .. } => "c:sym".into(),
      Token { val, precedence } => {
        format!(
          "[\"{}\"]{{{precedence}}}",
          val.to_str(db.string_store()).as_str()
        )
      }
      NonTerminal { id, .. } => format!("<non-term:{id:?}>"),
      NonTerminalToken { id, .. } => format!("tk:<non-term:{id:?}>"),
      Codepoint { val, .. } => format!("cp:{}", val),
      DBNonTerminal { key } => {
        let guard_str = db.prod_name_str(key);
        let name = guard_str.as_str();
        format!("{name}")
      }
      DBNonTerminalToken { prod_key, precedence, .. } => {
        let guard_str = db.prod_name_str(prod_key);
        let name = guard_str.as_str();

        format!("tk:{name}{{{precedence}}}")
      }
      DBToken { key: index } => db.sym(index).debug_string(db),
      Char { char, precedence } => {
        if char < 128 {
          format!("[\"{}\"]{{{precedence}}}", char::from(char))
        } else {
          format!("[0x{char:X}]{{{precedence}}}")
        }
      }
    }
  }
}

use ::std::sync;
/// This contains all grammars, productions, and parser states that have
/// been derived from source grammar inputs.
///
/// This object is generally only created once and then passed to entry
/// functions for parser, compilers, and analyzers, with which appropriate
/// derivatives can be created for the respective task.
#[derive(Clone, Default)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct GrammarSoup {
  pub grammar_headers: Arc<sync::RwLock<Map<GrammarId, Box<GrammarHeader>>>>,
  pub productions:     Arc<sync::RwLock<Map<ProductionId, Box<Production>>>>,
  pub custom_states:   Arc<sync::RwLock<Map<ProductionId, Box<CustomState>>>>,
  pub string_store:    IStringStore,
}

impl GrammarSoup {
  pub fn new() -> sync::Arc<Self> {
    sync::Arc::new(GrammarSoup {
      grammar_headers: Default::default(),
      productions:     Default::default(),
      custom_states:   Default::default(),
      string_store:    Default::default(),
    })
  }
}

struct ParseBuild {
  root_grammar: GrammarId,
  entries:      Vec<ProductionId>,
  states:       Map<IString, Box<ParseState>>,
  bytecode:     Option<Vec<u8>>,
}
