//! Utility functions for the evaluation, interpretation, and
//! comprehension of items
use crate::types::{item_2::Item as Item2, GrammarStore, Item, ProductionId, SymbolID};
use std::collections::{BTreeSet, VecDeque};

/// Retrieve the initial items of a production. Returns vector of
/// items, one for each rule belonging to the production.
#[inline]
pub(crate) fn get_production_start_items(prod_id: &ProductionId, g: &GrammarStore) -> Vec<Item> {
  g.production_rules
    .get(prod_id)
    .unwrap()
    .iter()
    .map(|id| Item::from_rule(id, g).unwrap())
    .collect()
}

#[inline]
pub(crate) fn get_production_start_items2(prod_id: &ProductionId, g: &GrammarStore) -> Vec<Item2> {
  g.production_rules
    .get(prod_id)
    .unwrap()
    .iter()
    .map(|id| Item2::from(g.get_rule(id).unwrap()))
    .collect()
}

/// Create the closure of a set of items.
pub(crate) fn create_closure(items: &[Item], g: &GrammarStore) -> Vec<Item> {
  let mut seen = BTreeSet::<Item>::new();

  let mut queue = VecDeque::<Item>::from_iter(items.iter().cloned());

  while let Some(item) = queue.pop_front() {
    if seen.insert(item) {
      if let SymbolID::Production(prod_id, _) = &item.get_symbol(g) {
        for item in get_production_start_items(prod_id, g) {
          queue.push_back(item.to_empty_state())
        }
      }
    }
  }

  seen.into_iter().collect()
}

/// Retrieve the closure of an item that is cached in the grammar store.
#[inline]
pub(crate) fn get_closure_cached<'a>(item: &Item, g: &'a GrammarStore) -> &'a Vec<Item> {
  static empty_closure: Vec<Item> = vec![];
  if g.closures.get(&item.to_empty_state()).is_none() {
    &empty_closure
  } else {
    g.closures.get(&item.to_empty_state()).unwrap()
  }
}

/// Retrieve the closure of an item that is cached in the grammar
/// store and applies the state of the base item to all closure members.
#[inline]
pub(crate) fn get_closure_cached_with_state<'a>(item: &Item, g: &'a GrammarStore) -> Vec<Item> {
  if g.closures.get(&item.to_empty_state()).is_none() {
    vec![]
  } else {
    g.closures
      .get(&item.to_empty_state())
      .unwrap()
      .into_iter()
      .map(|i| i.to_state(item.get_state()))
      .collect()
  }
}
