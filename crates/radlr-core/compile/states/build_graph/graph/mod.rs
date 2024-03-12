use super::{
  flow::get_kernel_items_from_peek_item,
  items::{get_follow_internal, FollowType},
};
use crate::{hash_id_value_u64, proxy::OrderedSet, types::*, Item};
use core::panic;
pub use node::*;
pub use scanner::*;
use std::{
  collections::{BTreeMap, HashMap, HashSet, VecDeque},
  fmt::Debug,
  hash::{Hash, Hasher},
  sync::Arc,
  vec,
};

#[derive(Hash, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PeekGroup {
  pub items:  ItemSet,
  pub is_oos: bool,
}

// STATE ID -------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum GraphIdSubType {
  Root    = 0,
  Regular,
  Goto,
  PostReduce,
  ExtendedClosure,
  ExtendSled,
  Invalid = 0xF,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateId(pub usize, pub GraphIdSubType);

impl Debug for StateId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut t = f.debug_tuple("StateId");
    t.field(&self.index());
    t.field(&self.subtype());
    t.finish()
  }
}

impl Hash for StateId {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.index().hash(state)
  }
}

impl Default for StateId {
  fn default() -> Self {
    Self(usize::MAX, GraphIdSubType::Invalid)
  }
}

impl StateId {
  pub fn new(index: usize, sub_type: GraphIdSubType) -> Self {
    Self(index, sub_type)
  }

  pub fn index(&self) -> usize {
    (self.0 & 0x0F_FF_FF_FF) as usize
  }

  pub fn subtype(&self) -> GraphIdSubType {
    self.1
  }

  /// Indicates the state was generated by transitioning from a goal state to
  /// items that follow from non-terms of goal items.
  pub fn is_oos(&self) -> bool {
    match self.subtype() {
      GraphIdSubType::ExtendSled | GraphIdSubType::ExtendedClosure => true,
      _ => false,
    }
  }

  /// A single use subtype to represent the transition from a an
  /// in-scope state to an out-of-scope state, item, or closure.
  pub fn is_oos_entry(&self) -> bool {
    self.subtype() == GraphIdSubType::ExtendSled
  }

  pub fn is_oos_closure(&self) -> bool {
    self.subtype() == GraphIdSubType::ExtendedClosure
  }

  pub fn extended_entry_base() -> Self {
    Self::new(0, GraphIdSubType::ExtendSled)
  }

  pub fn root() -> Self {
    Self::new(0, GraphIdSubType::Root)
  }

  pub fn is_invalid(&self) -> bool {
    self.subtype() == GraphIdSubType::Invalid
  }

  pub fn is_root(&self) -> bool {
    self.subtype() == GraphIdSubType::Root || self.index() == 0
  }

  pub fn is_rootish(&self) -> bool {
    self.subtype() == GraphIdSubType::Root
  }

  pub fn is_post_reduce(&self) -> bool {
    self.subtype() == GraphIdSubType::PostReduce
  }

  pub fn is_goto(&self) -> bool {
    self.subtype() == GraphIdSubType::Goto
  }

  pub fn to_post_reduce(&self) -> Self {
    Self::new(self.index(), GraphIdSubType::PostReduce)
  }

  pub fn to_goto(&self) -> Self {
    Self::new(self.index(), GraphIdSubType::Goto)
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Default)]
pub enum GraphType {
  /// Classic Recursive Descent Ascent with unlimited lookahead.
  #[default]
  Parser,
  // Scanner mode for creating tokens, analogous to regular expressions.
  Scanner,
}

/// Indicates the State type that generated
/// the item
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[allow(non_camel_case_types)]
pub enum Origin {
  None,
  /// The goal non-terminal that this item or it's predecessors will reduce to
  NonTermGoal(DBNonTermKey),
  /// The goal symbol id that this item or its predecessors will recognize
  TerminalGoal(DBTermKey, u16),
  /// The hash and state of the goal items set the peek item will resolve to
  Peek(u32),
  Fork(DBRuleKey),
  PEG(DBNonTermKey),
  Closure(StateId),
  Goto(StateId),
  __OOS_CLOSURE__,
  __OOS_ROOT__,
  __OOS_SCANNER_ROOT__(PrecedentDBTerm),
  /// Generated when the a goal non-terminal is completed.
  /// Goal non-terminals are determined by the
  /// root state (`StateId(0)`) kernel items
  GoalCompleteOOS,
}

impl Hash for Origin {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Origin::Peek(resolve_id) => resolve_id.hash(state),
      Origin::TerminalGoal(resolve_id, prec) => {
        resolve_id.hash(state);
        prec.hash(state);
      }
      Origin::Fork(resolve_id) => {
        resolve_id.hash(state);
      }
      _ => {}
    }

    std::mem::discriminant(self).hash(state)
  }
}

impl Default for Origin {
  fn default() -> Self {
    Self::None
  }
}

