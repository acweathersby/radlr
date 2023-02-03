use std::sync::Arc;

use bitmask_enum::bitmask;

use crate::{
  grammar::{
    compile::parser::sherpa::{ASTNode, Ascript},
    hash_id_value_u64,
  },
  types::*,
};

use super::{GrammarRef, Item};

#[bitmask]
pub enum RecursionType {
  NONE  = 0,
  LEFT_DIRECT = 1,
  LEFT_INDIRECT = 2,
  RIGHT = 4,
}

/// A convenient wrapper around information used to construct parser entry points
/// based on [productions](Production).
pub struct ExportedProduction<'a> {
  /// The name assigned to the production within the
  /// export clause of a grammar.
  /// e.g. `@EXPORT production as <export_name>`
  pub export_name: &'a str,
  /// The GUID name assigned of the corresponding production.
  pub guid_name:   &'a str,
  /// The exported production.
  pub production:  &'a Production,
}

impl Default for RecursionType {
  fn default() -> Self {
    RecursionType::NONE
  }
}
/// TODO: Docs
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct ProductionId(pub u64);

impl From<&String> for ProductionId {
  fn from(string: &String) -> Self {
    ProductionId(hash_id_value_u64(string))
  }
}

impl From<SymbolID> for ProductionId {
  fn from(sym: SymbolID) -> Self {
    ProductionId(hash_id_value_u64(sym))
  }
}

impl std::fmt::Display for ProductionId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0.to_string())
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]

pub struct RuleId(pub u64);
impl RuleId {
  pub fn new(prod_id: &ProductionId, rule_index: usize) -> Self {
    RuleId((prod_id.0 & 0xFFFF_FFFF_FFFF_F000) + rule_index as u64)
  }

  pub fn from_syms(syms: &[SymbolID]) -> Self {
    let val = hash_id_value_u64(syms);
    RuleId(val)
  }

  #[inline(always)]
  pub fn default() -> Self {
    Self(0)
  }

  #[inline(always)]
  pub fn is_null(&self) -> bool {
    self.0 == 0
  }
}

impl std::fmt::Display for RuleId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0.to_string())
  }
}
/// TODO: Docs
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Production {
  /// TODO: Docs
  pub id: ProductionId,
  /// A globally unique name of this production. This should always be distinct, particularly in cases
  /// were a host grammar imports a donor grammar that
  /// defines productions with the same name as those
  /// in the host.
  pub guid_name: String,
  /// The human friendly name of this production
  pub name: String,
  /// TODO: Docs
  pub number_of_rules: u16,
  /// TODO: Docs
  pub is_scanner: bool,
  /// TODO: Docs
  pub is_entry: bool,
  /// TODO: Docs
  pub recursion_type: RecursionType,
  /// TODO: Docs
  pub priority: u32,
  /// The token defining the substring in the source
  /// code from which this production was derived.
  pub loc: Token,
  /// An integer value used by bytecode
  /// to refer to this production
  pub bytecode_id: u32,
  /// If this is a scanner production,
  /// then this is a non-zero integer value
  /// that mirrors the TokenProduction or Defined* symbol
  /// bytecode_id that this production produces.
  pub symbol_bytecode_id: u32,
  /// The symbol of this production
  pub sym_id: SymbolID,
}

/// A wrapper around a symbol that includes unique information
/// relating a symbol to a particular production rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuleSymbol {
  pub sym_id: SymbolID,

  /// The 0 index position of this symbol within the original rule
  /// definition. If this symbol was generated by a compiler step,
  /// then this value will be 9999, indicating that the symbol does
  /// not exist in the source grammar.
  pub original_index: u32,

  /// The annotated name of this symbol: defined by `^<annotation>`
  pub annotation: String,

  /// If false, this symbol does not produce shift actions.
  pub consumable: bool,

  /// The number of related symbols that comprise
  /// a scanned token. For use by scanner code.
  /// If this symbol does not exist in scanner space then it is
  /// set to 0
  pub scanner_length: u32,

  /// The zero-based sequence index of this symbol in relation
  /// to other related symbols that comprise a scanned token.
  /// If this symbol does not exist in scanner space then it is
  /// set to 0
  pub scanner_index: u32,

  /// Always captures, regardless of other symbols
  pub exclusive: bool,

  pub tok: Token,

  /// A reference to the identifiers of the owning grammar.
  pub grammar_ref: Arc<GrammarRef>,
}

impl Default for RuleSymbol {
  fn default() -> Self {
    Self {
      sym_id: Default::default(),
      original_index: Default::default(),
      annotation: Default::default(),
      consumable: Default::default(),
      scanner_length: Default::default(),
      scanner_index: Default::default(),
      exclusive: Default::default(),
      tok: Default::default(),
      grammar_ref: Arc::new(Default::default()),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ReduceFunctionId(u64);

impl ReduceFunctionId {
  pub fn new(reduce_function: &ASTNode) -> Self {
    ReduceFunctionId(hash_id_value_u64(reduce_function.to_string()))
  }

  pub fn from_token(token: Token) -> Self {
    ReduceFunctionId(hash_id_value_u64(token))
  }

  pub fn is_undefined(&self) -> bool {
    self.0 == 0
  }
}

/// A single rule derived from a production
#[derive(Debug, Clone, Default)]
pub struct Rule {
  /// TODO: Docs
  pub syms: Vec<RuleSymbol>,
  /// TODO: Docs
  pub len: u16,
  /// TODO: Docs
  pub prod_id: ProductionId,
  /// TODO: Docs
  pub id: RuleId,
  /// TODO: Docs
  pub bytecode_id: u32,
  /// TODO: Docs
  pub ast_definition: Option<Ascript>,
  /// A token that covers the definition of this rule.
  pub tok: Token,
  /// A reference to the identifiers of the owning grammar.
  pub grammar_ref: Arc<GrammarRef>,
}

impl PartialEq for Rule {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl PartialOrd for Rule {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.id.partial_cmp(&other.id)
  }
}
impl Eq for Rule {}
impl Ord for Rule {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.id.cmp(&other.id)
  }
}

impl Rule {
  pub(crate) fn item(&self) -> Item {
    Item::from(self)
  }

  /// Creates a String diagram of the Rule.
  pub fn blame_string(&self, g: &GrammarStore) -> String {
    self.item().blame_string(g)
  }

  /// Returns the last [SymbolID] of the rule.
  pub fn last_symbol_id(&self) -> SymbolID {
    self.syms.last().map(|i| i.sym_id).unwrap()
  }
}

/// Maps a [ProductionId] to a [Production].
pub type ProductionTable = std::collections::BTreeMap<ProductionId, Production>;

/// Maps [ProductionId] to a vector of [RuleId](RuleId).
pub type ProductionBodiesTable = std::collections::BTreeMap<ProductionId, Vec<RuleId>>;

pub type RuleTable = std::collections::BTreeMap<RuleId, Rule>;
