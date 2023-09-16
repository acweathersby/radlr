use super::*;
use crate::{compile::build_graph::graph::GraphType, parser, CachedString, SherpaResult};
use std::collections::{HashMap, VecDeque};

/// Data used for the compilation of parse states. contains
/// additional metadata for compilation of LLVM and Bytecode
/// parsers.
#[derive(Clone, Default)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ParserDatabase {
  /// The name of the parser as defined by the `NAME <name>` preamble in
  /// the root grammar, or by the filename stem of of the root grammar's path.
  pub name: IString,
  /// Table of symbols.
  nonterm_symbols: Array<SymbolId>,
  /// Table of non-terminal names for public non-terminal.
  ///
  /// - First tuple member: GUID name,
  /// - Second member: friendly name.
  ///
  /// This is a 1-to-1 mapping of all non-terminal indices, so non-terminals
  /// that are scanner or are sub-non-terminals map to empty strings.
  nonterm_names: Array<(IString, IString)>,
  /// Table mapping non-terminal indices to rule indices.
  nonterm_nterm_rules: Array<Array<DBRuleKey>>,
  /// Table of all rules within the grammar and the non-terminal they reduce
  /// to.
  rules: Array<DBRule>,
  /// Table of all tokens generated by the scanners.
  tokens: Array<DBTokenData>,
  /// The entry point non-terminals of the grammar.
  entry_points: Array<EntryPoint>,
  /// The global string store
  string_store: IStringStore,
  /// Custom states that should be integrated into the final parsers
  custom_states: Array<Option<Box<parser::State>>>,
  /// True if the database represents a valid set of rules. This may not be
  /// the case if, for example, the database is comprised of rules that
  /// reference non-extant non-terminals.
  valid: bool,
  /// items

  /// All items that follow a non-terminal
  follow_items: Array<Array<StaticItem>>,
  /// Item closures, stores the closure of all items, excluding the closure's of
  /// items that are complete.
  item_closures: Array<Array<Array<StaticItem>>>,
  ///NonTerminal Recursion Type
  recursion_types: Array<u8>,
  /// Reduction types
  reduction_types: Array<ReductionType>,
}

impl ParserDatabase {
  pub fn new(
    name: IString,
    nonterm_symbols: Array<SymbolId>,
    nonterm_names: Array<(IString, IString)>,
    nonterm_nterm_rules: Array<Array<DBRuleKey>>,
    rules: Array<DBRule>,
    tokens: Array<DBTokenData>,
    entry_points: Array<EntryPoint>,
    string_store: IStringStore,
    custom_states: Array<Option<Box<parser::State>>>,
    valid: bool,
  ) -> Self {
    Self {
      name,
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
    }
    .process_data()
  }

  fn process_data(mut self) -> Self {
    let mut recursion_types: Array<u8> = vec![Default::default(); self.nonterms_len()];
    let mut follow_items_base: Array<Array<StaticItem>> = vec![Default::default(); self.nonterms_len()];
    let mut follow_items_final: Array<Array<StaticItem>> = vec![Default::default(); self.nonterms_len()];
    let mut reduce_types: Array<ReductionType> = vec![Default::default(); self.rules.len()];
    let mut closure_map = Array::with_capacity(self.rules.len());

    let db = &self;
    // Calculate closure for all items, and follow sets and recursion type for all
    // non-terminals.

    for (index, rule) in self.rules.iter().enumerate() {
      closure_map.insert(index, Vec::with_capacity(rule.rule.symbols.len()))
    }

    for item in self.rules().iter().enumerate().map(|(id, _)| Item::from_rule(DBRuleKey(id as u32), db)).flat_map(|mut i| {
      let mut out = Vec::with_capacity(i.len as usize);
      out.push(i);
      while let Some(inc) = i.increment() {
        out.push(inc);
        i = inc
      }
      out
    }) {
      let is_scanner = matches!(self.nonterm_sym(item.nonterm_index()), SymbolId::DBNonTerminalToken { .. });
      let mode = if is_scanner { GraphType::Scanner } else { GraphType::Parser };

      match item.get_type() {
        ItemType::TokenNonTerminal(nonterm, _) if is_scanner => {
          follow_items_base[nonterm.0 as usize].push((item.rule_id, item.sym_index))
        }
        ItemType::NonTerminal(nonterm) => follow_items_base[nonterm.0 as usize].push((item.rule_id, item.sym_index)),
        _ => {}
      }

      //-------------------------------------------------------------------------------------------
      // Use completed items to file out the reduce types table.
      if item.is_complete() {
        reduce_types[item.rule_id.0 as usize] = if item.is_left_recursive(mode) {
          ReductionType::LeftRecursive
        } else if item.rule().ast.as_ref().is_some_and(|a| matches!(a, ASTToken::Defined(..))) {
          ReductionType::SemanticAction
        } else if item.len == 1 {
          match item.to_start().is_term(mode) {
            true => ReductionType::SingleTerminal,
            false => ReductionType::SingleNonTerminal,
          }
        } else {
          ReductionType::Mixed
        };
      }

      //-------------------------------------------------------------------------------------------
      // Calculate closures for uncompleted items.

      fn create_closure<'db>(value: Item, db: &'db ParserDatabase, mode: GraphType) -> Items<'db> {
        if let Some(nterm) = value.nonterm_index_at_sym(mode) {
          if let Ok(rules) = db.nonterm_rules(nterm) {
            rules.iter().map(|r| Item::from_rule(*r, db)).collect()
          } else {
            Default::default()
          }
        } else {
          Default::default()
        }
      }