impl Origin {
  #[cfg(debug_assertions)]
  pub fn _debug_string_(&self) -> String {
    match self {
      Origin::NonTermGoal(nterm) => {
        format!("NonTermGoal[ {:?} ]", nterm)
      }
      Origin::TerminalGoal(sym_id, prec) => {
        format!("TerminalGoal[ {:?} {prec} ]", sym_id)
      }
      _ => format!("{:?}", self),
    }
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Origin::None)
  }

  pub fn is_out_of_scope(&self) -> bool {
    matches!(self, Origin::GoalCompleteOOS | Origin::__OOS_CLOSURE__ | Origin::__OOS_ROOT__ | Origin::__OOS_SCANNER_ROOT__(..))
  }

  pub fn is_scanner_oos(&self) -> bool {
    matches!(self, Origin::__OOS_SCANNER_ROOT__(..))
  }

  pub fn get_symbol(&self, db: &ParserDatabase) -> SymbolId {
    match self {
      Origin::TerminalGoal(sym_id, ..) => db.sym(*sym_id),
      _ => SymbolId::Undefined,
    }
  }

  pub fn get_symbol_key(&self) -> DBTermKey {
    match self {
      Origin::TerminalGoal(sym_id, ..) => *sym_id,
      _ => DBTermKey::default(),
    }
  }
}

// Transition Type ----------------------------------------------------

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub enum StateType {
  Undefined,
  Start,
  Shift,
  KernelShift,
  /// Contains kernel items for an out of scope state tha is used to
  /// disambiguate completed items. This states are internal only and do not
  /// appear in finalized graphs.
  _OosClosure_,
  /// The completion of this branch will complete one or more intermediary goto
  /// items.
  NonTerminalShiftLoop,
  NonTerminalComplete,
  ForkInitiator,
  ForkedState,
  Peek(u32),
  /// A peek path has been resolved to a single peek group located in the peek
  /// origin state.
  PeekEndComplete(u32),
  CompleteToken,
  Follow,
  AssignAndFollow(DBTermKey),
  Reduce(DBRuleKey, usize),
  /// Assigns the token id of a terminal symbol. This is always a leaf statement
  /// for scanner graphs.
  AssignToken(DBTermKey),
  /// Accept the current Nonterminal within the CST
  CSTNodeAccept(DBNonTermKey),
  /// Calls made on items within a state's closure but
  /// are not kernel items.
  InternalCall(DBNonTermKey),
  /// Calls made on kernel items
  KernelCall(DBNonTermKey),
  /// A shift that pushes pushes a non-terminal shift onto the stack before
  /// jumping to a terminal shift shifts into a terminal state
  ShiftFrom(StateId),
  /// Creates a leaf state that has a single `pop` instruction. This represents
  /// the completion of a non-terminal that had a left recursive rule.
  NonTermCompleteOOS,
  /// Creates a leaf state that has a single `pop` instruction,
  /// with the intent of removing a goto floor state.
  _PeekNonTerminalCompleteOOS,
  /// Creates a leaf state that has a single `pass` instruction.
  ScannerCompleteOOS,
  _FirstMatch,
  _LongestMatch,
  _ShortestMatch,
}

impl Default for StateType {
  fn default() -> Self {
    Self::Undefined
  }
}

impl StateType {
  pub fn currently_peeking(&self) -> bool {
    match self {
      StateType::Peek(_) => true,
      _ => false,
    }
  }

  pub fn is_peek(&self) -> bool {
    matches!(self, StateType::Peek(_))
  }

  pub fn peek_level(&self) -> u32 {
    match self {
      StateType::Peek(level) => *level,
      _ => 0,
    }
  }
}

mod node;
mod scanner;

