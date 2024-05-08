use super::*;
use crate::{compile::states::build_graph::graph::GraphType, parser, CachedString, RadlrResult};
use std::collections::VecDeque;

pub type SharedParserDatabase = std::sync::Arc<ParserDatabase>;

/// The internal data type used for the compilation and analysis of grammars and
/// parsers. contains additional metadata for compilation of LLVM and Bytecode
/// parsers.
#[derive(Clone, Default, Debug)]
pub struct ParserDatabase {
  pub root_grammar_id:  GrammarIdentities,
  /// Table of symbols.
  nonterm_symbols:      Array<SymbolId>,
  /// Maps a non-terminal to all other non-terminals that reference it, directly
  /// or indirectly,  within their rules.
  nonterm_predecessors: OrderedMap<DBNonTermKey, OrderedSet<DBNonTermKey>>,

  /// Maps a non-terminal to all rules that contain the nonterm's symbol in
  /// their bodies.
  nonterm_symbol_to_rules: OrderedMap<DBNonTermKey, OrderedSet<DBRuleKey>>,
  /// Table of non-terminal names for public non-terminal.
  ///
  /// - First tuple member: GUID name,
  /// - Second member: friendly name.
  ///
  /// This is a 1-to-1 mapping of all non-terminal indices, so non-terminals
  /// that are scanner or are sub-non-terminals map to empty strings.
  nonterm_names:           Array<(IString, IString)>,
  /// Table mapping non-terminal indices to rule indices.
  nonterm_nterm_rules:     Array<Array<DBRuleKey>>,
  /// Table of all rules within the grammar and the non-terminal they reduce
  /// to.
  rules:                   Array<DBRule>,
  /// Table of all tokens generated by the scanners.
  tokens:                  Array<DBTokenData>,
  /// The entry point non-terminals of the grammar.
  entry_points:            Array<DBEntryPoint>,
  /// The global string store
  string_store:            IStringStore,
  /// Custom states that should be integrated into the final parsers
  custom_states:           Array<Option<Box<parser::State>>>,
  /// True if the database represents a valid set of rules. This may not be
  /// the case if, for example, the database is comprised of rules that
  /// reference non-extant non-terminals.
  valid:                   bool,
  /// items

  /// All items that follow a non-terminal
  follow_items:            Array<Array<ItemIndex>>,
  /// Item closures, stores the closure of all items, excluding the closure's of
  /// items that are complete.
  item_closures:           Array<Array<Array<ItemIndex>>>,
  ///NonTerminal Recursion Type
  recursion_types:         Array<u8>,
  /// Reduction types
  reduction_types:         Array<ReductionType>,
}

impl ParserDatabase {
  pub fn new(
    root_grammar_id: GrammarIdentities,
    nonterm_symbols: Array<SymbolId>,
    nonterm_names: Array<(IString, IString)>,
    nonterm_nterm_rules: Array<Array<DBRuleKey>>,
    rules: Array<DBRule>,
    tokens: Array<DBTokenData>,
    entry_points: Array<DBEntryPoint>,
    string_store: IStringStore,
    custom_states: Array<Option<Box<parser::State>>>,
    valid: bool,
  ) -> Self {
    Self {
      root_grammar_id,
      nonterm_symbols,
      nonterm_names,
      nonterm_nterm_rules,
      rules,
      tokens,
      entry_points,
      string_store,
      custom_states,
      valid,
      follow_items: Default::default(),
      item_closures: Default::default(),
      recursion_types: Default::default(),
      reduction_types: Default::default(),
      nonterm_predecessors: Default::default(),
      nonterm_symbol_to_rules: Default::default(),
    }
    .process_data()
  }

