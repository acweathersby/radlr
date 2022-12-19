//! # Recursive Ascent Graph Compiler
//!
//! Though not a true representation of a recursive descent parser, which is simply the conversion of
//! an LR parse table into recursive functions, this compiler provides one of the most essential
//! features of LR parsers, the ability to parse grammars with left recursions. This feature is embodied
//! within by providing a primary GOTO state, in which productions that have
//! been completed are matched with branches that transition on a Production ids. Beyond that,
//! the implementation reuses much of the functionality found within the [recursive_descent], [peek],
//! and [lr] modules.
//!
//! Note: Since scanner productions are guaranteed to not have left recursion of any kind, there
//! is no need to run TokenProductions through this process.
use crate::{
  intermediate::utils::{get_follow_closure, hash_group_btreemap},
  journal::Journal,
  types::{GraphNode, TransitionGraph as TPack, *},
};
use std::{
  collections::{BTreeSet, VecDeque},
  rc::Rc,
  vec,
};

use super::{create_node, process_node};

pub(crate) fn construct_recursive_ascent(
  j: &mut Journal,
  goto_seeds: BTreeSet<Item>,
  root_ids: BTreeSet<ProductionId>,
) -> SherpaResult<TPackResults> {
  let g = j.grammar().unwrap();

  let mut t =
    TPack::new(g.clone(), TransitionMode::RecursiveAscent, ScanType::None, &vec![], root_ids);
  t.increment_lane(1);
  t.goto_scoped_closure = Some(Rc::new(Box::<Vec<Item>>::new(
    (!t.is_scan()).then(|| get_follow_closure(&g, &t.root_prod_ids)).unwrap_or_default(),
  )));

  let goto_seeds = goto_seeds.to_empty_state().to_set();

  t.accept_items = goto_seeds.clone();

  // Get closures of all items that could transition on the same production.

  let mut root_node =
    GraphNode::new(&t, SymbolID::Start, None, goto_seeds.clone().to_vec(), NodeType::RAStart);
  root_node.edge_type = EdgeType::Start;
  let parent_index = Some(t.insert_node(root_node));
  let mut unfulfilled_root = Some(*t.root_prod_ids.first().unwrap());

  for (production_id, group) in
    hash_group_btreemap(goto_seeds, |_, i| i.get_production_id_at_sym(&t.g))
  {
    let have_root_production = t.root_prod_ids.contains(&production_id);
    let mut have_end_items = false;
    let lane = t.increment_lane(1);

    let mut items: Vec<Item> = group
      .iter()
      .map(|i| {
        let stated_item = if i.completed() {
          have_end_items = true;
          i.to_state(ItemState::OUT_OF_SCOPE)
        } else {
          i.try_increment().to_state(ItemState::new(lane, OriginData::Undefined))
        };

        stated_item
      })
      .collect();

    let mut goto_node = create_node(
      &t,
      SymbolID::Production(production_id, GrammarId::default()),
      items.clone(),
      NodeType::Goto,
      EdgeType::Goto,
      parent_index,
      parent_index,
      items.as_vec().term_items(&t.g),
    );

    if have_root_production || (group.len() > 1 && have_end_items) {
      t.out_of_scope_closure =
        Some(g.lr_items.iter().flat_map(|(_, i)| i).cloned().collect::<Vec<Item>>());

      if have_root_production {
        unfulfilled_root = None;
        let mut out_of_scope_items = get_out_of_scope(&g, production_id, &group, false);
        goto_node.transition_items.append(&mut out_of_scope_items.clone());
        items.append(&mut out_of_scope_items);
      }
      let items = goto_node.transition_items.clone().to_set().to_vec();
      let node_index = t.insert_node(goto_node);

      t.queue_node(ProcessGroup { node_index, items, ..Default::default() });
    } else {
      let node_index = t.insert_node(goto_node);
      t.queue_node(ProcessGroup { node_index, items, ..Default::default() });
    }

    t.accept_items.append(&mut group.clone().to_complete().to_set());
  }

  // If the root production is not covered in the goto branches
  // then create a new node that serves as an accepting state
  // if the active production id is the root.

  if let Some(production_id) = unfulfilled_root {
    let mut goto_node = GraphNode::new(
      &t,
      SymbolID::Production(production_id, GrammarId::default()),
      parent_index,
      vec![],
      NodeType::Pass,
    );
    goto_node.edge_type = EdgeType::Goto;
    let index = t.insert_node(goto_node);
    t.leaf_nodes.push(index);
  }

  while let Some(process_group) = t.get_next_queued() {
    process_node(&mut t, j, process_group)?;
  }

  t.non_trivial_root = unfulfilled_root.is_none();

  SherpaResult::Ok(t.clean())
}
/// Returns the follow set of the production as out-of-scope items
pub(crate) fn get_out_of_scope(
  g: &std::sync::Arc<GrammarStore>,
  production_id: ProductionId,
  existing: &BTreeSet<Item>,
  follow_set: bool,
) -> Items {
  let mut seen = BTreeSet::new();
  let mut productions = VecDeque::from_iter(vec![production_id]);
  let mut lr_items = BTreeSet::new();
  let existing = existing.as_set().to_state(ItemState::default());

  while let Some(production_id) = productions.pop_front() {
    if seen.insert(production_id) {
      let mut new_items = g
        .lr_items
        .get(&production_id)
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|i| !existing.contains(&(**i).to_empty_state()))
        .map(|i| i.increment().unwrap().to_state(ItemState::OUT_OF_SCOPE))
        .collect::<BTreeSet<_>>();

      for completed in new_items.as_vec().completed_items() {
        productions.push_back(completed.get_prod_id(g));
      }

      lr_items.append(&mut new_items);
    }
    if !follow_set {
      break;
    }
  }

  lr_items.to_vec()
}
