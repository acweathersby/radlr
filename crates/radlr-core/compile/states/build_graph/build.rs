use super::{
  flow::{
    get_kernel_items_from_peek_origin,
    handle_fork,
    handle_nonterminal_shift,
    handle_peek_complete_groups,
    handle_peek_incomplete_items,
    handle_regular_complete_groups,
    handle_regular_incomplete_items,
  },
  graph::*,
  items::{get_completed_item_artifacts, get_follow_internal, merge_occluding_token_items, FollowType},
  stack_vec::StackVec,
};
use crate::{compile::states::build_graph::graph::StateType, types::*, utils::hash_group_btree_iter};

use ParserClassification as PC;

pub(crate) type TransitionGroup = (u16, Vec<TransitionPair>);
pub(crate) type GroupedFirsts = OrderedMap<SymbolId, TransitionGroup>;

pub(crate) fn handle_kernel_items(
  gb: &mut ConcurrentGraphBuilder,
  pred: &SharedGraphNode,
  config: &ParserConfig,
) -> RadlrResult<()> {
  let mut groups = get_firsts(gb, pred, config)?;

  if pred.is_scanner() && groups.len() == 0 {
    StagedNode::new(gb)
      .sym((SymbolId::Default, 0).into())
      .parent(pred.clone())
      .ty(StateType::ScannerCompleteOOS)
      .include_with_goto_state()
      .commit(gb);

    gb.commit(false, Some(pred), config, true)?;
  }

  let have_lookahead = pred.kernel_items().len() > 1;

  if handle_fork(gb, pred) {
    gb.commit(false, Some(pred), config, true)?;
    return Ok(());
  }

  handle_cst_actions(gb, pred, &config)?;

  let max_completed_precedence = handle_completed_items(gb, pred, &config, &mut groups, pred.is_scanner())?;

  let groups = handle_scanner_items(max_completed_precedence, gb, pred, groups)?;

  handle_incomplete_items(gb, pred, &config, groups, PC { max_k: have_lookahead as u16, ..PC::default() })?;

  let update_gotos = handle_nonterminal_shift(gb, pred, &config)?;

  let states_queued = gb.commit(update_gotos, Some(pred), config, true)?;

  if pred.state_type().is_peek() && pred.state_type().peek_level() > 0 && states_queued == 0 {
    // Todo(anthony) : if peeking, determine if the peek has terminated in a
    // non-deterministic way. If so, produce a NonDeterministicPeek error.
    //panic!("Undeterministic PARSE");
    let root_data = pred.root_data.db_key;

    let root_node = gb.db().nonterm_sym(root_data);

    let mut peek_items = vec![];

    for peek_item in pred.kernel_items() {
      let (_, PeekGroup { items, .. }) = get_kernel_items_from_peek_origin(gb, pred, peek_item.origin);

      peek_items.extend(items.into_iter());
    }

    Err(RadlrError::StateConstructionError(crate::compile::states::build_states::StateConstructionError::NonDeterministicPeek(
      pred.get_root_shared(),
      Box::new(
        format!(
          "Testing Messagine. Peek has no successors! \n Error occurred in nonterminal {} \n Unresolved leaf items: \n {}",
          root_node.debug_string(gb.db()),
          peek_items.to_debug_string(gb.db(), "\n")
        )
        .into(),
      ),
    )))
  } else {
    Ok(())
  }
}

/// Insert non-terminal shift actions
fn handle_cst_actions(gb: &mut ConcurrentGraphBuilder, pred: &SharedGraphNode, config: &ParserConfig) -> RadlrResult<()> {
  if config.ALLOW_CST_NONTERM_SHIFT {
    let d = &gb.db_rc();
    let mode = pred.graph_type();
    for nonterm in pred.kernel_items().iter().filter(|i| i.is_nonterm(mode, d)) {
      StagedNode::new(gb)
        .parent(pred.clone())
        .add_kernel_items([nonterm.try_increment()].into_iter())
        .ty(StateType::CSTNodeAccept(nonterm.nonterm_index_at_sym(mode, d).unwrap()))
        .commit(gb);
    }
  }
  Ok(())
}