  fn process_data(mut self) -> Self {
    let mut recursion_types: Array<u8> = vec![Default::default(); self.nonterms_len()];
    let mut follow_items_base: Array<Array<ItemIndex>> = vec![Default::default(); self.nonterms_len()];
    let mut follow_items: Array<Array<ItemIndex>> = vec![Default::default(); self.nonterms_len()];
    let mut reduction_types: Array<ReductionType> = vec![Default::default(); self.rules.len()];
    let mut item_closures = Array::with_capacity(self.rules.len());
    let mut nonterm_predecessors: OrderedMap<DBNonTermKey, OrderedSet<DBNonTermKey>> = OrderedMap::default();
    let mut nonterm_symbol_to_rules: OrderedMap<DBNonTermKey, OrderedSet<DBRuleKey>> = OrderedMap::default();

    let db = &self;
    // Calculate closure for all items, and follow sets and recursion type for all
    // non-terminals.

    for (index, rule) in self.rules.iter().enumerate() {
      item_closures.insert(index, Vec::with_capacity(rule.rule.symbols.len()))
    }

    for item in self.rules().iter().enumerate().map(|(id, _)| Item::from((DBRuleKey(id as u32), db))).flat_map(|mut i| {
      let mut out = Vec::with_capacity(i.sym_len() as usize);
      out.push(i);
      while let Some(inc) = i.increment() {
        out.push(inc);
        i = inc
      }
      out
    }) {
      let is_scanner = matches!(self.nonterm_sym(item.nonterm_index(db)), SymbolId::DBNonTerminalToken { .. });
      let mode = if is_scanner { GraphType::Scanner } else { GraphType::Parser };

      match item.get_type(db) {
        ItemType::TokenNonTerminal(nonterm, _) if is_scanner => follow_items_base[nonterm.0 as usize].push(item.index),
        ItemType::NonTerminal(nonterm) => {
          let val = nonterm_symbol_to_rules.entry(nonterm).or_default();
          val.insert(item.rule_id());
          follow_items_base[nonterm.0 as usize].push(item.index)
        }
        _ => {}
      }

      //-------------------------------------------------------------------------------------------
      // Use completed items to file out the reduce types table.
      if item.is_complete() {
        reduction_types[item.rule_id().0 as usize] = if item.rule_is_left_recursive(mode, db) {
          ReductionType::LeftRecursive
        } else if item.rule(db).ast.as_ref().is_some_and(|a| matches!(a, ASTToken::Defined(..))) {
          ReductionType::SemanticAction
        } else if item.sym_len() == 1 {
          match item.to_initial().is_term(mode, db) {
            true => ReductionType::SingleTerminal,
            false => ReductionType::SingleNonTerminal,
          }
        } else {
          ReductionType::Mixed
        };
      }

      //-------------------------------------------------------------------------------------------
      // Calculate closures for uncompleted items.
      fn create_closure(value: Item, db: &ParserDatabase, mode: GraphType) -> Items {
        if let Some(nterm) = value.nonterm_index_at_sym(mode, db) {
          if let Ok(rules) = db.nonterm_rules(nterm) {
            rules.iter().map(|r| Item::from((*r, db))).collect()
          } else {
            Default::default()
          }
        } else {
          Default::default()
        }
      }

      let root_nonterm = item.nonterm_index(db);
      let mut recursion_encountered = false;
      let mut closure = ItemSet::from_iter([]);
      let mut queue = VecDeque::from_iter([item]);

      while let Some(kernel_item) = queue.pop_front() {
        if closure.insert(kernel_item) {
          match kernel_item.get_type(db) {
            ItemType::TokenNonTerminal(nonterm, _) => {
              if is_scanner {
                for item in create_closure(kernel_item, &self, mode) {
                  recursion_encountered |= nonterm == root_nonterm;
                  queue.push_back(item.align(&kernel_item))
                }
              }
            }
            ItemType::NonTerminal(nonterm) => {
              for item in create_closure(kernel_item, &self, mode) {
                recursion_encountered |= nonterm == root_nonterm;
                queue.push_back(item.align(&kernel_item));

                if root_nonterm != nonterm {
                  nonterm_predecessors.entry(nonterm).or_insert(Default::default()).insert(root_nonterm);
                }
              }
            }
            _ => {}
          }
        }
      }

      // Don't include the root item.
      closure.remove(&item);

      item_closures
        .get_mut(item.rule_id().0 as usize)
        .unwrap()
        .insert(item.sym_index() as usize, closure.into_iter().map(|i| i.index).collect::<Array<_>>());

      //-------------------------------------------------------------------------------------------
      // Update recursion type
      if recursion_encountered {
        if item.is_initial() {
          recursion_types[root_nonterm.0 as usize] |= RecursionType::LeftRecursive as u8;
        } else {
          recursion_types[root_nonterm.0 as usize] |= RecursionType::RightRecursive as u8;
        }
      }
    }

    // Update nonterminal follow lists with items of non-terminals that arise from
    // items that are completed after shifting over the initial non-terminal
    for (base_id, _) in self.nonterm_symbols.iter().enumerate() {
      let mut nonterm_ids = VecDeque::from_iter([base_id]);
      let mut seen = Set::new();

      while let Some(id) = nonterm_ids.pop_front() {
        if seen.insert(id) {
          for static_item in &follow_items_base[id] {
            follow_items[base_id].push(*static_item);
            let item = Item::from((*static_item, db));
            if item.is_penultimate() {
              nonterm_ids.push_back(item.nonterm_index(db).0 as usize)
            }
          }
        }
      }
    }
    self.nonterm_predecessors = nonterm_predecessors;
    self.item_closures = item_closures;
    self.recursion_types = recursion_types;
    self.follow_items = follow_items;
    self.reduction_types = reduction_types;
    self.nonterm_symbol_to_rules = nonterm_symbol_to_rules;
    self
  }