      let root_nonterm = item.nonterm_index();
      let mut recursion_encountered = false;
      let mut closure = ItemSet::from_iter([]);
      let mut queue = VecDeque::from_iter([item]);

      while let Some(kernel_item) = queue.pop_front() {
        if closure.insert(kernel_item) {
          match kernel_item.get_type() {
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
                queue.push_back(item.align(&kernel_item))
              }
            }
            _ => {}
          }
        }
      }

      // Don't include the root item.
      closure.remove(&item);

      closure_map
        .get_mut(item.rule_id.0 as usize)
        .unwrap()
        .insert(item.sym_index as usize, closure.into_iter().map(|i| (i.rule_id, i.sym_index)).collect::<Array<_>>());

      //-------------------------------------------------------------------------------------------
      // Update recursion type
      if recursion_encountered {
        if item.is_at_initial() {
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
            follow_items_final[base_id].push(*static_item);
            let item = Item::from_static(*static_item, db);
            if item.is_penultimate() {
              nonterm_ids.push_back(item.nonterm_index().0 as usize)
            }
          }
        }
      }
    }

    self.item_closures = closure_map;
    self.recursion_types = recursion_types;
    self.follow_items = follow_items_final;
    self.reduction_types = reduce_types;
    self
  }

  /// Prints token ids and their friendly names to the console
  #[cfg(debug_assertions)]
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

  /// Returns an array of [EntryPoint]s of the entry point non-terminals.
  pub fn entry_points(&self) -> Array<&EntryPoint> {
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

  pub fn get_entry_offset(&self, entry_name: &str, hash_map: &HashMap<IString, u32>) -> Option<usize> {
    let string = entry_name.to_token();
    self
      .entry_points()
      .iter()
      .find(|e| e.entry_name == string)
      .and_then(|e| hash_map.get(&e.nonterm_entry_name).map(|v| (*v) as usize))
  }

  pub fn get_entry_data(&self, entry_name: &str, hash_map: &HashMap<IString, u32>) -> Option<(usize, &EntryPoint)> {
    let string = entry_name.to_token();
    self
      .entry_points
      .iter()
      .find(|e| e.entry_name == string)
      .and_then(|e| Some((hash_map.get(&e.nonterm_entry_name).map(|v| (*v) as usize).unwrap_or_default(), e)))
  }

  /// Returns the name of the database as a string.
  pub fn name_string(&self) -> String {
    self.name.to_string(&self.string_store)
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

  /// Given a [DBNonTermKey] returns an [Array] of [DBRuleKey], or `None`
  /// if the id is invalid.
  pub fn nonterm_rules(&self, key: DBNonTermKey) -> SherpaResult<&Array<DBRuleKey>> {
    o_to_r(self.nonterm_nterm_rules.get(key.0 as usize), "Could not find rule")
  }

  /// Returns the internal Rules
  pub fn rules(&self) -> &[DBRule] {
    self.rules.as_slice()
  }

  /// Given a [DBRuleKey] returns an [Rule], or `None` if
  /// the id is invalid.
  pub fn rule(&self, key: DBRuleKey) -> &Rule {
    self.rules.get(key.0 as usize).map(|e| &e.rule).unwrap()
  }

  /// Given a [DBRuleKey] returns an [Rule], or `None` if
  /// the id is invalid.
  pub fn custom_state(&self, key: DBNonTermKey) -> Option<&parser::State> {
    self.custom_states.get(key.0 as usize).unwrap().as_deref()
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
  pub fn get_reduce_type<'db>(&'db self, rule_id: DBRuleKey) -> ReductionType {
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
  pub fn get_closure<'db>(&'db self, item: &Item<'db>) -> impl ItemContainerIter {
    self.item_closures[item.rule_id.0 as usize][item.sym_index as usize].iter().map(|s| Item::from_static(*s, self))
  }

  /// Returns all regular (non token) nonterminals.
  pub fn parser_nonterms<'db>(&'db self) -> Array<DBNonTermKey> {
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
  pub fn nonterm_follow_items<'db>(&'db self, nonterm: DBNonTermKey) -> impl Iterator<Item = Item<'db>> + Clone {
    self.follow_items[nonterm.0 as usize].iter().map(|i| Item::from_static(*i, self))
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DBRuleKey(u32);
indexed_id_implementations!(DBRuleKey);

/// Used as a lookup key for non-terminal data stored within a
/// [CompileDatabase]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DBTermKey(u32);
indexed_id_implementations!(DBTermKey);

impl DBTermKey {
  pub fn default_sym() -> Self {
    Self(0)
  }

  /// Retrieves the binary / bytecode id of the symbol.
  pub fn to_val(&self, db: &ParserDatabase) -> u32 {
    db.tok_val(*self) as u32
  }

  pub fn to_index(&self) -> usize {
    (self.0) as usize
  }
}

#[derive(Default, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
#[derive(Default, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum RecursionType {
  #[default]
  None = 0,
  LeftRecursive = 1,
  RightRecursive = 2,
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

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct DBRule {
  pub rule:       Rule,
  pub nonterm:    DBNonTermKey,
  pub is_scanner: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(debug_assertions, derive(Debug))]
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

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct EntryPoint {
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
}