fn get_follow_symbol_data<'a>(
  builder: &mut ConcurrentGraphBuilder,
  node: &'a GraphNode,
  item: Item,
  db: &'a ParserDatabase,
) -> impl Iterator<Item = (Item, PrecedentDBTerm)> + 'a {
  let mode = GraphType::Parser;
  match item.get_type(db) {
    ItemType::Completed(_) => get_follow_internal(builder, node, item, FollowType::AllItems)
      .0
      .into_iter()
      .flat_map(move |i| {
        if let Some(sym) = i.precedent_db_key_at_sym(mode, db) {
          vec![(i.to_origin(item.origin), sym)]
        } else if i.is_nonterm(mode, db) {
          db.get_closure(&i)
            .filter_map(|i| {
              if let Some(sym) = i.precedent_db_key_at_sym(mode, db) {
                Some((i.to_origin(item.origin), sym))
              } else {
                None
              }
            })
            .collect::<Vec<_>>()
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>()
      .into_iter(),
    ItemType::NonTerminal(_) => db
      .get_closure(&item)
      .filter_map(|item| if let Some(sym) = item.precedent_db_key_at_sym(mode, db) { Some((item, sym)) } else { None })
      .collect::<Vec<_>>()
      .into_iter(),
    ItemType::TokenNonTerminal(..) | ItemType::Terminal(..) => {
      vec![(item, item.precedent_db_key_at_sym(mode, db).unwrap())].into_iter()
    }
  }
}

fn get_state_symbols<'a>(builder: &mut ConcurrentGraphBuilder, node: &GraphNode) -> Option<ScannerData> {
  let mode = node.graph_type();

  debug_assert_eq!(mode, GraphType::Parser);

  let db = builder.db_rc();
  let db = &db;

  let mut skipped = OrderedSet::new();

  let mut scanner_data = ScannerData { hash: 0, ..Default::default() };

  for item in node.kernel_items().clone() {
    skipped.extend(item.get_skipped(db));

    if let Some(sym) = item.precedent_db_key_at_sym(mode, db) {
      let follow_syms = get_follow_symbol_data(builder, node, item.increment().unwrap(), db)
        .into_iter()
        .map(|(_, sym)| sym)
        .collect::<OrderedSet<_>>();

      scanner_data.symbols.entry(sym).or_default().extend(follow_syms);
    } else if item.is_nonterm(mode, db) {
      for i in db.get_closure(&item) {
        let i = i.to_origin(item.origin).to_origin_state(item.origin_state);
        if let Some(sym) = i.precedent_db_key_at_sym(mode, db) {
          skipped.extend(i.get_skipped(db));
          let follow_syms = get_follow_symbol_data(builder, node, i.increment().unwrap(), db)
            .into_iter()
            .map(|(_, sym)| sym)
            .collect::<OrderedSet<_>>();

          scanner_data.symbols.entry(sym).or_default().extend(follow_syms);
        }
      }
    } else {
      for (i, sym) in get_follow_symbol_data(builder, node, item.to_complete(), db) {
        let i = i.to_origin(item.origin).to_origin_state(item.origin_state);
        let follow_syms = get_follow_symbol_data(builder, node, i.increment().unwrap(), db)
          .into_iter()
          .map(|(_, sym)| sym)
          .collect::<OrderedSet<_>>();
        skipped.extend(i.get_skipped(db));
        scanner_data.symbols.entry(sym).or_default().extend(follow_syms);
      }
    }
  }

  let syms = scanner_data.symbols.iter().map(|s| s.0.tok()).collect::<OrderedSet<_>>();

  let is_uncontested_reduce_state = node.kernel.len() == 1 && node.kernel.first().unwrap().is_complete();

  if !is_uncontested_reduce_state {
    let skipped_candidates = skipped.into_iter().flat_map(|s| {
      s.iter().filter_map(|s| {
        let id = s.tok_db_key().unwrap();
        (!syms.contains(&id)).then_some(id)
      })
    });

    scanner_data.skipped.extend(skipped_candidates)
  }

  if scanner_data.symbols.is_empty() {
    None
  } else {
    let hash_symbols = scanner_data.symbols.clone();
    //hash_symbols.extend(scanner_data.follow.iter());
    let hash = hash_id_value_u64((&scanner_data.skipped, hash_symbols));
    scanner_data.hash = hash;

    Some(scanner_data)
  }
}

fn create_lookahead_hash<'a, H: std::hash::Hasher>(builder: &mut ConcurrentGraphBuilder, node: &GraphNode, mut hasher: H) -> u64 {
  match node.graph_type {
    GraphType::Parser => {
      let mode = GraphType::Parser;

      let mut symbols = OrderedSet::new();
      for item in node.kernel_items() {
        {
          let (follow, _) = get_follow_internal(builder, node, item.to_complete(), FollowType::AllItems);
          for item in follow {
            if let Some(term) = item.term_index_at_sym(mode, builder.db()) {
              symbols.insert(term);
            } else if item.is_nonterm(mode, builder.db()) {
              for item in builder.db().get_closure(&item) {
                if let Some(term) = item.term_index_at_sym(node.graph_type, builder.db()) {
                  symbols.insert(term);
                }
              }
            }
          }
        }
      }

      symbols.hash(&mut hasher);

      hasher.finish()
    }
    GraphType::Scanner => hasher.finish(),
  }
}

fn create_state_hash<'a, H: std::hash::Hasher>(state: &GraphNode, lookahead: u64, mut hasher: H) -> u64 {
  let hasher = &mut hasher;

  state.root_data.hash(hasher);

  match state.ty {
    StateType::Peek(_) => "peek".hash(hasher),
    _ => state.ty.hash(hasher),
  };

  state.sym.hash(hasher);

  for item in state.kernel_items() {
    item.index().hash(hasher);
    item.from.hash(hasher);

    item.origin.hash(hasher);

    if !state.is_scanner() {
      item.from_goto_origin.hash(hasher);
      item.goto_distance.hash(hasher);
    }
  }

  state.follow_hash.hash(hasher);

  if let Some(reduce_item) = &state.reduce_item {
    reduce_item.hash(hasher)
  }

  lookahead.hash(hasher);

  hasher.finish()
}

pub enum PostNodeConstructorData {
  None,
}

pub type PostNodeConstructor =
  Box<dyn FnOnce(&SharedGraphNode, &mut ConcurrentGraphBuilder, PostNodeConstructorData) -> Vec<StagedNode>>;