  /// Prints token ids and their friendly names to the console
  pub fn print_tokens(&self) {
    let token_strings =
      self.tokens.iter().enumerate().map(|(idx, t)| format!("{:>0000}: {}", idx, t.name.to_str(&self.string_store).as_str()));

    println!("{}", token_strings.collect::<Vec<_>>().join("\n"))
  }

  pub fn is_valid(&self) -> bool {
    self.valid
  }

  /// Returns an array of [DBNonTermKey]s of the entry point non-terminals.
  pub fn entry_nterm_keys(&self) -> Array<DBNonTermKey> {
    self.entry_points.iter().map(|k| k.nonterm_key).collect()
  }

  /// Returns an array of [DBNonTermKey]s of the entry point non-terminals.
  pub fn entry_nterm_map(&self) -> OrderedMap<DBNonTermKey, &DBEntryPoint> {
    self.entry_points.iter().map(|k| (k.nonterm_key, k)).collect()
  }

  /// Returns an array of [EntryPoint]s of the entry point non-terminals.
  pub fn entry_points(&self) -> Array<&DBEntryPoint> {
    self.entry_points.iter().map(|k| k).collect()
  }

  /// Returns the number of non-terminals stored in the DB
  pub fn nonterms_len(&self) -> usize {
    self.nonterm_symbols.len()
  }

  /// Returns an ordered array of all non-terminals within the DB
  pub fn nonterms(&self) -> &Array<SymbolId> {
    &self.nonterm_symbols
  }

  /// Returns a map from a nonterm key to rules that have the matching nonterm
  /// symbol in their bodies.
  pub fn get_nonterm_symbol_to_rules(&self) -> &OrderedMap<DBNonTermKey, OrderedSet<DBRuleKey>> {
    &self.nonterm_symbol_to_rules
  }

  /// Returns an ordered array of all non-terminals within the DB
  pub fn get_nonterminal_predecessors(&self, key: DBNonTermKey) -> Option<&OrderedSet<DBNonTermKey>> {
    self.nonterm_predecessors.get(&key)
  }

  /// Given a [DBNonTermKey] returns the SymbolId representing the non-terminal,
  /// or [SymbolId::Undefined] if the id is invalid.
  pub fn nonterm_from_name(&self, name: &str) -> DBNonTermKey {
    let string = name.to_token();
    self
      .nonterm_names
      .iter()
      .enumerate()
      .find_map(|(v, (a, b))| if *a == string || *b == string { Some(v.into()) } else { None })
      .unwrap_or_default()
  }

