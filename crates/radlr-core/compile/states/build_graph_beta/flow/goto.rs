use super::super::graph::*;
use crate::{
  compile::states::build_graph::graph::{GraphBuildState, Origin, StateType},
  types::*,
  utils::hash_group_btreemap,
};
use std::collections::{BTreeSet, VecDeque};

pub(crate) fn handle_nonterminal_shift(gb: &mut ConcurrentGraphBuilder, pred: &GraphNodeShared) -> RadlrResult<bool> {
  if pred.is_scanner() || !gb.config().ALLOW_LR || pred.state_type().currently_peeking() {
    return Ok(false);
  };

  let mode = pred.graph_type();
  let db = &gb.db_rc();
  let kernel_base: ItemSet = pred.kernel_items().iter().inscope_items();

  let indices = if pred.is_root() { kernel_base.iter().indices() } else { Default::default() };
  let indices = &indices; // Make a reference to allow its use within closures.

  let parent_id = pred.id();
  let origin = Origin::Goto(parent_id);

  let mut nterm_items = kernel_base.iter().nonterm_items::<ItemSet>(mode, db);
  nterm_items.extend(kernel_base.iter().filter(|i| !i.is_complete()).flat_map(|i| {
    let basis = i.to_origin(origin).to_origin_state(parent_id);
    let closure = i
      .closure_iter_align_with_lane_split(basis, db)
      .filter(move |i| i.is_nonterm(mode, db) && !indices.contains(&i.index()))
      .enumerate()
      .map(|(index, i)| (index > 0).then_some(i.as_goto_origin()).unwrap_or(i));
    closure
  }));

  let out_items = gb.get_pending_items();

  let is_at_root = parent_id.is_root();

  let out_items: ItemSet = if false && parent_id.is_root() {
    out_items
  } else {
    out_items.into_iter().filter(|i| i.origin_state == parent_id && (!kernel_base.contains(i) || i.is_initial())).collect()
  };

  if out_items.is_empty() {
    return Ok(false);
  }

  // Get all the nonterminal symbols that are shifted in the kernel
  let mut kernel_nterm_ids = kernel_base.iter().nonterm_items::<ItemSet>(mode, db).iter().nonterm_ids_at_index(mode, db);
  kernel_nterm_ids.extend(kernel_base.iter().nonterm_items::<ItemSet>(mode, db).iter().rule_nonterm_ids(db));

  // NonTerms that appear in to the the right side of the specifier in
  // used_nonterm_items.

  let filter_nterms = false;

  let used_nterm_items =
    if filter_nterms { get_used_nonterms(gb, pred, out_items, nterm_items, &kernel_base) } else { nterm_items };

  if used_nterm_items.is_empty() {
    return Ok(false);
  }

  gb.set_nonterm_items(pred.id().0 as u64, used_nterm_items.clone());

  let used_nterm_groups = hash_group_btreemap(used_nterm_items, |_, t| t.nonterm_index_at_sym(mode, db).unwrap_or_default());

  for (target_nonterm, items) in &used_nterm_groups {
    let are_shifting_a_goal_nonterm = is_at_root && pred.goal_items().iter().rule_nonterm_ids(db).contains(&target_nonterm);
    let contains_completed_kernel_items = items.iter().any(|i| kernel_base.contains(i) && i.is_penultimate());
    let contains_completed_items = items.iter().any(|i| i.is_penultimate());

    let mut incremented_items = items.iter().try_increment();
    let nterm_shift_type = StateType::NonTerminalShiftLoop;

    let should_include_oos = {
      let contains_left_recursive_items = items.iter().any(|i| i.rule_is_left_recursive(mode, db));
      kernel_nterm_ids.remove(&target_nonterm) && is_at_root && contains_left_recursive_items && !contains_completed_items
    };

    // if there is a path to complete a kernel item, then we need to inject oos
    // lookahead items to ensure that we are not ignoring local ambiguity
    if should_include_oos {
      let local_nonterms = incremented_items.iter().nonterm_ids_at_index(mode, db);
      // This state completes this NonTerminal, but there is also one or more items
      // that transitions on the goal non-terminal. The trick is determining
      // whether we should complete the non-terminal or allow further processing the
      // left recursive items. This is a classic shift reduce problem, except
      // the condition to reduce is dependent on external items that we have to pull
      // into this scope. So we dump all items that shift on this
      // non-terminal into this state. We call this Out-of-Scope items and are
      // only used to determine if we should perform a reduction or a
      // completion.

      // We only need OOS items if there are no completed items after the non-terminal
      // transition. This will handle the cases of left-recursion.
      let incremented_indices = incremented_items.iter().indices();
      let oos_items = ItemSet::from_iter(
        db.nonterm_follow_items(*target_nonterm)
          .filter_map(|i| i.increment())
          .filter(|i| {
            i.nonterm_index_at_sym(mode, db) == Some(*target_nonterm)
              && i.nonterm_index(db) != *target_nonterm
              && !local_nonterms.contains(&i.nonterm_index(db))
              && !incremented_indices.contains(&i.index)
          })
          .map(|i| i.to_origin(Origin::GoalCompleteOOS).to_origin_state(parent_id)),
      );

      incremented_items.extend(oos_items);
    }

    // A State following a goto point must either end with a return to that GOTO or
    // a completion of the gotos kernel items.

    let state = GraphNodeBuilder::new()
      .set_build_state(GraphBuildState::NormalGoto)
      .set_parent(pred.clone())
      .set_sym((target_nonterm.to_sym(), 0).into())
      .set_type(nterm_shift_type)
      .set_kernel_items(incremented_items.into_iter().map(|i| i))
      .commit(gb)?;

    if are_shifting_a_goal_nonterm && !contains_completed_kernel_items {
      GraphNodeBuilder::new()
        .set_build_state(GraphBuildState::Leaf)
        .set_parent(state)
        .set_sym((SymbolId::Default, 0).into())
        .set_type(StateType::NonTermCompleteOOS)
        .make_leaf()
        .commit(gb)?;
    }
  }

  // The remaining non-terminals are comprised of accept items for this state.
  for nonterm_id in kernel_nterm_ids {
    GraphNodeBuilder::new()
      .set_build_state(GraphBuildState::Leaf)
      .set_parent(pred.clone())
      .set_sym((nonterm_id.to_sym(), 0).into())
      .set_type(StateType::NonTerminalComplete)
      .make_leaf()
      .commit(gb);
  }

  increment_gotos(gb, pred);

  RadlrResult::Ok(true)
}

