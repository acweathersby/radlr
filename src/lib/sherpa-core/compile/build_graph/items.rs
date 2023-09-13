use super::{
  build::{GroupedFirsts, TransitionGroup},
  graph::*,
};
use crate::types::*;
use sherpa_rust_runtime::utf8::{get_token_class_from_codepoint, lookup_table::CodePointClass};
use std::collections::VecDeque;

/// Returns a tuple comprised of a vector of all items that follow the given
/// item, provided the given item is in a complete state, and a list of a all
/// items that are completed from directly or indirectly transitioning on the
/// nonterminal of the given item.
pub(crate) fn get_follow<'db>(
  gb: &GraphBuilder<'db>,
  item: Item<'db>,
  single_reduction_only: bool,
) -> SherpaResult<(Items<'db>, Items<'db>)> {
  get_follow_internal(gb.graph(), item, single_reduction_only)
}

/// Returns a tuple comprised of a vector of all items that follow the given
/// item, provided the given item is in a complete state, and a list of a all
/// items that are completed from directly or indirectly transitioning on the
/// nonterminal of the given item.
pub(crate) fn get_follow_internal<'db>(
  graph: &GraphHost<'db>,
  item: Item<'db>,
  single_reduction_only: bool,
) -> SherpaResult<(Items<'db>, Items<'db>)> {
  if !item.is_complete() {
    return SherpaResult::Ok((vec![item], vec![]));
  }
  let ____is_scan____ = graph.graph_type == GraphType::Scanner;
  let mut completed = OrderedSet::new();
  let mut follow = OrderedSet::new();
  let mut queue = VecDeque::from_iter(vec![item]);
  let db = graph.get_db();
  let mode = graph.graph_type;
  let root_nterm = item.nonterm_index();

  while let Some(item) = queue.pop_front() {
    if completed.insert(item) {
      let nterm: DBNonTermKey = item.nonterm_index();

      if single_reduction_only && nterm != root_nterm {
        continue;
      }

      let closure = if item.goal_is_oos() {
        db.nonterm_follow_items(nterm)
          //graph[item.origin_state]
          //  .get_root_closure_ref()?
          //.iter()
          .filter(|i| /* i.is_out_of_scope() && */ i.nonterm_index_at_sym(mode).unwrap_or_default() == nterm)
          .map(|i| i.to_origin(item.origin).to_oos_index().to_origin_state(StateId::root()))
          .collect::<Array<_>>()
      } else {
        let state = item.origin_state;
        let origin = item.origin;
        let goal = item.goal;
        graph[item.origin_state]
          .kernel_items_ref()
          .iter()
          .filter(|kernel| kernel.goal == goal)
          .flat_map(|kernel| {
            db.get_closure(kernel)
              .map(|i| {
                if i.is_canonically_equal(kernel) {
                  *kernel
                } else {
                  i.to_origin_state(state).to_goal(goal).to_origin(origin)
                }
              })
              .chain([kernel.clone()])
          })
          .filter(|i| i.nonterm_index_at_sym(mode) == Some(nterm))
          .collect::<Array<_>>()
      };

      if closure.len() > 0 {
        for item in closure.try_increment() {
          match item.get_type() {
            ItemType::Completed(_) => queue.push_back(item),
            _ => {
              follow.insert(item);
            }
          }
        }
      } else if !item.origin_state.is_root() {
        let parent_state = graph[item.origin_state].get_parent();
        queue.push_back(item.to_origin_state(parent_state));
      } else if !____is_scan____ && !item.is_out_of_scope() {
        let item: Item<'_> = item.to_oos_index();
        queue.push_back(item);
      }
    }
  }

  SherpaResult::Ok((follow.to_vec(), completed.to_vec()))
}

// Inserts out of scope sentinel items into the existing
// items groups if we are in scanner mode and the item that
// was completed belongs to the parse state goal set.
pub(super) fn _get_oos_follow_from_completed<'db>(
  gb: &GraphBuilder<'db>,
  completed_items: &Items<'db>,
  handler: &mut dyn FnMut(Follows<'db>),
) -> SherpaResult<()> {
  let mut out = OrderedSet::new();
  for completed_item in completed_items {
    if !completed_item.is_out_of_scope() {
      let (_, completed) = get_follow(gb, *completed_item, false)?;

      let goals: ItemSet = get_goal_items_from_completed(&completed, gb.graph());

      for goal in goals {
        let (follow, _) = get_follow(
          gb,
          goal
            .to_complete()
            .to_origin(if gb.is_scanner() { Origin::ScanCompleteOOS } else { Origin::GoalCompleteOOS })
            .to_oos_index(),
          false,
        )?;
        out.extend(&mut follow.into_iter().map(|f| -> Follow { (*completed_item, f, gb.get_mode()).into() }));
      }
    }
  }
  if !out.is_empty() {
    handler(out.into_iter().collect());
  }
  SherpaResult::Ok(())
}