  /// Returns the guid name of the database as a string.
  pub fn name_string(&self) -> String {
    self.root_grammar_id.guid_name.to_string(&self.string_store)
  }

  pub fn friendly_name_string(&self) -> String {
    self.root_grammar_id.name.to_string(&self.string_store)
  }

  /// Given a [DBNonTermKey] returns the SymbolId representing the non-terminal,
  /// or [SymbolId::Undefined] if the id is invalid.
  pub fn nonterm_sym(&self, key: DBNonTermKey) -> SymbolId {
    debug_assert!((key.0 as usize) < self.nonterm_symbols.len(), "Invalid DBNonTermKey received");
    self.nonterm_symbols[key.0 as usize].clone()
  }

  /// Given a [DBNonTermKey] returns an IString comprising the name of the
  /// non-terminal, or an empty string if the id is invalid.
  pub fn nonterm_guid_name(&self, key: DBNonTermKey) -> IString {
    self.nonterm_names.get(key.0 as usize).cloned().map(|(n, _)| n).unwrap_or_default()
  }

  /// Given a [DBNonTermKey] returns a [GuardedStr] of the non-terminal's name.
  /// Returns an empty string if the key is invalid.
  pub fn nonterm_guid_name_string<'a>(&'a self, key: DBNonTermKey) -> String {
    self.nonterm_guid_name(key).to_string(&self.string_store)
  }

  /// Given a [DBNonTermKey] returns an IString comprising the name of the
  /// non-terminal, or an empty string if the id is invalid.
  pub fn nonterm_friendly_name(&self, key: DBNonTermKey) -> IString {
    self.nonterm_names.get(key.0 as usize).cloned().map(|(_, n)| n).unwrap_or_default()
  }

  /// Given a [DBNonTermKey] returns a [GuardedStr] of the non-terminal's name.
  /// Returns an empty string if the key is invalid.
  pub fn nonterm_friendly_name_string<'a>(&'a self, key: DBNonTermKey) -> String {
    self.nonterm_friendly_name(key).to_string(&self.string_store)
  }

  /// Given a [DBSymKey] returns the token identifier representing the symbol,
  pub fn token(&self, key: DBTermKey) -> DBTokenData {
    debug_assert!((key.0 as usize) < self.tokens.len(), "Invalid DBSymKey received");
    self.tokens[key.0 as usize]
  }

  /// Given a [DBSymKey] returns the token identifier representing the symbol,
  pub fn tokens(&self) -> &Array<DBTokenData> {
    &self.tokens
  }

  /// Given a [DBSymKey] returns the token identifier representing the symbol,
  pub fn tok_val(&self, key: DBTermKey) -> usize {
    #[cfg(debug_assertions)]
    {
      let val = self.token(key).tok_id.0 as usize;
      debug_assert!((key.0 as usize) == val);
      val
    }
    #[cfg(not(debug_assertions))]
    {
      key.0 as usize
    }
  }

  /// Given a [DBSymKey] returns the token identifier representing the symbol,
  pub fn tok_data(&self, key: DBTermKey) -> &DBTokenData {
    self.tokens.get(key.0 as usize).as_ref().unwrap()
  }

  /// Given a [DBTermKey] returns the [DBNonTermKey] representing the scanner
  /// nonterminal for the symbol, or None
  pub fn tok_prod(&self, key: DBTermKey) -> Option<DBNonTermKey> {
    self.tokens.get(key.0 as usize).map(|s| s.nonterm_id)
  }

  /// Given a [DBTermKey] returns the associated [SymbolId]
  pub fn sym(&self, key: DBTermKey) -> SymbolId {
    self.tokens.get(key.0 as usize).map(|s| s.sym_id).unwrap_or_default()
  }

  /// Given a nonterm [DBNonTermKey] returns an [Array] of [DBRuleKey]
  /// belonging to rules that reduce to that nonterm, or `None` if the id is
  /// invalid.
  pub fn nonterm_rules(&self, key: DBNonTermKey) -> RadlrResult<&Array<DBRuleKey>> {
    o_to_r(self.nonterm_nterm_rules.get(key.0 as usize), "Could not find rule")
  }

  /// Returns the internal Rules
  pub fn rules(&self) -> &[DBRule] {
    self.rules.as_slice()
  }

  /// Given a [DBRuleKey] returns an [Rule], or `None` if
  /// the id is invalid.
  pub fn rule(&self, key: DBRuleKey) -> &Rule {
    if cfg!(debug_assertions) {
      self.rules.get(key.0 as usize).map(|e| &e.rule).unwrap()
    } else {
      unsafe { self.rules.get(key.0 as usize).map(|e| &e.rule).unwrap_unchecked() }
    }
  }

  /// Given a [DBRuleKey] returns an [Rule], or `None` if
  /// the id is invalid.
  pub fn db_rule(&self, key: DBRuleKey) -> &DBRule {
    if cfg!(debug_assertions) {
      self.rules.get(key.0 as usize).unwrap()
    } else {
      unsafe { self.rules.get(key.0 as usize).unwrap_unchecked() }
    }
  }

  /// Given a [DBRuleKey] returns an [Rule], or `None` if
  /// the id is invalid.
  pub fn custom_state(&self, key: DBNonTermKey) -> Option<&parser::State> {
    if cfg!(debug_assertions) {
      self.custom_states.get(key.0 as usize).unwrap().as_deref()
    } else {
      unsafe { self.custom_states.get(key.0 as usize).unwrap_unchecked().as_deref() }
    }
  }

  /// Given a [DBRuleKey] returns the [DBNonTermKey] the rule reduces to.
  pub fn rule_nonterm(&self, key: DBRuleKey) -> DBNonTermKey {
    self.rules.get(key.0 as usize).map(|e| e.nonterm).unwrap_or_default()
  }

  /// Returns a reference to the [IStringStore]
  pub fn string_store(&self) -> &IStringStore {
    &self.string_store
  }

  /// Returns a reference to the [IStringStore]
  pub fn get_reduce_type(&self, rule_id: DBRuleKey) -> ReductionType {
    self.reduction_types[rule_id.0 as usize]
  }

  pub fn nonterm_recursion_type(&self, nonterm: DBNonTermKey) -> RecursionType {
    match self.recursion_types[nonterm.0 as usize] {
      3 => RecursionType::LeftRightRecursive,
      2 => RecursionType::RightRecursive,
      1 => RecursionType::LeftRecursive,
      _ => RecursionType::None,
    }
  }

  /// Returns the closure of the item.
  /// > note: The closure does not include the item used as the seed for the
  /// > closure.
  pub fn get_closure<'db>(&'db self, item: &Item) -> impl ItemContainerIter + 'db {
    let item = *item;
    self.item_closures[item.rule_id().0 as usize][item.sym_index() as usize].iter().map(move |s| {
      let item = Item::from((*s, self)).as_from(item);

      item
    })
  }

  /// Returns all regular (non token) nonterminals.
  pub fn parser_nonterms(&self) -> Array<DBNonTermKey> {
    self
      .nonterms()
      .iter()
      .enumerate()
      .filter_map(|(i, p)| match p {
        SymbolId::NonTerminal { .. } => Some(DBNonTermKey(i as u32)),
        _ => None,
      })
      .collect()
  }

  /// Returns an iterator of all items that are `_ = ...•Aa`  for some
  /// [DBNonTermKey] `A`, or in other words this returns the list of items that
  /// would shift over the [DBNonTermKey] `A`. If an item is `B = ...•A`, then
  /// this also returns items that are `_ = ...•Ba`
  pub fn nonterm_follow_items<'db>(&'db self, nonterm: DBNonTermKey) -> impl Iterator<Item = Item> + Clone + 'db {
    self.follow_items[nonterm.0 as usize].iter().map(move |i| Item::from((*i, self)))
  }
}