pub type Finalizer = Box<dyn FnOnce(&mut GraphNode, &mut ConcurrentGraphBuilder, bool)>;

/// Temporary Represention of a graph node before goto transformations are
/// applied
pub struct StagedNode {
  node:                    GraphNode,
  /// Post processor that finalizes the configuration of this node, right before
  /// it is converted into a read-only node
  pnc_constructor:         Option<PostNodeConstructor>,
  pnc_data:                PostNodeConstructorData,
  finalizer:               Option<Finalizer>,
  enqueued_leaf:           bool,
  include_with_goto_state: bool,
}

impl Debug for StagedNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("PreStagedNode");
    s.field("node", &self.node);
    s.finish()
  }
}

impl AsRef<GraphNode> for StagedNode {
  fn as_ref(&self) -> &GraphNode {
    &self.node
  }
}

impl StagedNode {
  pub fn new(gb: &ConcurrentGraphBuilder) -> Self {
    Self {
      node:                    GraphNode {
        graph_type:   GraphType::Parser,
        hash_id:      0,
        id:           StateId::default(),
        is_leaf:      false,
        kernel:       Default::default(),
        scanner_root: Default::default(),
        // lookahead_id: 0,
        predecessor:  None,
        reduce_item:  None,
        follow_hash:  Default::default(),
        sym:          PrecedentSymbol::default(),
        ty:           StateType::Undefined,
        symbol_set:   None,
        db:           gb.db_rc().clone(),
        is_goto:      false,
        invalid:      Default::default(),
        class:        Default::default(),
        root_data:    RootData {
          db_key:    DBNonTermKey::default(),
          is_root:   false,
          root_name: Default::default(),
          version:   -1,
        },
      },
      include_with_goto_state: false,
      pnc_constructor:         None,
      finalizer:               None,
      pnc_data:                PostNodeConstructorData::None,
      enqueued_leaf:           false,
    }
  }

  pub fn set_reduce_item(mut self, item: Item) -> Self {
    self.node.reduce_item = Some(item.index());
    self
  }

  pub fn include_with_goto_state(mut self) -> Self {
    self.include_with_goto_state = true;
    self
  }

  pub fn to_classification(mut self, class: ParserClassification) -> Self {
    self.node.class |= class;
    self
  }

  pub fn add_scanner_root(mut self, scanner: Arc<ScannerData>) -> Self {
    self.node.scanner_root = Some(scanner);
    self
  }

  pub fn add_kernel_items<T: ItemContainerIter>(mut self, items: T) -> Self {
    self.node.kernel.extend(items);
    self
  }

  pub fn kernel_items(mut self, items: impl Iterator<Item = Item>) -> Self {
    self.node.kernel = OrderedSet::from_iter(items);
    self
  }

  pub fn set_follow_hash(mut self, hash: u64) -> Self {
    self.node.follow_hash = Some(hash);
    self
  }

  pub fn parent(mut self, parent: SharedGraphNode) -> Self {
    self.node.graph_type = parent.graph_type;
    self.node.predecessor = Some(parent);
    self
  }

  pub fn sym(mut self, sym: PrecedentSymbol) -> Self {
    self.node.sym = sym;
    self
  }

  pub fn make_enqueued_leaf(mut self) -> Self {
    self.node.is_leaf = true;
    self.enqueued_leaf = true;
    self
  }

  pub fn make_leaf(mut self) -> Self {
    self.node.is_leaf = true;
    self
  }

  pub fn make_root(mut self, root_name: IString, db_key: DBNonTermKey, version: i16) -> Self {
    self.node.root_data = RootData { db_key, is_root: true, root_name, version: version.min(i16::MAX) };
    self
  }

  pub fn graph_ty(mut self, ty: GraphType) -> Self {
    self.node.graph_type = ty;
    self
  }

  pub fn ty(mut self, ty: StateType) -> Self {
    self.node.ty = ty;
    self
  }

  fn id(mut self, id: StateId) -> Self {
    self.node.id = id;
    self
  }

  pub fn pnc(mut self, pnc: PostNodeConstructor, pnc_data: PostNodeConstructorData) -> Self {
    debug_assert!(self.pnc_constructor.is_none(), "Expected finalizer to be None: This should only be set once");
    self.pnc_constructor = Some(pnc);
    self.pnc_data = pnc_data;
    self
  }

  pub fn commit(self, builder: &mut ConcurrentGraphBuilder) {
    if self.node.is_root() {
      builder.pre_stage.push(self);
    } else {
      debug_assert!(self.node.predecessor.is_some(), "Nodes that are not root should have at least one predecessor");
      builder.pre_stage.push(self);
    }
  }
}

type SharedRW<T> = std::sync::Arc<std::sync::RwLock<T>>;

type RootStateData = (GraphType, SharedGraphNode, ParserConfig);