pub(super) fn get_goal_items_from_completed<'db, 'follow>(items: &Items<'db>, graph: &GraphHost<'db>) -> ItemSet<'db> {
  items.iter().filter(|i| graph.item_is_goal(*i)).cloned().collect()
}

pub(super) fn _merge_follow_items_into_group<'db>(
  follows: &Vec<Follow<'db>>,
  _par: StateId,
  firsts_groups: &mut GroupedFirsts<'db>,
) {
  // Dumb symbols that could cause termination of parse into the intermediate
  // item groups

  for follow in follows {
    if !firsts_groups.contains_key(&follow.sym) {
      firsts_groups.insert(follow.sym, (follow.prec, vec![*follow]));
    }
  }
}

pub(super) fn merge_occluding_token_items<'db>(from_groups: GroupedFirsts<'db>, into_groups: &mut GroupedFirsts<'db>) {
  for (sym, group) in into_groups.iter_mut() {
    let occluding_items = get_set_of_occluding_token_items(sym, group, &from_groups);
    group.1.extend(occluding_items);
  }
}

pub(super) fn get_set_of_occluding_token_items<'db>(
  into_sym: &SymbolId,
  into_group: &TransitionGroup<'db>,
  groups: &GroupedFirsts<'db>,
) -> Firsts<'db> {
  let mut occluding = Firsts::new();
  let into_prec = into_group.0;

  if into_prec >= 9999 {
    return occluding;
  }

  for (from_sym, from_group) in groups.iter().filter(|(other_sym, (prec, _))| into_sym != *other_sym && into_prec <= *prec) {
    if symbols_occlude(into_sym, from_sym) {
      occluding.extend(from_group.1.iter().cloned());
    }
  }

  occluding
}

/// Compares whether symbolB occludes symbolA
/// ( produces an ambiguous parse path )
///
/// Symbols that can occlude are as follows
///
/// - `g:id` and any single identifier character.
/// - `g:num` and any single numeric character.
/// - `g:sym` and any single character thats not a numeric, identifier, space,
///   newline, or tab.
fn symbols_occlude(symA: &SymbolId, symB: &SymbolId) -> bool {
  match symA {
    SymbolId::Char { char, .. } => match symB {
      SymbolId::ClassNumber { .. } => {
        (*char < 128) && get_token_class_from_codepoint(*char as u32) == CodePointClass::Number as u32
      }
      SymbolId::ClassIdentifier { .. } => {
        (*char < 128) && get_token_class_from_codepoint(*char as u32) == CodePointClass::Identifier as u32
      }
      SymbolId::ClassSymbol { .. } => {
        (*char < 128) && get_token_class_from_codepoint(*char as u32) == CodePointClass::Symbol as u32
      }
      SymbolId::Default => false,
      symB => *symA == *symB,
    },
    SymbolId::Codepoint { val, .. } => match symB {
      SymbolId::ClassNumber { .. } => get_token_class_from_codepoint(*val) == CodePointClass::Number as u32,
      SymbolId::ClassIdentifier { .. } => get_token_class_from_codepoint(*val) == CodePointClass::Identifier as u32,
      SymbolId::ClassSymbol { .. } => get_token_class_from_codepoint(*val) == CodePointClass::Symbol as u32,
      SymbolId::Default => false,
      symB => *symA == *symB,
    },
    SymbolId::Default => false,
    symA => *symA == *symB,
  }
}

pub(crate) fn get_completed_item_artifacts<'a, 'db: 'a, 'follow, T: ItemRefContainerIter<'a, 'db>>(
  gb: &GraphBuilder<'db>,
  completed: T,
) -> SherpaResult<CompletedItemArtifacts<'db>> {
  let mut follow_pairs = OrderedSet::new();
  //let mut follow_items = ItemSet::new();
  let mut default_only_items = ItemSet::new();

  for c_i in completed {
    let (f, _) = get_follow(gb, *c_i, false)?;

    if f.is_empty() {
      default_only_items.insert(*c_i);
    } else {
      follow_pairs.extend(f.iter().flat_map(|i| vec![*i].iter().closure::<Vec<_>>(gb.current_state_id())).map(|i| {
        TransitionPair {
          kernel: *c_i,
          next:   i.to_origin(c_i.origin).to_origin_state(gb.current_state_id()),
          prec:   i.token_precedence(),
          sym:    i.sym(),
        }
      }));
      //follow_items.append(&mut f.to_set());
    }
  }

  SherpaResult::Ok(CompletedItemArtifacts { follow_pairs, default_only: default_only_items })
}