macro_rules! indexed_id_implementations {
  ($id_type:ty) => {
    impl $id_type {
      pub fn to_string(&self) -> String {
        self.0.to_string()
      }
    }

    impl From<u32> for $id_type {
      fn from(value: u32) -> Self {
        Self(value)
      }
    }

    impl From<usize> for $id_type {
      fn from(value: usize) -> Self {
        Self(value as u32)
      }
    }

    impl Into<usize> for $id_type {
      fn into(self) -> usize {
        self.0 as usize
      }
    }

    impl Into<u32> for $id_type {
      fn into(self) -> u32 {
        self.0 as u32
      }
    }

    impl Default for $id_type {
      fn default() -> Self {
        Self(u32::MAX)
      }
    }
  };
}

/// An opaque key used for the access of a rule in a [CompileDatabase]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct DBRuleKey(u32);
indexed_id_implementations!(DBRuleKey);

/// Used as a lookup key for non-terminal data stored within a
/// [CompileDatabase]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct DBNonTermKey(u32);
indexed_id_implementations!(DBNonTermKey);

impl DBNonTermKey {
  /// Returns the symbol representation of this index.
  pub fn to_sym(&self) -> SymbolId {
    SymbolId::DBNonTerminal { key: *self }
  }

  /// Retrieves the binary / bytecode id of the nonterminal.
  pub fn to_val(&self) -> u32 {
    self.0 as u32
  }
}

