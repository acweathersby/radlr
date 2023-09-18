use super::super::graph::*;
use crate::types::*;

use GraphBuildState::*;
use StateType::*;

pub struct CreateCallResult<'db> {
  /// `true` if the state is a KernelCall
  pub is_kernel:         bool,
  /// The new state that will perform the call
  pub state_id:          StateId,
  /// A list of items from the parent closure that transition on the called
  /// non-terminal.
  pub _transition_items: Items<'db>,
}

/// Attempts to make a "call" states, which jumps to the root of another
/// non-terminal parse graph. Returns an optional tuple:
/// (
///   is_ker
/// )
pub(crate) fn create_call<'a, 'db: 'a, T: TransitionPairRefIter<'a, 'db> + Clone>(
  gb: &mut GraphBuilder<'db>,
  group: T,
  sym: PrecedentSymbol,
) -> Option<CreateCallResult<'db>> {
  let ____is_scan____ = gb.is_scanner();
  let ____allow_rd____: bool = gb.config.ALLOW_RECURSIVE_DESCENT || ____is_scan____;
  let ____allow_ra____: bool = gb.config.ALLOW_LR || ____is_scan____;

  if
  /* TODO(anthony) remove this after scan peek is implemented >>> */
  ____is_scan____ /* <<< */ || !____allow_rd____ || group.clone().any(|i| i.is_kernel_terminal()) {
    return None;
  };

  // if all kernels are on same nonterminal symbol then we can do a call, provided
  // the nonterminal is not left recursive.

  let kernel_symbol = group.clone().kernel_nonterm_sym(gb.get_mode());

  if kernel_symbol.len() == 1 {
    if let Some(Some(nonterm)) = kernel_symbol.first() {
      match gb.db.nonterm_recursion_type(*nonterm) {
        RecursionType::LeftRecursive | RecursionType::LeftRightRecursive => {
          // Can't make a call on a left recursive non-terminal.
        }
        _ => {
          // Create call on the kernel items.
          let items = group.to_kernel().to_vec();
          gb.set_classification(ParserClassification { calls_present: true, ..Default::default() });

          return Some(CreateCallResult {
            is_kernel:         true,
            state_id:          gb
              .create_state(Normal, sym, KernelCall(*nonterm), Some(items.try_increment().iter().cloned()))
              .to_state(),
            _transition_items: items,
          });
        }
      }
    }
  }

  // We'll need to climb the closure graph to find the highest mutual non-terminal
  // that is not left recursive. This is only allowed if the system allows LR
  if !____allow_ra____ {
    return None;
  };

  if let Some((nonterm, items)) = climb_nonterms(gb, group) {
    gb.set_classification(ParserClassification { calls_present: true, ..Default::default() });

    return Some(CreateCallResult {
      is_kernel:         false,
      state_id:          gb
        .create_state(Normal, sym, InternalCall(nonterm), Some(items.try_increment().iter().cloned()))
        .to_state(),
      _transition_items: items,
    });
  } else {
    None
  }
}

fn climb_nonterms<'a, 'db: 'a, T: TransitionPairRefIter<'a, 'db> + Clone>(
  gb: &mut GraphBuilder<'db>,
  group: T,
) -> Option<(DBNonTermKey, Vec<Item<'db>>)> {
  let db = gb.graph().get_db();

  if all_items_come_from_same_nonterminal_call(group.clone()) {
    let nterm = unsafe { group.clone().next().unwrap_unchecked() }.next.nonterm_index();

    if matches!(db.nonterm_recursion_type(nterm), RecursionType::LeftRecursive | RecursionType::LeftRightRecursive) {
      return None;
    };

    let climbed_firsts = group
      .clone()
      .flat_map(|p| {
        p.kernel
          .closure_iter()
          .filter(|i| match i.nonterm_index_at_sym(gb.get_mode()) {
            Some(id) => id == nterm && i.nonterm_index() != nterm,
            _ => false,
          })
          .map(|i| -> TransitionPair { (p.kernel, i.align(&p.next), gb.get_mode()).into() })
      })
      .collect::<Vec<_>>();

    // There may be a superior candidate. evaluate that.
    if let Some(candidate) = climb_nonterms(gb, climbed_firsts.iter()) {
      return Some(candidate);
    }

    Some((nterm, climbed_firsts.iter().to_next().cloned().collect()))
  } else {
    None
  }
}

pub(super) fn all_items_come_from_same_nonterminal_call<'a, 'db: 'a, T: TransitionPairRefIter<'a, 'db> + Clone>(
  group: T,
) -> bool {
  group.clone().all(|i| i.next.is_initial()) && group.map(|i| i.next.nonterm_index()).collect::<Set<_>>().len() == 1
}