/// Iterate over each item's closure and collect the terminal transition symbols
/// of each item. The items are then classified by their terminal symbol.
/// Completed items are classified with the default symbol.
fn get_firsts(gb: &mut ConcurrentGraphBuilder, pred: &GraphNode, config: &ParserConfig) -> RadlrResult<GroupedFirsts> {
  let mut oos_scan_completed_tokens = OrderedSet::<PrecedentDBTerm>::new();
  let mut oos_scan_incompletes = OrderedSet::<PrecedentDBTerm>::new();

  let mut too_process_items = Vec::new();

  for item in pred.kernel_items() {
    match item.origin {
      Origin::__OOS_SCANNER_ROOT__(token) if config.ALLOW_LOOKAHEAD_SCANNERS => {
        if item.is_complete() {
          let mut follow = StackVec::<512, _>::new();
          let mut _default: StackVec<512, Item> = StackVec::<512, _>::new();

          get_follow_internal(gb, pred, *item, FollowType::AllItems, &mut follow, &mut _default);

          if follow.is_empty() {
            // No follow items indicates a completed OOS token.
            oos_scan_completed_tokens.insert(token);
            too_process_items.push(*item);
          } else {
            // Continue parsing with the OOS items scanned
            oos_scan_incompletes.insert(token);
            too_process_items.extend(follow.iter().cloned());
          }
        } else {
          if !item.is_initial() {
            oos_scan_incompletes.insert(token);
          }

          too_process_items.push(*item);
        }
      }
      _ => too_process_items.push(*item),
    }
  }

  let iter = too_process_items
    .iter()
    .filter(|i| match i.origin {
      Origin::TerminalGoal(key, prec) => !oos_scan_completed_tokens.contains(&(key, prec, false).into()),
      _ => true,
    })
    .flat_map(|k_i| {
      let basis = k_i.to_origin_state(pred.id());
      k_i
        .closure_iter_align_with_lane_split(basis, gb.db())
        .term_items_iter(pred.is_scanner(), gb.db())
        .map(|t_item| -> TransitionPair { (*k_i, t_item, pred.graph_type(), gb.db()).into() })
    });

  let mut groups: OrderedMap<SymbolId, (u16, Vec<TransitionPair>)> =
    hash_group_btree_iter::<Vec<_>, _, _, _, _>(iter, |_, first| first.sym)
      .into_iter()
      .map(|(s, g)| {
        (s, (g.iter().map(|f| f.kernel.origin.is_scanner_oos().then_some(0).unwrap_or(f.prec)).max().unwrap_or_default(), g))
      })
      .collect();

  if pred.is_scanner() && config.ALLOW_LOOKAHEAD_SCANNERS {
    if let Some(group) = groups.get_mut(&SymbolId::Default) {
      for pair in group.1.iter_mut() {
        let origin = pair.kernel.origin;
        if let Origin::TerminalGoal(tok, prec) = origin {
          let prec: PrecedentDBTerm = (tok, prec, false).into();
          if oos_scan_incompletes.contains(&prec) {
            pair.allow_assign = false;
          }
        }
      }
    }
  }

  RadlrResult::Ok(groups)
}

/// Removes transition pairs from groups that have lower precedences then the
/// group max or the max precedence of completed items, and also merges groups
/// that have occluding symbols symbols
fn handle_scanner_items(
  max_completed_precedence: u16,
  _gb: &ConcurrentGraphBuilder,
  node: &SharedGraphNode,
  mut groups: GroupedFirsts,
) -> RadlrResult<GroupedFirsts> {
  if node.is_scanner() {
    for (_, (_, pair)) in &mut groups {
      if pair.iter().any(|p| !p.kernel.origin.is_scanner_oos()) {
        // Remove all oos items from group
        let filtered_items = pair.iter().filter_map(|i| (!i.kernel.origin.is_scanner_oos()).then_some(*i)).collect::<Vec<_>>();
        *pair = filtered_items;
      }
    }

    groups = groups
      .into_iter()
      .filter_map(|(s, (p, g))| {
        // Remove symbols in
        let outer_prec =
          (max_completed_precedence > CUSTOM_TOKEN_PRECEDENCE_BASELINE).then_some(max_completed_precedence).unwrap_or_default();
        let inner_prec: u16 = (p > CUSTOM_TOKEN_PRECEDENCE_BASELINE).then_some(p).unwrap_or_default();
        let prec = outer_prec.max(inner_prec);

        if s == SymbolId::Default {
          // Completed items are an automatic pass
          Some((s, (p, g)))
        } else {
          let g = g.into_iter().filter(|i| i.prec >= prec).collect::<Vec<_>>();
          if g.is_empty() {
            None
          } else {
            Some((s, (p, g)))
          }
        }
      })
      .collect();
    merge_occluding_token_items(groups.clone(), &mut groups, _gb.db());
  }

  Ok(groups)
}

