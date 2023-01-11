use sherpa_runtime::{types::END_OF_INPUT_TOKEN_ID, utf8::lookup_table::CodePointClass};

use super::{GrammarId, GrammarRef, GrammarStore, ProductionId, SherpaResult, Token};
use crate::grammar::{get_scanner_info_from_defined, uuid::hash_id_value_u64};

use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Display,
  sync::Arc,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct StringId(pub u64);

impl From<&String> for StringId {
  fn from(string: &String) -> Self {
    StringId(hash_id_value_u64(string))
  }
}

impl Display for StringId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0.to_string())
  }
}

pub const space_sym_str: &str = "g:sp";
pub const nl_sym_str: &str = "g:nl";
pub const symbol_sym_str: &str = "g:sym";
pub const id_sym_str: &str = "g:id";
pub const num_sym_str: &str = "g:num";
pub const gen_rec_marker_str: &str = "g:rec";
pub const tab_sym_str: &str = "g:tab";
pub const eof_str: &str = "$eof";
pub const undefined_symbol_id: u32 = 99999;
/// TODO: Docs
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash, Eq, Ord)]
pub enum SymbolID {
  /// Represents a defined sequence of characters from the set `0-9`.
  DefinedNumeric(StringId),

  /// Represents a defined sequence of characters that are members of the
  /// unicode character classes `ID_Start` and `ID_Nonstart`.
  DefinedIdentifier(StringId),

  /// Represents any defined token sequence that contains a mixture of
  /// identifier, numeric, and general characters. Examples include
  /// `ba$$`, `#123`, `$10.20`, and `h3ll0W0rld!`.
  DefinedSymbol(StringId),

  /// Represents a defined sequence of characters from the set `0-9`.
  ExclusiveDefinedNumeric(StringId),

  /// Represents a defined sequence of characters that are members of the
  /// unicode character classes `ID_Start` and `ID_Nonstart`.
  ExclusiveDefinedIdentifier(StringId),

  /// Represents any defined token sequence that contains a mixture of
  /// identifier, numeric, and general characters. Examples include
  /// `ba$$`, `#123`, `$10.20`, and `h3ll0W0rld!`.
  ExclusiveDefinedSymbol(StringId),

  /// Represents any non-terminal production symbol.
  Production(ProductionId, GrammarId),

  /// Represents any terminal production symbol defined through
  /// the production token specifier, `tk:`, as in `tk:<production_name>`.
  /// - `.0` = The base ProductionID,
  /// - `.1` = The host GrammarID
  /// - `.2` = The scanner ProductionID. This production is created
  ///      when the root grammar is finalized
  TokenProduction(ProductionId, GrammarId, ProductionId),

  /// Represent the grammar symbol `g:sp`.
  GenericSpace,

  /// Represent the grammar symbol `g:tab`.
  GenericHorizontalTab,

  /// Represent the grammar symbol `g:nl`.
  GenericNewLine,

  /// Represent the grammar symbol `g:id`.
  GenericIdentifier,

  /// Represent the grammar symbol `g:num`.
  GenericNumber,

  /// Represent the grammar symbol `g:sym`.
  GenericSymbol,

  /// Represent the grammar symbol `g:rec`.
  Recovery,
  Default,
  /// TODO: Docs
  /// Represent end of input. This is also used
  /// to represent the `symbol` of items in the completed
  /// position
  EndOfInput,
  /// Used to differentiate different completed items in the
  /// `peek` module.
  DistinctGroup(u32),
  /// TODO: Docs
  Undefined,
  /// TODO: Docs
  UndefinedA,
  /// TODO: Docs
  UndefinedB,
  /// TODO: Docs
  UndefinedC,
  /// TODO: Docs
  UndefinedD,
  /// TODO: Docs
  Start,
  /// Represents a symbol that does not belong to
  /// a given closure, used to detect and resolve shift
  /// conflicts within nested productions.
  OutOfScope,
}