type RootStates = Map<u64, RootStateData>;

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ConcurrentGraphBuilder {
  queue: SharedRW<VecDeque<(SharedGraphNode, ParserConfig)>>,
  root_states: SharedRW<RootStates>,
  pub(crate) state_nonterms: SharedRW<Map<u64, ItemSet>>,
  peek_resolves: SharedRW<Map<u64, PeekGroup>>,
  pub(crate) graph: SharedRW<Map<SharedGraphNode, HashSet<SharedGraphNode>>>,
  produced_nodes: SharedRW<Set<u64>>,
  /// The next node that can be processed without creating a cross thread sync.
  local_next: Option<(SharedGraphNode, ParserConfig)>,
  symbol_sets: Map<u64, Arc<ScannerData>>,
  oos_roots: Map<DBNonTermKey, SharedGraphNode>,
  oos_closures: Map<Item, SharedGraphNode>,
  state_lookups: Map<u64, SharedGraphNode>,
  db: SharedParserDatabase,
  pre_stage: Vec<StagedNode>,
  /// A Peek node has entered a recursive loop
  recursive_peek_error: bool,
}

unsafe impl Send for ConcurrentGraphBuilder {}
unsafe impl Sync for ConcurrentGraphBuilder {}

impl Clone for ConcurrentGraphBuilder {
  fn clone(&self) -> Self {
    Self {
      queue:                self.queue.clone(),
      root_states:          self.root_states.clone(),
      state_nonterms:       self.state_nonterms.clone(),
      peek_resolves:        self.peek_resolves.clone(),
      local_next:           self.local_next.clone(),
      db:                   self.db.clone(),
      graph:                self.graph.clone(),
      produced_nodes:       self.produced_nodes.clone(),
      symbol_sets:          self.symbol_sets.clone(),
      oos_closures:         Default::default(),
      oos_roots:            Default::default(),
      state_lookups:        Default::default(),
      pre_stage:            Default::default(),
      recursive_peek_error: false,
    }
  }
}

impl ConcurrentGraphBuilder {
  pub fn new(db: SharedParserDatabase) -> Self {
    ConcurrentGraphBuilder {
      db,
      graph: Default::default(),
      queue: Default::default(),
      root_states: Default::default(),
      state_nonterms: Default::default(),
      peek_resolves: Default::default(),
      local_next: Default::default(),
      symbol_sets: Default::default(),
      produced_nodes: Default::default(),
      oos_roots: Default::default(),
      oos_closures: Default::default(),
      state_lookups: Default::default(),
      pre_stage: Default::default(),
      recursive_peek_error: false,
    }
  }

  pub fn set_peek_resolve_state<T: Iterator<Item = Item>>(&mut self, items: T, is_oos: bool) -> Origin {
    let peek_group = PeekGroup { items: items.collect(), is_oos };

    let index = hash_id_value_u64(&peek_group) as u32;

    match self.peek_resolves.write() {
      Ok(mut peek_resolve_states) => {
        peek_resolve_states.insert(index as u64, peek_group);
      }
      Err(err) => panic!("{err}"),
    }

    Origin::Peek(index)
  }

  pub fn invalidate_nonterms(&mut self, invalidate_nonterms: &[DBNonTermKey], graph_version: i16) -> RadlrResult<()> {
    let set = invalidate_nonterms.iter().cloned().collect::<OrderedSet<_>>();
    match self.root_states.write() {
      Ok(nt) => {
        for (_, (_, node, _)) in nt.iter() {
          if set.contains(&node.root_data.db_key) && node.root_data.version == graph_version {
            node.invalid.store(true, std::sync::atomic::Ordering::Release);
          }
        }

        Ok(())
      }
      Err(err) => Err(err.into()),
    }
  }

  pub fn set_nonterm_items(&mut self, id: u64, nonterms: ItemSet) {
    match self.state_nonterms.write() {
      Ok(mut nt) => {
        nt.insert(id, nonterms);
      }
      Err(_) => panic!("queue has been poisoned"),
    }
  }

  pub fn declare_recursive_peek_error(&mut self) {
    self.recursive_peek_error = true;
  }

  pub fn get_goto_pending_items(&self) -> ItemSet {
    let mut items: std::collections::BTreeSet<Item> = ItemSet::default();

    for item in self.pre_stage.iter().filter_map(|i| {
      if i.include_with_goto_state {
        if i.node.ty.is_peek() {
          let mut items = ItemSet::new();
          for item in i.node.kernel_items() {
            items.extend(get_kernel_items_from_peek_item(self, item).items);
          }
          Some(items)
        } else {
          Some(i.node.kernel.clone())
        }
      } else {
        None
      }
    }) {
      items.extend(item)
    }

    items
  }

  pub fn get_peek_resolve_items(&self, id: u64) -> PeekGroup {
    match self.peek_resolves.read() {
      Ok(peek) => peek.get(&id).cloned().unwrap(),
      Err(_) => panic!("queue has been poisoned"),
    }
  }