/// Used as a lookup key for a symbol data within a
/// [CompileDatabase]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct DBTermKey(u32);
indexed_id_implementations!(DBTermKey);

impl DBTermKey {
  pub fn default_sym() -> Self {
    Self(0)
  }

  /// Retrieves the binary / bytecode id of the symbol.
  pub fn to_val(&self) -> u32 {
    self.0
  }

  pub fn to_index(&self) -> usize {
    (self.0) as usize
  }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum ReductionType {
  /// Any reduction resulting in the execution of a some kind of semantic
  /// action. At this point only `:ast` semantic actions are available.
  SemanticAction,
  /// A reduction of a terminal symbol to a nonterminal
  SingleTerminal,
  /// A reduction of single nonterminal symbol to another nonterminal
  SingleNonTerminal,
  /// A reduction of a left-recursive rule
  LeftRecursive,
  #[default]
  /// A reduction of more than one symbol to a nonterminal
  Mixed,
}

// The type of recursion that can occur for a given rule.
#[derive(Default, Clone, Copy, Debug)]
pub enum RecursionType {
  #[default]
  None               = 0,
  LeftRecursive      = 1,
  RightRecursive     = 2,
  LeftRightRecursive = 3,
}

impl RecursionType {
  #[inline]
  pub fn is_recursive(&self) -> bool {
    self.is_left_recursive() || self.is_right_recursive()
  }

  #[inline]
  pub fn is_left_recursive(&self) -> bool {
    matches!(self, Self::LeftRecursive | Self::LeftRightRecursive)
  }

  #[inline]
  pub fn is_right_recursive(&self) -> bool {
    matches!(self, Self::RightRecursive | Self::LeftRightRecursive)
  }
}

#[derive(Clone, Debug)]
pub struct DBRule {
  pub rule:       Rule,
  pub nonterm:    DBNonTermKey,
  pub is_scanner: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DBTokenData {
  /// The symbol type and precedence.
  pub sym_id:     SymbolId,
  /// The friendly name of this token.
  pub name:       IString,
  /// The scanner non-terminal id of this token.
  pub nonterm_id: DBNonTermKey,
  /// The id of the symbol when used as a lexer token.
  pub tok_id:     DBTermKey,
}

#[derive(Clone, Copy, Debug)]
pub struct DBEntryPoint {
  pub nonterm_key:        DBNonTermKey,
  /// The GUID name of the non-terminal.
  pub nonterm_name:       IString,
  /// The GUID name of the non-terminal's entry state.
  pub nonterm_entry_name: IString,
  /// The GUID name of the non-terminal's exit state.
  pub nonterm_exit_name:  IString,
  /// The friendly name of the non-terminal as specified in the
  /// `IMPORT <nonterm> as <entry_name>` preamble.
  pub entry_name:         IString,
  ///
  pub export_id:          usize,
  /// `true` if the entry point was defined through an `ENTRY` clause
  pub is_export:          bool,
}