impl Default for SymbolID {
  fn default() -> Self {
    SymbolID::Undefined
  }
}

impl SymbolID {
  /// TODO: Docs
  pub const DefinedSymbolIndexBasis: u32 = 8;
  /// TODO: Docs
  pub const Generics: [SymbolID; 6] = [
    SymbolID::GenericSpace,
    SymbolID::GenericHorizontalTab,
    SymbolID::GenericNewLine,
    SymbolID::GenericIdentifier,
    SymbolID::GenericNumber,
    SymbolID::GenericSymbol,
  ];

  /// TODO: Docs
  pub fn from_string(symbol_string: &str, g: Option<&GrammarStore>) -> Self {
    match symbol_string {
      "default" => Self::Default,
      "[??]" => Self::Undefined,
      gen_rec_marker_str => Self::Recovery,
      eof_str => Self::EndOfInput,
      tab_sym_str => Self::GenericHorizontalTab,
      nl_sym_str => Self::GenericNewLine,
      space_sym_str => Self::GenericSpace,
      id_sym_str => Self::GenericIdentifier,
      num_sym_str => Self::GenericNumber,
      symbol_sym_str => Self::GenericSymbol,
      _ => {
        if let Some(g) = g {
          match g.symbol_strings.iter().find(|(_, string)| string.as_str() == symbol_string) {
            Some((sym_id, _)) => *sym_id,
            _ => match g.get_production_by_name(symbol_string) {
              SherpaResult::Ok(prod) => get_scanner_info_from_defined(&prod.sym_id, g).0,
              _ => SymbolID::Undefined,
            },
          }
        } else {
          SymbolID::Undefined
        }
      }
    }
  }

  /// Returns a human friendly string representation
  pub fn to_string(&self, g: &GrammarStore) -> String {
    match self {
      Self::DefinedNumeric(_)
      | Self::DefinedIdentifier(_)
      | Self::DefinedSymbol(_)
      | Self::ExclusiveDefinedNumeric(_)
      | Self::ExclusiveDefinedIdentifier(_)
      | Self::ExclusiveDefinedSymbol(_) => {
        format!("\\{}", g.symbol_strings.get(self).unwrap())
      }
      Self::Production(prod_id, _) => g.productions.get(prod_id).unwrap().name.to_string(),
      Self::TokenProduction(.., prod_id) => {
        let name = &g.productions.get(prod_id).unwrap().name;
        if name.starts_with("tk:") {
          name.clone()
        } else {
          format!("tk:{}", name)
        }
      }
      Self::Start => "start".to_string(),
      Self::Default => "default".to_string(),
      Self::Recovery => gen_rec_marker_str.to_string(),
      Self::EndOfInput => eof_str.to_string(),
      Self::GenericHorizontalTab => tab_sym_str.to_string(),
      Self::GenericNewLine => nl_sym_str.to_string(),
      Self::GenericSpace => space_sym_str.to_string(),
      Self::GenericIdentifier => id_sym_str.to_string(),
      Self::GenericNumber => num_sym_str.to_string(),
      Self::GenericSymbol => symbol_sym_str.to_string(),
      _ => "[??]".to_string(),
    }
  }

  /// TODO: Docs
  pub fn to_default_string(&self) -> String {
    match self {
      Self::DefinedNumeric(_)
      | Self::DefinedIdentifier(_)
      | Self::DefinedSymbol(_)
      | Self::ExclusiveDefinedNumeric(_)
      | Self::ExclusiveDefinedIdentifier(_)
      | Self::ExclusiveDefinedSymbol(_) => "__defined".to_string(),
      Self::Production(..) => "__defined".to_string(),
      Self::TokenProduction(..) => "__defined".to_string(),
      Self::Default => "__default".to_string(),
      Self::Start => "__start".to_string(),
      Self::Recovery => "__rec".to_string(),
      Self::EndOfInput => "__eof".to_string(),
      Self::GenericHorizontalTab => "__tab".to_string(),
      Self::GenericNewLine => "__nl".to_string(),
      Self::GenericSpace => "__sp".to_string(),
      Self::GenericIdentifier => "__id".to_string(),
      Self::GenericNumber => "__num".to_string(),
      Self::GenericSymbol => "__sym".to_string(),
      _ => "__undefined".to_string(),
    }
  }