  fn enqueue_state_for_processing_kernel(&mut self, state: SharedGraphNode, parser_config: ParserConfig, allow_local: bool) {
    if self.local_next.is_none() && allow_local {
      {
        self.local_next = Some((state, parser_config));
      }
    } else {
      match self.queue.write() {
        Ok(mut queue) => {
          queue.push_back((state, parser_config));
        }
        Err(_) => panic!("queue has been poisoned"),
      }
    }
  }

  pub fn get_local_work(&mut self) -> Option<(SharedGraphNode, ParserConfig)> {
    self.recursive_peek_error = false;
    match self.local_next.take() {
      Some(work) => {
        if work.0.get_root().invalid.load(std::sync::atomic::Ordering::Acquire) {
          None
        } else {
          Some(work)
        }
      }
      _ => None,
    }
  }

  pub fn get_global_work(&mut self) -> Option<(SharedGraphNode, ParserConfig)> {
    self.recursive_peek_error = false;
    match self.queue.write() {
      Ok(mut queue) => {
        return queue.pop_front();
      }
      Err(_) => panic!("queue has been poisoned"),
    }
  }

  pub fn db(&self) -> &ParserDatabase {
    &self.db
  }

  pub fn db_rc(&self) -> SharedParserDatabase {
    self.db.clone()
  }

  pub fn get_state(&self, state: u64) -> Option<SharedGraphNode> {
    self.state_lookups.get(&state).cloned()
  }

  /// Drops all nodes that have yet to be committed to the graph.
  pub fn drop_uncommitted(&mut self) {
    self.pre_stage.clear()
  }

  pub fn get_oos_scanner_follow(&self, node: &GraphNode, terms: &OrderedSet<PrecedentDBTerm>) -> ItemSet {
    if let Some(scanner) = node.scanner_root.as_ref() {
      let mut items = ItemSet::new();
      let db = self.db();
      for precendent_term in terms {
        if let Some(follow_syms) = scanner.symbols.get(&precendent_term) {
          items.extend(
            follow_syms
              .iter()
              .map(|s| {
                ItemSet::start_items(db.token(s.tok()).nonterm_id, db)
                  .to_origin_state(StateId::default())
                  .to_origin(Origin::__OOS_SCANNER_ROOT__(*precendent_term))
              })
              .flatten()
              .collect::<Items>(),
          );
        }
      }
      items
    } else {
      Default::default()
    }
  }

  /// Creates or returns a state whose kernel items is the FOLLOW closure of the
  /// givin non-terminal, that is all items that are `_  = b A • b` for some
  /// non-terminal `A`
  pub fn get_oos_root_state(&mut self, nterm: DBNonTermKey) -> SharedGraphNode {
    if let Some(state_id) = self.oos_roots.get(&nterm) {
      state_id.clone()
    } else {
      let id = StateId::new(self.oos_roots.len(), GraphIdSubType::ExtendedClosure);

      let item_id = StateId::new(0, GraphIdSubType::ExtendedClosure);

      let closure = self
        .db()
        .nonterm_follow_items(nterm)
        .map(|i| i.to_origin(Origin::__OOS_CLOSURE__).to_origin_state(item_id))
        .filter_map(|i| i.increment());

      let pending = StagedNode::new(&self).id(id).ty(StateType::_OosClosure_).kernel_items(closure).sym(Default::default());

      let mut state = pending.node;

      state.hash_id = hash_id_value_u64(nterm);

      let state = Arc::new(state);

      self.state_lookups.insert(id.0 as u64, state.clone());
      self.oos_roots.insert(nterm, state);
      self.get_oos_root_state(nterm)
    }
  }

  pub fn get_oos_closure_state(&mut self, item: Item) -> SharedGraphNode {
    debug_assert!(item.origin_state.is_oos());

    let state = item.origin_state;

    let item = item.to_canonical().to_origin_state(state);

    if let Some(node) = self.oos_closures.get(&item) {
      node.clone()
    } else {
      let id = StateId::new(self.oos_closures.len(), GraphIdSubType::ExtendedClosure);

      let kernel = item.to_origin_state(id).to_origin(Origin::__OOS_CLOSURE__);

      let closure = kernel.closure_iter_align(kernel, self.db());

      let origin = item.origin_state.0 as u64;

      let pending = StagedNode::new(&self)
        .id(id)
        .ty(StateType::_OosClosure_)
        .kernel_items(closure)
        .parent(self.get_state(origin).unwrap())
        .sym(Default::default());

      let mut state = pending.node;

      state.hash_id = hash_id_value_u64(item);

      let state = Arc::new(state);

      self.state_lookups.insert(id.0 as u64, state.clone());
      self.oos_closures.insert(item, state);
      self.get_oos_closure_state(item)
    }
  }

