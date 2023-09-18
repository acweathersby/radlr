use super::{
  flow::{
    handle_nonterminal_shift,
    handle_peek_complete_groups,
    handle_peek_incomplete_items,
    handle_regular_complete_groups,
    handle_regular_incomplete_items,
  },
  graph::*,
  items::{get_completed_item_artifacts, merge_occluding_token_items},
};
use crate::{types::*, utils::hash_group_btree_iter};

use GraphBuildState::*;

pub(crate) type TransitionGroup<'db> = (u16, Vec<TransitionPair<'db>>);
pub(crate) type GroupedFirsts<'db> = OrderedMap<SymbolId, TransitionGroup<'db>>;

pub(crate) fn handle_kernel_items(gb: &mut GraphBuilder) -> SherpaResult<()> {
  let mut groups = get_firsts(gb)?;

  let max_precedence = handle_completed_items(gb, &mut groups)?;

  let groups = handle_scanner_items(max_precedence, gb, groups)?;

  handle_incomplete_items(gb, groups)?;

  handle_nonterminal_shift(gb)?;

  Ok(())
}

// Iterate over each item's closure and collect the terminal transition symbols
// of each item. The item's are then catagorized by these nonterminal symbols.
// Completed items are catagorized by the default symbol.
fn get_firsts<'db>(gb: &mut GraphBuilder<'db>) -> SherpaResult<GroupedFirsts<'db>> {
  let state = gb.current_state();
  let iter = state.get_kernel_items().iter().flat_map(|k_i| {
    let basis = k_i.to_origin_state(gb.current_state_id());
    k_i
      .closure_iter_align_with_lane_split(basis)
      .term_items_iter(gb.is_scanner())
      .map(|t_item| -> TransitionPair { (*k_i, t_item, gb.get_mode()).into() })
  });

  let groups = hash_group_btree_iter::<Vec<_>, _, _, _, _>(iter, |_, first| first.sym);

  let groups: OrderedMap<SymbolId, (u16, Vec<TransitionPair<'db>>)> =
    groups.into_iter().map(|(s, g)| (s, (g.iter().map(|f| f.prec).max().unwrap_or_default(), g))).collect();

  SherpaResult::Ok(groups)
}

fn handle_scanner_items<'db>(
  max_precedence: u16,
  gb: &GraphBuilder<'db>,
  mut groups: GroupedFirsts<'db>,
) -> SherpaResult<GroupedFirsts<'db>> {
  if gb.is_scanner() {
    if max_precedence > CUSTOM_TOKEN_PRECEDENCE_BASELINE {
      groups = groups
        .into_iter()
        .filter_map(|(s, (p, g))| {
          if s == SymbolId::Default {
            // Completed items are an automatic pass
            Some((s, (p, g)))
          } else {
            let g = g.into_iter().filter(|i| i.prec >= max_precedence).collect::<Vec<_>>();
            if g.is_empty() {
              None
            } else {
              Some((s, (p, g)))
            }
          }
        })
        .collect();
    }

    merge_occluding_token_items(groups.clone(), &mut groups);
  }

  Ok(groups)
}

fn handle_incomplete_items<'nt_set, 'db: 'nt_set>(gb: &mut GraphBuilder<'db>, groups: GroupedFirsts<'db>) -> SherpaResult<()> {
  for (sym, group) in groups {
    let ____is_scan____ = gb.is_scanner();
    let prec_sym: PrecedentSymbol = (sym, group.0).into();

    match gb.current_state().get_type() {
      StateType::Peek(level) => handle_peek_incomplete_items(gb, prec_sym, group, level),
      _REGULAR_ => handle_regular_incomplete_items(gb, prec_sym, group),
    }?;
  }
  Ok(())
}

fn handle_completed_items<'db>(gb: &mut GraphBuilder<'db>, groups: &mut GroupedFirsts<'db>) -> SherpaResult<u16> {
  let ____is_scan____ = gb.is_scanner();
  let mut max_precedence = 0;

  if let Some(completed) = groups.remove(&SymbolId::Default) {
    max_precedence = max_precedence.max(completed.0);

    let CompletedItemArtifacts { lookahead_pairs, .. } = get_completed_item_artifacts(gb, completed.1.iter().map(|i| &i.kernel))?;

    if !lookahead_pairs.is_empty() {
      // Create reduce states for follow items that have not already been covered.
      let mut completed_groups: OrderedMap<SymbolId, Vec<TransitionPair>> =
        hash_group_btree_iter(lookahead_pairs.iter(), |_, fp| match fp.next.get_type() {
          //ItemType::Completed(_) => {
          //  unreachable!("Should be handled outside this path")
          //}
          ItemType::TokenNonTerminal(_, sym) if !gb.is_scanner() => sym,
          ItemType::Terminal(sym) => sym,
          _ => SymbolId::Undefined,
        });

      completed_groups.remove(&SymbolId::Undefined);

      for (sym, follow_pairs) in completed_groups {
        handle_completed_groups(gb, groups, sym, follow_pairs)?;
      }
    }

    // If there is a single rule that is being reduced then we can create a default
    // state for tha rule Otherwise lookaheads are used to disambiguate the
    // completed items, and items that have no lookahead need to be
    // disambiguated dynamically.

    // TODO(anthony) - create the correct filter to identify the number of rules
    // that are being reduced (compare item indices.)
    let default: Lookaheads = if completed.1.iter().to_kernel().items_are_the_same_rule() {
      completed.1
    } else {
      lookahead_pairs.iter().filter(|i| i.is_complete()).cloned().collect()
    };

    if default.len() > 0 {
      handle_completed_groups(gb, groups, SymbolId::Default, default)?;
    } else {
      debug_assert!(!lookahead_pairs.is_empty())
    }
  }

  SherpaResult::Ok(max_precedence)
}

pub(crate) fn handle_completed_groups<'db>(
  gb: &mut GraphBuilder<'db>,
  groups: &mut GroupedFirsts<'db>,
  sym: SymbolId,
  follow_pairs: Lookaheads<'db>,
) -> SherpaResult<()> {
  let ____is_scan____ = gb.is_scanner();
  let prec_sym: PrecedentSymbol = (sym, follow_pairs.iter().max_precedence()).into();

  match gb.current_state().get_type() {
    StateType::Peek(_) => handle_peek_complete_groups(gb, groups, prec_sym, follow_pairs),
    _REGULAR_ => handle_regular_complete_groups(gb, groups, prec_sym, follow_pairs),
  }
}