fn get_used_nonterms(
  gb: &ConcurrentGraphBuilder,
  node: &GraphNodeShared,
  out_items: BTreeSet<Item>,
  nterm_items: BTreeSet<Item>,
  kernel_base: &BTreeSet<Item>,
) -> BTreeSet<Item> {
  let db = gb.db();
  let mut used_nterm_items = ItemSet::new();
  let mut seen = OrderedSet::new();
  let mut queue = VecDeque::from_iter(out_items.iter().map(|i| i.nonterm_index(db)));

  while let Some(nterm) = queue.pop_front() {
    if seen.insert(nterm) {
      for item in nterm_items.iter().filter(|i| i.nonterm_index_at_sym(node.graph_type(), db).unwrap() == nterm) {
        used_nterm_items.insert(*item);
        if !kernel_base.contains(item) || item.is_initial() {
          queue.push_back(item.nonterm_index(db));
        }
      }
    }
  }

  used_nterm_items
}

fn increment_gotos(gb: &mut ConcurrentGraphBuilder, node: &GraphNodeShared) {
  let current_id = node.id();

  gb.iter_pending_states_mut(&mut |sb| {
    /*     let peek_items = sb
      .get_peek_resolve_items()
      .map(|i| i.map(|(i, PeekGroup { items, is_oos })| (i, *is_oos, items.clone())).collect::<Vec<_>>());
    if let Some(peek_resolve_items) = peek_items {
      let old_kernel_items = sb.state_ref().get_kernel_items().clone();
      let mut new_kernel_items: Items = Default::default();
      for (v, is_oos, items) in peek_resolve_items {
        let old_origin = Origin::Peek(v);
        let new_items =
          items.iter().map(|i| if i.origin_state.0 == current_id.0 { i.as_goto_origin() } else { i.increment_goto() });

        let origin = sb.set_peek_resolve_state(new_items, is_oos);

        new_kernel_items.extend(old_kernel_items.iter().filter(|i| i.origin == old_origin).map(|i| i.to_origin(origin)));
      }

      sb.set_kernel_items(new_kernel_items.into_iter());
    } else { */
    let items = sb
      .get_kernel_items()
      .iter()
      .map(|i| if i.origin_state.0 == current_id.0 { i.as_goto_origin() } else { i.increment_goto() })
      .collect::<Items>();

    sb.set_kernel_items_mut(items.into_iter());
    //  }
  });
}