  pub fn commit(
    &mut self,
    increment_goto: bool,
    pred: Option<&SharedGraphNode>,
    parser_config: &ParserConfig,
    allow_local_queueing: bool,
  ) -> RadlrResult<u32> {
    let mut nodes = self.pre_stage.drain(..).collect::<VecDeque<_>>();
    let mut queued = 0;
    let pred_id = pred.map(|d| d.id).unwrap_or_default().0;

    // Ensure we are still working on a valid graph.
    if let Some(pred) = pred {
      if pred.get_root().invalid.load(std::sync::atomic::Ordering::Acquire) {
        return Ok(u32::MAX);
      }
    }

    let mut child_states: HashMap<Arc<GraphNode>, Vec<_>> = HashMap::new();
    let mut output_states = Vec::new();

    while let Some(StagedNode {
      node,
      pnc_constructor,
      pnc_data,
      finalizer,
      enqueued_leaf,
      include_with_goto_state: allow_goto_increment,
    }) = nodes.pop_front()
    {
      let mut state = if increment_goto && allow_goto_increment && !node.ty.is_peek() {
        GraphNode {
          kernel: node
            .kernel
            .iter()
            .map(|i| if i.origin_state.0 == pred_id { i.as_goto_origin() } else { i.increment_goto() })
            .collect::<_>(),
          ..node
        }
      } else {
        node
      };

      // Insert scanner lookahead items if allowed by config
      if state.is_scanner() && parser_config.ALLOW_LOOKAHEAD_SCANNERS {
        if let Some(pred) = pred {
          let kernel_items = &mut state.kernel;
          let mut completed_symbols = OrderedSet::new();
          for item in kernel_items.iter() {
            if item.is_complete() {
              if let Origin::TerminalGoal(t, p) = item.origin {
                let term: PrecedentDBTerm = (t, p, false).into();
                completed_symbols.insert(term);
              }
            }
          }

          if !completed_symbols.is_empty() {
            kernel_items.extend(self.get_oos_scanner_follow(pred, &completed_symbols));
          }
        }
      }

      if let Some(finalizer) = finalizer {
        finalizer(&mut state, self, increment_goto);
      }

      let is_root = update_root_info(&mut state, pred);

      state = self.append_state_hashes(is_root, state);

      if !state.is_scanner() {
        if let Some(scanner_data) = get_state_symbols(self, &state) {
          let scanner_data = Arc::new(scanner_data);
          let sym_set_id = scanner_data.hash;

          if !self.symbol_sets.contains_key(&sym_set_id) {
            // Build a scanner entry
            let db = self.db();

            let start_items = scanner_data
              .symbols
              .iter()
              .map(|s| {
                ItemSet::start_items(db.token(s.0.tok()).nonterm_id, db)
                  .to_origin_state(StateId::default())
                  .to_origin(Origin::TerminalGoal(s.0.tok(), s.0.precedence()))
              })
              .chain(scanner_data.skipped.iter().map(|s| {
                ItemSet::start_items(db.token(*s).nonterm_id, db)
                  .to_origin_state(StateId::default())
                  .to_origin(Origin::TerminalGoal(*s, 0))
              }))
              .flatten();

            let scanner_root = StagedNode::new(self)
              .kernel_items(start_items)
              .ty(StateType::Start)
              .graph_ty(GraphType::Scanner)
              .add_scanner_root(scanner_data.clone())
              .make_root(scanner_data.create_scanner_name(db), DBNonTermKey::default(), 0)
              .id(StateId(sym_set_id as usize, GraphIdSubType::Root));

            nodes.push_back(scanner_root);

            self.symbol_sets.insert(sym_set_id, scanner_data);
          }

          state.symbol_set = self.symbol_sets.get(&sym_set_id).cloned();
        }
      }

      let state = Arc::new(state);

      if let Some(pnc) = pnc_constructor {
        nodes.extend(pnc(&state, self, pnc_data));
      }

      if state.ty == StateType::Start {
        match self.root_states.write() {
          Ok(mut root_state) => {
            root_state.insert(state.hash_id, (GraphType::Parser, state.clone(), *parser_config));
            output_states.push((state, false));
          }
          Err(err) => return Err(err.into()),
        }
      } else {
        let par = state.parent().cloned().unwrap();
        child_states.entry(par).or_default().push((state, enqueued_leaf));
      }
    }

    if !(child_states.is_empty() && output_states.is_empty()) {
      match self.graph.clone().write() {
        Ok(mut graph) => {
          for (pred, child_states) in child_states {
            let successor_nodes = graph.entry(pred).or_default();

            for (child_node, enqueued_leaf) in &child_states {
              if successor_nodes.insert(child_node.clone()) {
                output_states.push((child_node.clone(), *enqueued_leaf))
              }
            }
          }

          match self.produced_nodes.clone().write() {
            Ok(mut produced_nodes) => {
              for (state, enqueued_leaf) in output_states {
                graph.entry(state.clone()).or_default();
                if produced_nodes.insert(state.hash_id) {
                  if (!state.is_leaf) || enqueued_leaf {
                    self.enqueue_state_for_processing_kernel(state, *parser_config, allow_local_queueing);
                  }
                  queued += 1;
                }
              }
            }
            _ => panic!("Poisoned lock"),
          }
        }
        _ => panic!("Poisoned lock"),
      }
    }

    Ok(queued)
  }

