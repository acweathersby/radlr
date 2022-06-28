//! Utility functions for the evaluation, interpretation, and
//! comprehension of items

use std::collections::HashSet;
use std::collections::VecDeque;

use crate::primitives::GrammarStore;
use crate::primitives::Item;
use crate::primitives::ProductionId;
use crate::primitives::SymbolID;

/// Retrieve the initial items of a production. Returns vector of
/// items, one for each body belonging to the production.

pub fn get_production_start_items(
    production_id: &ProductionId,
    grammar: &GrammarStore,
) -> Vec<Item>
{
    grammar
        .production_bodies_table
        .get(production_id)
        .unwrap()
        .iter()
        .map(|id| Item::from_body(id, grammar).unwrap())
        .collect()
}

/// Retrieve the closure of a set of items.

pub fn get_closure(items: &[Item], grammar: &GrammarStore) -> Vec<Item>
{
    let mut seen = HashSet::<Item>::new();

    let mut queue = VecDeque::<Item>::from_iter(items.iter().cloned());

    while let Some(item) = queue.pop_front() {
        if seen.insert(item) {
            if let SymbolID::Production(prod_id, _) = &item.get_symbol(grammar)
            {
                for item in get_production_start_items(prod_id, grammar) {
                    queue.push_back(item)
                }
            }
        }
    }

    seen.into_iter().collect()
}

/// Retrieve the closure of an item that is cached in the grammar
/// store. Falls back to manually building the closure if it is not
/// cached. Does not modify the grammar store object.
pub fn get_closure_cached<'a>(
    item: &Item,
    grammar: &'a GrammarStore,
) -> &'a Vec<Item>
{
    static empty_closure: Vec<Item> = vec![];
    if item.is_end() {
        &empty_closure
    } else {
        grammar.closures.get(&item.to_zero_state()).unwrap()
    }
}

/// Memoized form of 'get_closure_cached', which adds the Item's
/// closure to the grammar store if it is not already present.

pub fn get_closure_cached_mut<'a>(
    item: &Item,
    grammar: &'a mut GrammarStore,
) -> &'a Vec<Item>
{
    let item = &item.to_zero_state();

    if !grammar.closures.contains_key(item) {
        let closure = get_closure(&vec![*item], grammar);

        grammar.closures.insert(*item, closure);
    }

    grammar.closures.get(item).unwrap()
}