  /// Returns a tuple indicating the type CLASS of shift that is performed
  /// on this symbol.
  pub fn shift_info(&self, g: &GrammarStore) -> (u32, &'static str) {
    match self {
      SymbolID::GenericSpace
      | SymbolID::GenericHorizontalTab
      | SymbolID::GenericNewLine
      | SymbolID::GenericIdentifier
      | SymbolID::GenericNumber
      | SymbolID::GenericSymbol => (self.bytecode_id(Some(g)), "CLASS"),
      SymbolID::ExclusiveDefinedIdentifier(..)
      | SymbolID::ExclusiveDefinedNumeric(..)
      | SymbolID::ExclusiveDefinedSymbol(..)
      | SymbolID::DefinedNumeric(..)
      | SymbolID::DefinedIdentifier(..)
      | SymbolID::DefinedSymbol(..) => {
        let symbol = g.symbols.get(self).unwrap();
        let id = g.symbol_strings.get(self).unwrap();
        let sym_char = id.as_bytes()[0];
        if symbol.byte_length > 1 || sym_char > 128 {
          (symbol.bytecode_id, "CODEPOINT")
        } else {
          (sym_char as u32, "BYTE")
        }
      }
      _ => (0, "BYTE"),
    }
  }

  /// TODO: Docs
  pub fn is_token_production(&self) -> bool {
    match self {
      Self::TokenProduction(..) => true,
      _ => false,
    }
  }

  /// TODO: Docs
  pub fn is_defined(&self) -> bool {
    match self {
      Self::DefinedNumeric(_)
      | Self::DefinedIdentifier(_)
      | Self::DefinedSymbol(_)
      | Self::ExclusiveDefinedNumeric(_)
      | Self::ExclusiveDefinedIdentifier(_)
      | Self::ExclusiveDefinedSymbol(_) => true,
      _ => false,
    }
  }

  /// TODO: Docs
  pub fn is_exclusive(&self) -> bool {
    match self {
      Self::ExclusiveDefinedNumeric(_)
      | Self::ExclusiveDefinedIdentifier(_)
      | Self::ExclusiveDefinedSymbol(_) => true,
      _ => false,
    }
  }

  /// TODO: Docs
  pub fn is_production(&self) -> bool {
    self.get_production_id().is_some()
  }

  /// TODO: Docs
  pub fn get_production_id(&self) -> Option<ProductionId> {
    match self {
      Self::Production(id, _) => Some(*id),
      Self::TokenProduction(.., id) => Some(*id),
      _ => None,
    }
  }

  /// TODO: Docs
  pub fn get_grammar_id(&self) -> GrammarId {
    match self {
      Self::Production(_, id) | Self::TokenProduction(.., id, _) => *id,
      _ => GrammarId::default(),
    }
  }