fn handle_incomplete_items<'nt_set, 'db: 'nt_set>(
  gb: &mut ConcurrentGraphBuilder,
  node: &SharedGraphNode,
  config: &ParserConfig,
  groups: GroupedFirsts,
  classification: ParserClassification,
) -> RadlrResult<()> {
  for (sym, group) in groups {
    let ____is_scan____ = node.is_scanner();
    let prec_sym: PrecedentSymbol = (sym, group.0).into();

    match node.state_type() {
      StateType::Peek(level) => handle_peek_incomplete_items(gb, node, prec_sym, group, level),
      _REGULAR_ => handle_regular_incomplete_items(gb, node, config, prec_sym, group, classification),
    }?;
  }
  Ok(())
}

fn handle_completed_items(
  gb: &mut ConcurrentGraphBuilder,
  pred: &SharedGraphNode,
  config: &ParserConfig,
  groups: &mut GroupedFirsts,
  ____is_scan____: bool,
) -> RadlrResult<u16> {
  let mut max_precedence = 0;

  if let Some(completed) = groups.remove(&SymbolId::Default) {
    max_precedence = max_precedence.max(completed.0);

    let CompletedItemArtifacts { lookahead_pairs, .. } =
      get_completed_item_artifacts(gb, pred, completed.1.iter().map(|i| &i.kernel))?;

    if !lookahead_pairs.is_empty() {
      // Create reduce states for follow items that have not already been covered.
      let mut completed_groups: OrderedMap<SymbolId, Vec<TransitionPair>> =
        hash_group_btree_iter(lookahead_pairs.iter(), |_, fp| match fp.next.get_type(gb.db()) {
          //ItemType::Completed(_) => {
          //  unreachable!("Should be handled outside this path")
          //}
          ItemType::TokenNonTerminal(_, sym) if !____is_scan____ => sym,
          ItemType::Terminal(sym) => sym,
          _ => SymbolId::Undefined,
        });

      completed_groups.remove(&SymbolId::Undefined);

      for (sym, follow_pairs) in completed_groups {
        handle_completed_groups(gb, pred, config, groups, sym, follow_pairs)?;
      }
    }

    // If there is a single rule that is being reduced then we can create a default
    // state for tha rule Otherwise lookaheads are used to disambiguate the
    // completed items, and items that have no lookahead need to be
    // disambiguated dynamically.

    // TODO(anthony) - create the correct filter to identify the number of rules
    // that are being reduced (compare item indices.)
    let default: Lookaheads = if completed.1.iter().to_kernel().items_are_the_same_rule()
      || completed.1.iter().all(|i| i.kernel.origin.is_scanner_oos())
    {
      completed.1.clone()
    } else {
      lookahead_pairs.iter().filter(|i| i.is_eoi_complete()).cloned().collect()
    };

    if default.len() > 0 {
      handle_completed_groups(gb, pred, config, groups, SymbolId::Default, default)?;
    } else {
      #[cfg(debug_assertions)]
      debug_assert!(
        !lookahead_pairs.is_empty(),
        "No default reduce! {pred:?} {}",
        completed.1.iter().map(|c| c._debug_string_(gb.db())).collect::<Vec<_>>().join("\n")
      )
    }
  }

  RadlrResult::Ok(max_precedence)
}

pub(crate) fn handle_completed_groups(
  gb: &mut ConcurrentGraphBuilder,
  node: &SharedGraphNode,
  config: &ParserConfig,
  groups: &mut GroupedFirsts,
  sym: SymbolId,
  follow_pairs: Lookaheads,
) -> RadlrResult<()> {
  let ____is_scan____ = node.is_scanner();
  let prec_sym: PrecedentSymbol = (sym, follow_pairs.iter().max_precedence()).into();

  match node.state_type() {
    StateType::Peek(level) => handle_peek_complete_groups(gb, node, config, groups, prec_sym, follow_pairs, level),
    _REGULAR_ => handle_regular_complete_groups(gb, node, config, groups, prec_sym, follow_pairs),
  }
}
