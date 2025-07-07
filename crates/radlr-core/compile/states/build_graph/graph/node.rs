use crate::{
  compile::states::build_graph::graph::{GraphType, StateId, StateType},
  proxy::OrderedSet,
  types::{ItemIndex, ItemSet, PrecedentSymbol, SharedGrammarDatabase},
  DBNonTermKey,
  IString,
  Item,
  ParserClassification,
};
use std::{
  self,
  fmt::Debug,
  hash::Hash,
  sync::{atomic::AtomicBool, Arc},
};

use super::ScannerData;

#[derive(Clone, Copy)]
pub(crate) struct RootData {
  pub db_key:    DBNonTermKey,
  pub root_name: IString,
  pub version:   i16,
  pub is_root:   bool,
  pub root_hash: u64,
}

impl Hash for RootData {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.version.hash(state);
    self.root_hash.hash(state);
  }
}

pub struct GraphNode {
  pub(crate) id:           StateId,
  pub(crate) class:        ParserClassification,
  pub(crate) ty:           StateType,
  pub(crate) sym:          PrecedentSymbol,
  pub(crate) hash_id:      u64,
  pub(crate) kernel:       OrderedSet<Item>,
  pub(crate) follow_hash:  Option<u64>,
  pub(crate) graph_type:   GraphType,
  pub(crate) reduce_item:  Option<ItemIndex>,
  pub(crate) predecessor:  Option<SharedGraphNode>,
  pub(crate) symbol_set:   Option<Arc<ScannerData>>,
  /// Only present on scanner states.
  pub(crate) scanner_root: Option<Arc<ScannerData>>,
  pub(crate) db:           SharedGrammarDatabase,
  pub(crate) is_leaf:      bool,
  pub(crate) is_goto:      bool,
  pub(crate) root_data:    RootData,
  pub(crate) invalid:      std::sync::atomic::AtomicBool,
}

impl Hash for GraphNode {
  fn hash<H: std::hash::Hasher>(&self, hash: &mut H) {
    hash.write_u64(self.hash_id)
  }
}

impl PartialEq for GraphNode {
  fn eq(&self, other: &Self) -> bool {
    self.hash_id == other.hash_id
  }
}

impl Eq for GraphNode {}

impl Debug for GraphNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let db = &self.db;

    let header = format!("{:?} - {:?} ", self.graph_type, self.id);

    let scanner = self
      .symbol_set
      .as_ref()
      .map(|s| s.symbols.iter().map(|s| db.sym(s.0.tok()).debug_string(db)).collect::<Vec<_>>().join(" | "))
      .unwrap_or_default();

    if self.is_leaf() {
      f.write_fmt(format_args!(
        "
{:=<120}
== LEAF {}
HASH: {:016X}

ty         : {:?}
sym        : {}
reduce on  : {:?}
prime pred : {:?}
items:
{}
{scanner}
{:?}
{:=>120}",
        "",
        header,
        self.hash_id,
        self.ty,
        self.sym.sym().debug_string(db),
        self.reduce_item.map(|f| Into::<Item>::into((f, db.as_ref()))._debug_string_w_db_(db)),
        self.predecessor.as_ref().map(|i| i.id()),
        self.kernel.iter().map(|i| i._debug_string_w_db_(db)).collect::<Vec<_>>().join("\n"),
        self.get_classification().to_string(),
        ""
      ))
    } else if self.is_root() {
      f.write_fmt(format_args!(
        "
{:#>120}
## {} [{}] {}
HASH: {:016X}
items:
{}
{scanner}
{:?}
{:#>120}",
        "",
        if self.invalid.load(std::sync::atomic::Ordering::Relaxed) { "!!POISONED!! ROOT" } else { "ROOT" },
        db.nonterm_friendly_name_string(self.root_data.db_key),
        header,
        self.hash_id,
        self.kernel.iter().map(|i| i._debug_string_w_db_(db)).collect::<Vec<_>>().join("\n"),
        self.get_classification().to_string(),
        ""
      ))
    } else {
      f.write_fmt(format_args!(
        "
{:-<120}
-- {} {}
HASH: {:016X}
ty         : {:?}
sym        : {}
prime pred : {:?}
items:
{}
{scanner}
{:?}
{:->120}",
        "",
        if self.get_root().invalid.load(std::sync::atomic::Ordering::Relaxed) { "!!POISONED!! INTR" } else { "INTR" },
        header,
        self.hash_id,
        self.ty,
        self.sym.sym().debug_string(db),
        self.predecessor.as_ref().map(|i| i.id()),
        self.kernel.iter().map(|i| i._debug_string_w_db_(db)).collect::<Vec<_>>().join("\n"),
        self.get_classification().to_string(),
        ""
      ))
    }
  }
}

pub type SharedGraphNode = std::sync::Arc<GraphNode>;

impl GraphNode {
  pub fn get_root(&self) -> &GraphNode {
    if self.is_root() {
      self
    } else {
      self.predecessor.as_ref().unwrap().get_root()
    }
  }

  pub fn get_root_shared(self: &SharedGraphNode) -> SharedGraphNode {
    if self.is_root() {
      self.clone()
    } else {
      self.predecessor.as_ref().map(|s| s.get_root_shared()).expect("Graph node should either be a root or have a predecessor")
    }
  }

  pub fn is_goto(&self) -> bool {
    self.is_goto
  }

  pub fn to_goto(&self) -> SharedGraphNode {
    Arc::new(Self {
      is_goto:      true,
      class:        self.class,
      db:           self.db.clone(),
      graph_type:   self.graph_type,
      hash_id:      self.hash_id,
      id:           self.id,
      invalid:      AtomicBool::new(false),
      is_leaf:      self.is_leaf,
      kernel:       self.kernel.clone(),
      predecessor:  None,
      reduce_item:  self.reduce_item.clone(),
      root_data:    self.root_data,
      sym:          self.sym,
      symbol_set:   self.symbol_set.clone(),
      ty:           self.ty,
      scanner_root: Default::default(),
      follow_hash:  Default::default(),
    })
  }

  pub fn goal_items(&self) -> &ItemSet {
    self.get_root().kernel_items()
  }

  pub fn item_is_goal(&self, item: Item) -> bool {
    let root = self.get_root();
    root.kernel_items().iter().any(|i| i.to_canonical().to_complete() == item.to_canonical())
  }

  pub fn get_classification(&self) -> ParserClassification {
    self.class
  }

  pub fn is_root(&self) -> bool {
    self.root_data.is_root
  }

  pub fn kernel_items(&self) -> &OrderedSet<Item> {
    &self.kernel
  }

  pub fn id(&self) -> StateId {
    self.id
  }

  pub fn is_leaf(&self) -> bool {
    self.is_leaf
  }

  pub fn graph_type(&self) -> GraphType {
    self.graph_type
  }

  pub fn state_type(&self) -> StateType {
    self.ty
  }

  pub fn is_scanner(&self) -> bool {
    self.graph_type == GraphType::Scanner
  }

  pub fn get_predecessor<'a>(self: &'a GraphNode, id: StateId) -> Option<&'a GraphNode> {
    if id == self.id {
      Some(self)
    } else if let Some(pred) = self.predecessor.as_ref() {
      pred.get_predecessor(id)
    } else {
      None
    }
  }

  pub fn parent<'a>(&'a self) -> Option<&'a SharedGraphNode> {
    self.predecessor.as_ref()
  }
}