  /// TODO: Docs
  pub fn bytecode_id(&self, g: Option<&GrammarStore>) -> u32 {
    match self {
      Self::DefinedNumeric(_)
      | Self::DefinedIdentifier(_)
      | Self::DefinedSymbol(_)
      | Self::ExclusiveDefinedNumeric(_)
      | Self::ExclusiveDefinedIdentifier(_)
      | Self::ExclusiveDefinedSymbol(_) => {
        if let Some(g) = g {
          g.symbols.get(self).unwrap().bytecode_id
        } else {
          undefined_symbol_id
        }
      }
      Self::TokenProduction(.., prod_id) => match g {
        Some(g) => g.get_production(prod_id).unwrap().symbol_bytecode_id,
        None => undefined_symbol_id,
      },
      Self::Production(prod_id, _) => match g {
        Some(g) => g.get_production(prod_id).unwrap().bytecode_id,
        None => undefined_symbol_id,
      },
      Self::Default | Self::Start => undefined_symbol_id,
      Self::EndOfInput => END_OF_INPUT_TOKEN_ID,
      Self::GenericHorizontalTab => CodePointClass::HorizontalTab as u32,
      Self::GenericNewLine => CodePointClass::NewLine as u32,
      Self::GenericSpace => CodePointClass::Space as u32,
      Self::GenericIdentifier => CodePointClass::Identifier as u32,
      Self::GenericNumber => CodePointClass::Number as u32,
      Self::GenericSymbol => CodePointClass::Symbol as u32,
      _ => 0,
    }
  }
}

pub type SymbolUUID = SymbolID;

/// TODO: Docs
#[repr(C, align(64))]
#[derive(Debug, Clone, Default)]
pub struct Symbol {
  /// The globally unique identifier of this symbol
  /// which encapsulates the set of Symbols that are
  /// unique based on the combination of the symbol's
  /// class_id,
  pub guid:          SymbolUUID,
  /// The unique identifier of the class of this symbol
  /// which either identifies symbol's generic class id
  /// i.e (g:sp , g:nl, g:tab, g:id ...) or by the unique
  /// or the explicit character sequence this symbol represents.
  pub bytecode_id:   u32,
  /// The length in bytes of the character sequence
  /// represented by this symbol
  pub byte_length:   u32,
  /// The number of utf8 code points represented by
  /// this symbol.
  pub cp_len:        u32,
  ////
  /// True if only scanner productions use
  /// this symbol
  pub scanner_only:  bool,
  /// A name that can be used in debug and
  /// error reports .
  pub friendly_name: String,
  /// The first location this symbol was identified
  pub loc:           Token,
  /// TODO: Docs
  pub g_ref:         Option<Arc<GrammarRef>>,
}

impl Symbol {
  /// TODO: Docs
  pub const Generics: [&'static Symbol; 6] = [
    &Symbol {
      guid:          SymbolID::GenericSpace,
      bytecode_id:   CodePointClass::Space as u32,
      cp_len:        1,
      byte_length:   1,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
    &Symbol {
      guid:          SymbolID::GenericHorizontalTab,
      bytecode_id:   CodePointClass::HorizontalTab as u32,
      byte_length:   1,
      cp_len:        1,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
    &Symbol {
      guid:          SymbolID::GenericNewLine,
      bytecode_id:   CodePointClass::NewLine as u32,
      byte_length:   1,
      cp_len:        1,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
    &Symbol {
      guid:          SymbolID::GenericIdentifier,
      bytecode_id:   CodePointClass::Identifier as u32,
      cp_len:        1,
      byte_length:   0,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
    &Symbol {
      guid:          SymbolID::GenericNumber,
      bytecode_id:   CodePointClass::Number as u32,
      cp_len:        1,
      byte_length:   0,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
    &Symbol {
      guid:          SymbolID::GenericSymbol,
      bytecode_id:   CodePointClass::Symbol as u32,
      cp_len:        1,
      byte_length:   0,
      friendly_name: String::new(),
      loc:           Token::empty(),
      scanner_only:  false,
      g_ref:         None,
    },
  ];

  /// TODO: Docs
  pub fn generics_lu() -> BTreeMap<SymbolID, &'static Symbol> {
    BTreeMap::from_iter(Self::Generics.clone().iter().map(|s| (s.guid, *s)))
  }
}

/// A table that maps a symbol class_id to a utf8 string.

pub type SymbolStringTable = BTreeMap<SymbolID, String>;

/// A table that contains defined symbols () keyed by their [SymbolUUID].
pub type SymbolsTable = BTreeMap<SymbolUUID, Symbol>;

pub type SymbolSet = BTreeSet<SymbolID>;