  /// Create hash id's for the given state.
  ///
  /// WARNING: Ensure the state's root_data is set before calling this method.
  fn append_state_hashes(&mut self, is_root: bool, mut state: GraphNode) -> GraphNode {
    let lookahead =
      if is_root { 0 } else { create_lookahead_hash(self, &state, std::collections::hash_map::DefaultHasher::new()) };

    let state_hash = create_state_hash(&state, lookahead, std::collections::hash_map::DefaultHasher::new());

    state.hash_id = state_hash;
    state.id = StateId::new(state.hash_id as usize, is_root.then_some(GraphIdSubType::Root).unwrap_or(GraphIdSubType::Regular));
    state.kernel = state
      .kernel
      .into_iter()
      .map(|i| {
        if i.origin_state.is_invalid() {
          debug_assert!(!state.id.is_invalid());
          i.to_origin_state(state.id)
        } else {
          i
        }
      })
      .collect();

    state
  }
}

fn update_root_info(state: &mut GraphNode, pred: Option<&Arc<GraphNode>>) -> bool {
  let is_root = if !state.root_data.is_root {
    match pred {
      Some(pred) => {
        state.root_data = pred.root_data;
        state.root_data.is_root = false;
        state.scanner_root = pred.scanner_root.clone();
        false
      }
      None => {
        panic!("Non-root states should have a predecessor\n Offending state:\n {state:?}")
      }
    }
  } else {
    true
  };
  is_root
}

pub struct Graphs {
  pub root_states:    RootStates,
  pub successors:     Map<Arc<GraphNode>, HashSet<Arc<GraphNode>>>,
  pub state_nonterms: Map<u64, ItemSet>,
}

impl Debug for Graphs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("Graphs");
    s.field("root_states", &self.root_states);
    s.field("successors", &self.successors);
    s.finish()
  }
}

impl From<ConcurrentGraphBuilder> for Graphs {
  fn from(value: ConcurrentGraphBuilder) -> Self {
    debug_assert!(value.queue.read().unwrap().is_empty());
    Self {
      root_states:    value.root_states.read().unwrap().clone(),
      successors:     value.graph.read().unwrap().clone(),
      state_nonterms: value.state_nonterms.read().unwrap().clone(),
    }
  }
}

impl Graphs {
  pub fn create_ir_precursors<'a>(&'a self) -> IrPrecursorData<'a> {
    // Collect all leaf states that are part of a valid source

    let mut precursors = OrderedMap::new();

    for (par, nodes) in &self.successors {
      let key = par.hash_id;

      if par.get_root().invalid.load(std::sync::atomic::Ordering::Relaxed) {
        continue;
      }

      //if !precursors.contains_key(&key) {
      precursors.insert(key, IRPrecursorGroup {
        node:          par.clone(),
        successors:    nodes.iter().map(|n| (n.hash_id, n.clone())).collect(),
        non_terminals: self.state_nonterms.get(&par.hash_id).cloned(),
        root_name:     par.is_root().then(|| par.root_data.root_name),
      });
      // }
    }

    IrPrecursorData { graph: self, precursors }
  }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct IRPrecursorGroup {
  pub node:          SharedGraphNode,
  pub successors:    BTreeMap<u64, SharedGraphNode>,
  pub non_terminals: Option<ItemSet>,
  pub root_name:     Option<IString>,
}

pub struct IrPrecursorData<'a> {
  graph:      &'a Graphs,
  precursors: BTreeMap<u64, IRPrecursorGroup>,
}

#[cfg(debug_assertions)]
impl<'a> Debug for IrPrecursorData<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("IrPrecursorData");
    s.field("precursors", &self.precursors);
    s.finish()
  }
}

impl<'a> IrPrecursorData<'a> {
  pub fn iter<'p>(&'p self) -> GraphIterator<'p> {
    self.into()
  }
}

pub struct GraphIterator<'a> {
  precursors: &'a IrPrecursorData<'a>,
  queue:      VecDeque<u64>,
}

impl<'a: 'b, 'b> From<&'b IrPrecursorData<'a>> for GraphIterator<'b> {
  fn from(value: &'b IrPrecursorData<'a>) -> Self {
    let mut out_queue = VecDeque::new();
    let mut process_queue = VecDeque::from_iter(value.precursors.iter());

    while let Some((id, _)) = process_queue.pop_front() {
      out_queue.push_back(*id);
      /* if seen.insert(id) {
        if let Some(IRPrecursorGroup { successors, .. }) = value.precursors.get(&id).as_ref() {
          for successor in successors.values() {
            process_queue.push_back(successor.hash_id)
          }
        }
      } */
    }

    Self { precursors: value, queue: out_queue }
  }
}

impl<'a> Iterator for GraphIterator<'a> {
  type Item = &'a IRPrecursorGroup;

  fn next(&mut self) -> Option<Self::Item> {
    let next = self.queue.pop_front();
    next.and_then(|n| self.precursors.precursors.get(&n))
  }
}
