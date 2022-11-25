use super::utils::*;
use std::collections::btree_map;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;
use std::path::PathBuf;
use std::process::id;
use std::rc::Rc;
use std::sync::Arc;
use std::vec;

use crate::debug::debug_items;
use crate::grammar::create_closure;
use crate::grammar::get_closure_cached;
use crate::grammar::get_production_start_items;
use crate::grammar::hash_id_value_u64;
use crate::types::BlameColor;
use crate::types::GrammarId;
use crate::types::GrammarStore;
use crate::types::HCResult;
use crate::types::Item;
use crate::types::ItemState;
use crate::types::OriginData;
use crate::types::ProductionId;
use crate::types::SymbolID;
use crate::types::TPackResults;
use crate::types::TransitionGraphNode as TGN;
use crate::types::TransitionGraphNodeId;
use crate::types::TransitionMode;
use crate::types::TransitionPack as TPack;
use crate::types::TransitionStateType as TST;

/// Constructs an initial transition graph that parses a production
/// using a recursive descent strategy. Productions that are ambiguous
/// or are left recursive cannot be parsed, so this tries to do its
/// best to define a parse path for a production before we have to
/// resort to LR and Forking based parse strategies.

pub fn construct_recursive_descent(
  g: Arc<GrammarStore>,
  is_scanner: bool,
  starts: &[Item],
  root_ids: BTreeSet<ProductionId>,
) -> TPackResults {
  let start_items = apply_state_info(starts);

  let mut t =
    TPack::new(g.clone(), TransitionMode::RecursiveDescent, is_scanner, &start_items, root_ids);

  add_item_closures(&start_items, &mut t);

  let root_index = t.insert_node(TGN::new(&t, SymbolID::Undefined, usize::MAX, start_items));

  t.get_node_mut(root_index).set_type(TST::I_DESCENT_START);

  t.queue_node(root_index);

  while let Some(node_id) = t.get_next_queued() {
    process_node(&mut t, node_id, node_id, node_id != 0);
  }

  t.clean()
}

fn apply_state_info(starts: &[Item]) -> Vec<Item> {
  let start_items = starts
    .iter()
    .enumerate()
    .map(|(i, item)| item.to_state(ItemState::new(i as u32, 0)))
    .collect::<Vec<_>>();
  start_items
}

pub fn construct_goto(
  g: Arc<GrammarStore>,
  is_scanner: bool,
  starts: &[Item],
  goto_seeds: &[Item],
  root_ids: BTreeSet<ProductionId>,
) -> TPackResults {
  let start_items = apply_state_info(starts);

  let mut t = TPack::new(g.clone(), TransitionMode::GoTo, is_scanner, &start_items, root_ids);

  // Get closures of all items that could transition on the same production.

  let global_closure = Rc::new(Box::<Vec<Item>>::new(if t.is_scanner {
    vec![]
  } else {
    get_follow_closure(&g, goto_seeds, &t.root_prod_ids)
  }));

  let mut parent = TGN::new(&t, SymbolID::Undefined, TGN::OrphanIndex, goto_seeds.to_vec());

  parent.set_type(TST::I_GOTO_START);

  let parent_index = t.insert_node(parent);

  let mut unfulfilled_root = Some(*t.root_prod_ids.first().unwrap());

  for (production_id, group) in get_goto_starts(&g, &start_items, goto_seeds) {
    t.goto_scoped_closure = Some(global_closure.clone());

    let have_root_production = t.root_prod_ids.contains(&production_id);
    let mut have_end_items = false;

    let mut items: Vec<Item> = group
      .iter()
      .map(|i| {
        let stated_item = i.increment().unwrap();

        let stated_item = if stated_item.at_end() {
          have_end_items = true;
          stated_item.to_state(ItemState::GOTO_END_GOAL_STATE)
        } else {
          t.scoped_closures.push(get_closure_cached(&stated_item, &g).clone());
          stated_item.to_state(ItemState::new((t.scoped_closures.len() - 1) as u32, 0))
        };

        stated_item
      })
      .collect();

    let mut goto_node = TGN::new(
      &t,
      SymbolID::Production(production_id, GrammarId::default()),
      parent_index,
      items.clone(),
    );
    goto_node.set_type(TST::O_GOTO);

    if have_root_production || (group.len() > 1 && have_end_items) {
      t.out_of_scope_closure =
        Some(g.lr_items.iter().flat_map(|(_, i)| i).cloned().collect::<Vec<Item>>());

      if have_root_production {
        unfulfilled_root = None;
        let mut reducible: Vec<Item> = g
          .lr_items
          .get(&production_id)
          .unwrap_or(&Vec::new())
          .iter()
          .filter(|i| !group.iter().any(|g| *g == **i))
          .map(|i| i.increment().unwrap().to_state(ItemState::GOTO_ROOT_END_GOAL_STATE))
          .collect();

        goto_node.items.append(&mut reducible.clone());
        items.append(&mut reducible);
      }

      let node_id = t.insert_node(goto_node);
      create_peek(&mut t, node_id, items);
    } else {
      let node_index = t.insert_node(goto_node);
      process_node(&mut t, node_index, node_index, false);
    }
  }

  // If the root production is not covered in the goto branches
  // then create a new node that serves as an accepting state
  // if the active production id is the root.

  if let Some(production_id) = unfulfilled_root {
    let mut goto_node =
      TGN::new(&t, SymbolID::Production(production_id, GrammarId::default()), parent_index, vec![]);
    goto_node.set_type(TST::O_GOTO_ROOT_ACCEPT | TST::I_PASS);
    let index = t.insert_node(goto_node);
    t.leaf_nodes.push(index);
  }

  while let Some(node_index) = t.get_next_queued() {
    process_node(&mut t, node_index, node_index, node_index != 0);
  }

  t.non_trivial_root = unfulfilled_root.is_none();

  t.clean()
}

pub fn get_goto_starts(
  g: &GrammarStore,
  starts: &Vec<Item>,
  seeds: &[Item],
) -> Vec<(ProductionId, BTreeSet<Item>)> {
  type GotoMap = BTreeMap<ProductionId, BTreeSet<Item>>;

  let mut output_items = GotoMap::new();

  for item in seeds.iter() {
    output_items
      .entry(item.get_production_id_at_sym(g))
      .and_modify(|e| {
        e.insert(*item);
      })
      .or_insert_with(|| BTreeSet::from_iter(vec![*item]));
  }

  output_items.into_iter().collect()
}

pub fn get_follow_closure(
  g: &GrammarStore,
  gotos: &[Item],
  root_ids: &BTreeSet<ProductionId>,
) -> Vec<Item> {
  let mut pending_prods = VecDeque::<ProductionId>::new();
  let mut seen_prods = BTreeSet::<ProductionId>::new();

  for prod_id in root_ids {
    pending_prods.push_back(*prod_id);
  }

  let mut output = BTreeSet::new();

  while let Some(production_id) = pending_prods.pop_front() {
    if !seen_prods.insert(production_id) {
      continue;
    }

    let items: Vec<Item> = g
      .lr_items
      .get(&production_id)
      .unwrap_or(&Vec::new())
      .iter()
      .map(|i| i.increment().unwrap())
      .collect();

    for item in items {
      if item.is_end() {
        pending_prods.push_back(item.get_prod_id(g));
      }

      output.insert(item.decrement().unwrap());
    }

    seen_prods.insert(production_id);
  }

  output.into_iter().collect()
}

fn process_node(t: &mut TPack, node_id: usize, par_id: usize, allow_increment: bool) -> Vec<usize> {
  let node = t.get_node(node_id).clone();
  let mut items: Vec<Item> = node.items.clone();
  let grammar = t.g.clone();
  let g = &grammar;

  if allow_increment {
    items = items
      .into_iter()
      .map(|i| {
        if i.at_end() {
          i
        } else {
          let item_next = i.increment().unwrap();
          let item_link = t.get_closure_link(&i);
          t.set_closure_link(item_next, item_link);
          item_next
        }
      })
      .collect();
  }

  let n_term: Vec<Item> = items.iter().filter(|i| i.is_term(&t.g)).cloned().collect();
  let n_nonterm: Vec<Item> = items.iter().filter(|i| i.is_nonterm(&t.g)).cloned().collect();
  let n_end: Vec<Item> = items.iter().filter(|i| i.is_end()).cloned().collect();

  // If the depth is 0 and we are not trying to produce
  // a scanner state tree, then we can attempt to find the
  // highest common production between non-term items. That
  // is the production derivative that is non-term that all
  // or a subset of non-term items share.

  if !n_nonterm.is_empty() {
    let production_ids = BTreeSet::<ProductionId>::from_iter(
      n_nonterm.iter().map(|i| i.get_production_id_at_sym(&t.g)),
    );

    if n_term.is_empty()
      && n_end.is_empty()
      && production_ids.len() == 1
      && (node.shifts > 0 || non_recursive(&n_nonterm, &t.root_prod_ids, &t.g))
    {
      create_production_call(t, *production_ids.iter().next().unwrap(), n_nonterm, par_id)
    } else {
      create_peek(t, par_id, items)
    }
  } else if n_end.len() == 1 && n_term.is_empty() {
    // A single end item
    process_end_item(t, n_end[0], par_id)
  } else if n_end.is_empty() && n_term.len() == 1 {
    // A single terminal item
    process_terminal_node(t, &vec![n_term[0]], par_id)
  } else if n_end.is_empty()
    // The terminal symbol is identical in all items.
    && n_term.iter().map(|i| i.get_symbol(&t.g)).collect::<BTreeSet<_>>().len() == 1
  {
    process_terminal_node(t, &n_term, par_id)
  } else {
    // else if n_end.is_empty() && n_term.len() > 1  {
    // Multiple terminal items
    create_peek(t, par_id, items)
  }
}

fn create_peek(t: &mut TPack, par_id: usize, items: Vec<Item>) -> Vec<usize> {
  let mut cached_depth = 0;
  let mut peek_nodes = vec![];

  {
    let parent = t.get_node_mut(par_id);
    parent.set_type(TST::I_PEEK_ORIGIN);
    cached_depth = parent.shifts;
  }

  let goals: Vec<usize> = hash_group_vec(items.clone(), |ind, i| {
    if i.at_end() {
      format!("at_end_{}", ind)
    } else {
      format!("{:?} {}", i.get_symbol(&t.g), i.is_out_of_scope())
    }
  })
  .iter()
  .enumerate()
  .map(|(i, items)| {
    let item = items[0];

    let mut goal_node = TGN::new(t, item.get_symbol(&t.g).to_owned(), usize::MAX, items.to_vec());

    goal_node.shifts = cached_depth;

    if item.is_out_of_scope() {
      goal_node.trans_type |= TST::I_OUT_OF_SCOPE;
      if item.is_goto_end_origin() {
        goal_node.trans_type |= TST::I_GOTO_END;
      } else {
        goal_node.trans_type |= TST::I_GOTO_ROOT;
      }
    } else if t.mode == TransitionMode::GoTo && item.at_end() {
      goal_node.trans_type |= TST::I_COMPLETE;
    }

    t.insert_node(goal_node)
  })
  .collect();
  for (i, goal_index) in goals.iter().cloned().enumerate() {
    let items = t
      .get_node(goal_index)
      .items
      .to_owned()
      .iter()
      .map(|i| LinkedItem { link: t.get_closure_link(i), item: *i })
      .collect::<Vec<_>>();

    for mut node in create_term_nodes_from_items(&create_term_item_closure(t, &items), t, par_id) {
      node.goal = goal_index;

      node.shifts = t.get_node(goal_index).shifts;

      node.peek_shifts = 0;

      peek_nodes.push(t.insert_node(node));
    }
  }

  let mut leaves = vec![];

  disambiguate(t, peek_nodes, &mut leaves, 0);

  let resolved_leaves = process_peek_leaves(t, leaves);

  // All goal nodes can be recycled, as copy operations where used to
  // insert goal nodes as children of peek leaves
  for goal in goals {
    t.drop_node(&goal);
  }

  t.clear_peek_data();

  resolved_leaves
}

fn create_term_item_closure(t: &mut TPack, items: &Vec<LinkedItem>) -> Vec<LinkedItem> {
  let mut term_items = BTreeSet::new();
  for linked in items {
    if linked.item.is_term(&t.g) || linked.item.is_end() {
      term_items.insert(*linked);
    } else {
      for closure_item in create_closure(&[linked.item], &t.g) {
        let closure_item = closure_item.to_origin(linked.item.get_origin());
        if !closure_item.is_nonterm(&t.g) {
          let closure_item = closure_item.to_state(linked.item.get_state());
          term_items.insert(LinkedItem { link: linked.item, item: closure_item });
        }
      }
    }
  }
  term_items.into_iter().collect::<Vec<_>>()
}

fn create_term_nodes_from_items(items: &Vec<LinkedItem>, t: &mut TPack, par_id: usize) -> Vec<TGN> {
  let mut term_nodes = vec![];

  for linked in items {
    t.set_closure_link(linked.item, linked.link);
    let node = TGN::new(t, linked.item.get_symbol(&t.g), par_id, vec![linked.item]);
    term_nodes.push(node);
  }

  term_nodes
}

#[inline]
fn disambiguate(t: &mut TPack, node_ids: Vec<usize>, leaves: &mut Vec<usize>, peek_depth: u32) {
  let mut term_nodes = vec![];
  let mut end_nodes = vec![];
  let mut exclusive_ended = false;

  // We must first complete end-items and generate new
  // nodes that arise from the completion of a production.

  for node_index in node_ids {
    let node = t.get_node(node_index);
    let origin = node.items[0].get_origin();
    let item = node.items[0];
    let goal = node.goal;
    let parent_index = node.parent;

    if !item.is_end() {
      term_nodes.push(node_index)
    } else {
      let (mut terms, mut final_ends) =
        { get_continue_nodes(t, item, parent_index, peek_depth, goal) };

      exclusive_ended = match origin {
        OriginData::Symbol(sym) if sym.is_exclusive() => true,
        _ => exclusive_ended,
      };

      if terms.is_empty() && final_ends.is_empty() {
        end_nodes.push(node_index);
      } else {
        term_nodes.append(&mut terms);
        end_nodes.append(&mut final_ends);
        t.drop_node(&node_index);
      }
    }
  }

  match end_nodes.len() {
    0 => {}
    1 => {
      set_transition_type(t, end_nodes[0], peek_depth);
      leaves.push(end_nodes[0]);
    }
    _ => {
      if get_goals(&end_nodes, t).len() == 1 || all_nodes_are_out_of_scope(&end_nodes, t) {
        for end_node in &end_nodes[1..] {
          t.drop_node(end_node);
        }
        set_transition_type(t, end_nodes[0], peek_depth);
        leaves.push(end_nodes[0]);
      } else {
        handle_unresolved_nodes(t, end_nodes, leaves);
      }
    }
  }

  let mut nodes_to_be_dropped: BTreeSet<usize> = BTreeSet::new();

  let term_nodes = if exclusive_ended {
    for node_id in term_nodes {
      nodes_to_be_dropped.insert(node_id);
    }
    vec![]
  } else {
    term_nodes
  };

  let mut groups = hash_group_vec(term_nodes, |_, n| (t.get_node(n).edge_symbol));
  let mut next_peek_groups = vec![];
  let mut primary_nodes: BTreeSet<usize> = BTreeSet::new();

  merge_occluding_groups(t, &mut groups);

  for mut group in groups.iter_mut() {
    let prime_node_index = group[0];

    set_transition_type(t, prime_node_index, peek_depth);

    if group.iter().any(|i| t.get_node(*i).is_out_of_scope()) {
      let first = t.get_node(group[0]);
      let term_is_generic = matches!(
        first.edge_symbol,
        SymbolID::GenericNumber | SymbolID::GenericIdentifier | SymbolID::GenericSymbol
      );

      if !first.is_out_of_scope() {
        // Remove nodes that have items that alias in-scope items
        let inscope_items = group
          .iter()
          .filter_map(|node_id| {
            let node = t.get_node(*node_id);
            if (!node.is_out_of_scope()) {
              Some(node.items.iter().map(|i| i.to_zero_state()).collect::<Vec<_>>())
            } else {
              None
            }
          })
          .flatten()
          .collect::<BTreeSet<_>>();

        // Remove outscope items if the symbol of the merged items and
        // original nodes is generic, or if the outscope item occludes
        // an existing inscope item.
        nodes_to_be_dropped.append(
          &mut group
            .drain_filter(|i| {
              let node = t.get_node(*i);
              !(node.is_out_of_scope()
                && (inscope_items.contains(&node.items[0].to_zero_state())
                  || (term_is_generic && node.edge_symbol == first.edge_symbol)))
            })
            .collect(),
        );
      }
    }

    let goals = get_goals(group, t);

    if goals.len() > 1 && group.len() > 1 {
      let mut peek_transition_group = vec![];
      for node_index in group.iter().cloned() {
        let transition_on_skipped_symbol = t.get_node(node_index).is(TST::I_SKIPPED_COLLISION);
        let goal = t.get_node(node_index).goal;

        for mut node in create_term_nodes_from_items(
          &create_term_item_closure(
            t,
            &t.get_node(node_index)
              .items
              .clone()
              .into_iter()
              .map(|i| {
                let link = t.get_closure_link(&i);
                if transition_on_skipped_symbol {
                  LinkedItem { link, item: i }
                } else {
                  LinkedItem { link, item: i.increment().unwrap() }
                }
              })
              .collect::<Vec<_>>(),
          ),
          t,
          prime_node_index,
        ) {
          node.goal = goal;
          node.peek_shifts = (peek_depth + 1) as i32;
          peek_transition_group.push(t.insert_node(node));
        }
      }

      if peek_transition_group.is_empty() {
        if get_goals(group, t).iter().all(|i| t.get_node(*i).is(TST::I_OUT_OF_SCOPE)) {
          leaves.push(prime_node_index);
        } else {
          panic!("Invalid state, unable to continue disambiguating \n");
        }
      } else {
        next_peek_groups.push(peek_transition_group);
      }
    } else {
      leaves.push(prime_node_index);
    }

    primary_nodes.insert(prime_node_index);

    for node_index in group.iter() {
      if *node_index != prime_node_index {
        let items = t.get_node(*node_index).items.clone();
        nodes_to_be_dropped.insert(*node_index);
        t.get_node_mut(prime_node_index).items.append(&mut items.clone());
      }
    }
  }

  for drop_node in nodes_to_be_dropped {
    if !primary_nodes.contains(&drop_node) {
      t.drop_node(&drop_node);
    }
  }

  for peek_group in next_peek_groups {
    if !peek_group.is_empty() {
      if all_nodes_are_out_of_scope(&peek_group, t) {
        for end_node in &peek_group[1..] {
          t.drop_node(end_node);
        }

        set_transition_type(t, peek_group[0], peek_depth);
        leaves.push(peek_group[0]);
      } else if handle_shift_reduce_conflicts(t, &peek_group, leaves) {
        continue;
      } else if groups_are_aliased(&peek_group, t) || group_is_repeated(&peek_group, t) {
        handle_unresolved_nodes(t, peek_group, leaves);
      } else {
        disambiguate(t, peek_group, leaves, peek_depth + 1);
      }
    }
  }
}

fn all_nodes_are_out_of_scope(peek_group: &Vec<usize>, t: &mut TPack) -> bool {
  peek_group.iter().all(|i| t.get_node(*i).items.iter().all(|i| i.is_out_of_scope()))
}

/// Places goal nodes onto the tips of peek leaf nodes to complete
/// a peek parse path. We then resume constructing the transition
/// graph from the goal nodes onward.
///
/// ### Notes
/// - #### Graph Structure
/// Multiple peek leaves may resolve to a single goal, so we make sure
/// we only continue construction of the transition graph using a
/// single goal node by allowing that node to point to multiple parent
/// nodes by way of the `proxy_parent` vector.
///
/// - #### Scanner Nodes
/// Scanners do not peek, but instead greedily match characters
/// as they traverse the disambiguating path, yielding the lengths of
/// disambiguated tokens. Thus, when we reach leaf peek nodes of a
/// scanner scope, we simply complete any outstanding items and yield
/// goto productions to be resolved within the goto states. The
/// transitions of all parent nodes are reconfigured as
/// ASSERT_SHIFT.
#[inline]
fn process_peek_leaves(t: &mut TPack, leaves: Vec<usize>) -> Vec<usize> {
  let mut resolved_leaves = Vec::<usize>::new();
  let g = &t.g.clone();

  let mut leaves = if t.is_scanner {
    let mut out_leaves = vec![];
    for node_index in leaves {
      // Instead of resetting our position back to
      // the goal item, we simply continue parsing
      // from whatever position we are at.

      if t.get_node_mut(node_index).peek_shifts <= 0 {
        // Allows recursive descent style calls to be made.
        out_leaves.push(node_index);
      } else {
        // Process all proceeding nodes and remove all peeks,
        // replacing with shifts.

        let mut iter_index = node_index;

        while !t.get_node(iter_index).is(TST::I_PEEK_ORIGIN) {
          let node = t.get_node_mut(iter_index);
          node.unset_type(TST::O_PEEK);
          node.set_type(TST::I_SHIFT);
          iter_index = node.parent;
        }

        // Proceed to find the lowest transition action (the original
        // non-term origin) for the current state. For nodes
        // that have more than one item this will not apply.

        let mut items = t.get_node_mut(node_index).items.clone();

        let mut new_items = items
          .into_iter()
          .map(|mut item| {
            while item.at_start() {
              // get the originating item.
              item = t.get_previous_link(&item);
            }
            item
          })
          .collect::<Vec<_>>();

        let mut node = t.get_node_mut(node_index);

        if node.items.len() == 1 {
          node.items = new_items;
        }

        if node.items.iter().any(|i| i.at_end()) {
          node.unset_type(TST::I_SHIFT);
          process_node(t, node_index, node_index, true);
        } else if node.items.iter().any(|i| i.is_nonterm(g)) {
          node.unset_type(TST::I_SHIFT);
          process_node(t, node_index, node_index, false);
        } else {
          node.set_type(TST::I_SHIFT);
          process_node(t, node_index, node_index, true);
        }
      }
    }
    out_leaves
  } else {
    leaves
  };

  // Handle root out of scope items independently from all other nodes
  // since their action is fail
  let mut root_out_of_scope =
    leaves.drain_filter(|leaf| t.get_goal(*leaf).is(TST::I_GOTO_ROOT)).collect::<Vec<_>>();

  if !root_out_of_scope.is_empty() {
    if !nodes_contain_end_items(&root_out_of_scope, t) {
      let parent = t.get_node(root_out_of_scope[0]).parent;

      let mut default_nod = TGN::new(t, SymbolID::Default, parent, vec![]);

      default_nod.parent = parent;

      root_out_of_scope.push(t.insert_node(default_nod));
    }

    for node_index in root_out_of_scope {
      t.get_node_mut(node_index).set_type(TST::I_FAIL | TST::I_OUT_OF_SCOPE);
      t.leaf_nodes.push(node_index)
    }
  }

  for mut peek_leaf_group in hash_group_vec(leaves, |_, leaf| t.get_node(leaf).goal).iter().cloned()
  {
    let primary_peek_parent_index = peek_leaf_group[0];
    let prime_node = t.get_node(primary_peek_parent_index).to_owned();
    let goal_index = prime_node.goal;
    let goal_node = t.get_node(goal_index).clone();

    if goal_node.is(TST::I_GOTO_END) {
      if !nodes_contain_end_items(&peek_leaf_group, t) {
        let parent = t.get_node(peek_leaf_group[0]).parent;
        let mut default_nod = TGN::new(t, SymbolID::Default, parent, vec![]);
        default_nod.parent = parent;
        peek_leaf_group.push(t.insert_node(default_nod));
      }
    }
    // Use the goal node as a proxy to generate child nodes that
    // are then linked to the current peek leaf nodes.

    let primary_parent = peek_leaf_group[0];
    let proxy_parents = peek_leaf_group[1..].to_owned();
    let have_proxy_parents = !proxy_parents.is_empty();

    for child_index in process_node(t, goal_index, primary_parent, false) {
      let mut child_node = t.get_node_mut(child_index);

      child_node.parent = primary_peek_parent_index;

      if have_proxy_parents {
        child_node.proxy_parents.append(&mut proxy_parents.to_owned());
      }

      if child_node.prod_sym.is_some() {
        child_node.edge_symbol = prime_node.edge_symbol
      }

      resolved_leaves.push(child_index);
    }

    // Note: Remember all goal nodes are DROPPED at the
    // end of the peek resolution process
  }

  resolved_leaves
}

fn nodes_contain_end_items(nodes: &[usize], t: &mut TPack) -> bool {
  nodes.iter().any(|node_id| t.get_node(*node_id).items[0].is_end())
}

fn get_continue_nodes(
  t: &mut TPack,
  end_item: Item,
  parent_index: usize,
  peek_depth: u32,
  goal: usize,
) -> (Vec<usize>, Vec<usize>) {
  let mut term_nodes = vec![];
  let mut final_nodes = vec![];
  let mut need_to_prune = false;

  let TermAndEndItemGroups { term_items, mut end_items } = scan_items(t, &end_item, None);

  if end_item.is_out_of_scope() {
    end_items = vec![];
  }

  let scan_items = if end_item.is_out_of_scope() && peek_depth < 1 {
    // Remove item pairs that are expected to show up
    // within the "in-scope" nodes.
    create_term_item_closure(t, &term_items)
      .into_iter()
      .filter(|l| {
        let item = l.item.to_zero_state();
        let bool_val = !(t.starts.contains(&item)
          || (item.is_nonterm(&t.g)
            && (t.root_prod_ids.contains(&item.get_prod_id(&t.g))
              || t.root_prod_ids.contains(&item.get_production_id_at_sym(&t.g)))));
        bool_val
      })
      .collect::<Vec<_>>()
  } else {
    create_term_item_closure(t, &term_items)
  };

  for mut term_node in create_term_nodes_from_items(&scan_items, t, parent_index) {
    term_node.goal = goal;

    term_node.peek_shifts = peek_depth as i32;

    term_nodes.push(t.insert_node(term_node));
  }

  for mut final_node in create_term_nodes_from_items(&end_items, t, parent_index) {
    final_node.goal = goal;

    final_node.set_type(TST::I_END | TST::O_ASSERT);

    final_node.peek_shifts = peek_depth as i32;

    let node_index = t.insert_node(final_node);

    final_nodes.push(node_index);
  }

  (term_nodes, final_nodes)
}

fn group_is_repeated(peek_group: &[usize], t: &mut TPack) -> bool {
  let group_id = peek_group
    .iter()
    .flat_map(|i| {
      let node = t.get_node(*i);
      node.items.iter().map(|i| i.to_zero_state().to_hash())
    })
    .collect::<BTreeSet<_>>();

  let ids = group_id.iter().collect::<Vec<_>>();

  let hash_id = hash_id_value_u64(ids.clone());

  !t.peek_ids.insert(hash_id)
}

fn handle_shift_reduce_conflicts(
  t: &mut TPack,
  peek_group: &Vec<usize>,
  leaves: &mut Vec<usize>,
) -> bool {
  let goals =
    get_goals(peek_group, t).into_iter().map(|i| (i, &t.get_node(i).items)).collect::<Vec<_>>();

  if goals.iter().any(|(i, _)| t.get_node(*i).is(TST::I_OUT_OF_SCOPE)) {
    return false;
  }

  if goals.len() == 2
    && (goals[0].1.len() == 1 && goals[1].1.len() == 1 && (t.mode == TransitionMode::GoTo)
      || (goals[0].1[0].get_prod_id(&t.g) == goals[1].1[0].get_prod_id(&t.g)))
  {
    let shift = goals.iter().filter(|(_, i)| !i[0].is_end()).collect::<Vec<_>>();

    let mut reduce = goals.iter().filter(|(_, i)| i[0].is_end());

    if !shift.is_empty() && reduce.next().is_some() {
      let shift_goal = shift[0].0;

      for node_index in peek_group {
        if t.get_node(*node_index).goal == shift_goal {
          leaves.push(*node_index)
        } else {
          t.drop_node(node_index);
        }
      }

      return true;
    }
  }

  false
}

fn groups_are_aliased(peek_group: &Vec<usize>, t: &mut TPack) -> bool {
  return false;

  hash_group_vec(peek_group.to_owned(), |_, n| {
    t.get_node(n).items.clone().iter().map(|i| i.to_zero_state()).collect::<Vec<_>>().sort()
  })
  .iter()
  .any(|g| g.len() > 1)
}

/// Compares the terminal symbols of node groups and merges those
/// groups whose terminal symbols occlude each other.
///
/// For instance, given a group `A` with the symbol `g:id` and an
/// other group `B` with symbol `\g`, the character `g` could be
/// accepted by either group. As long as group `A` (the "defined"
/// group) is not exclusive, we merge group `B` into `A` to into
/// account the ambiguous nature of the groups.

fn merge_occluding_groups(t: &mut TPack, groups: &mut [Vec<usize>]) {
  // Clone the from_group store so we are able
  // to merge its members into to_groups without
  // going fowl of the borrow checker.
  if (!t.is_scanner) {
    return;
  }

  for i in 0..groups.len() {
    for j in 0..groups.len() {
      if i == j {
        continue;
      }

      let from_node = t.get_node(groups[i][0]);
      let to_node = t.get_node(groups[j][0]);

      let from_origin = from_node.items[0].get_origin();
      let to_origin = to_node.items[0].get_origin();

      // Scanner items that originate from the same symbol do not require occlusion
      // checking.
      if matches!(from_origin, OriginData::Symbol(..)) && from_origin == to_origin {
        continue;
      }

      let from_sym = from_node.edge_symbol;
      let to_sym = to_node.edge_symbol;

      if symbols_occlude(&to_sym, &from_sym, &t.g)
        && ((!from_node.is_out_of_scope()) || (from_sym.is_defined() || to_sym.is_defined()))
      {
        let mut clone = groups[i].clone();
        groups[j].append(&mut clone);
        t.get_node_mut(groups[j][0]).set_type(TST::I_MERGE_ORIGIN);
      }
    }
  }
}

fn get_goals(e_nodes: &[usize], t: &TPack) -> Vec<usize> {
  BTreeSet::<usize>::from_iter(e_nodes.iter().map(|node_id| t.get_node(*node_id).goal))
    .into_iter()
    .collect::<Vec<_>>()
}

fn handle_unresolved_scanner_nodes(t: &mut TPack, nodes: Vec<usize>, leaves: &mut Vec<usize>) {
  let mut defined = nodes;
  let mut generic = defined
    .drain_filter(|i| match t.get_node(*i).items[0].get_origin() {
      OriginData::Symbol(sym) => !sym.is_defined(),
      _ => false,
    })
    .collect::<Vec<_>>();
  let productions = generic
    .drain_filter(|i| match t.get_node(*i).items[0].get_origin() {
      OriginData::Symbol(sym) => !sym.is_production(),
      OriginData::Production(_) => {
        unreachable!("Origin Data should be a symbol!");
      }
      OriginData::UNDEFINED => {
        unreachable!("Origin Symbols Data not defined!");
      }
      _ => false,
    })
    .collect::<Vec<_>>();

  match (defined.len(), productions.len()) {
    (1, _) => {
      leaves.push(defined[0]);
      for node_id in generic.iter().chain(productions.iter()) {
        t.drop_node(&node_id);
      }
    }
    (0, 1) => {
      leaves.push(productions[0]);
      for node_id in generic.iter() {
        t.drop_node(&node_id);
      }
    }
    (a, b) if a + b > 1 => {
      /// HCTKError::Transition_Invalid_Defined{root_symbols, chains}
      panic!(
        "Invalid combination of defined  \n {}!",
        defined
          .into_iter()
          .chain(productions.into_iter())
          .map(|node_id| t.get_node(node_id).debug_string(&t.g))
          .collect::<Vec<_>>()
          .join("\n")
      );
    }
    _ => {
      /// InvalidGenerics
      /// HCTKError::Transition_Invalid_Generics{root_symbols, chains}
      panic!(
        "Invalid combination of generics while creating transition states for [{:#?}]\n {}!",
        t.root_prod_ids.iter().map(|p_id| t.g.get_production_plain_name(p_id)).collect::<Vec<_>>(),
        generic
          .into_iter()
          .map(|node_id| t.get_node(node_id).debug_string(&t.g))
          .collect::<Vec<_>>()
          .join("\n")
      );
    }
  }
}

fn handle_unresolved_nodes(t: &mut TPack, peek_group: Vec<usize>, leaves: &mut Vec<usize>) {
  if t.is_scanner {
    handle_unresolved_scanner_nodes(t, peek_group, leaves)
  } else {
    let goals = get_goals(&peek_group, t);

    if goals.len() < 2 {
      panic!("Unexpectedly ended up here with only one goal!");
    }

    // TODO: Filter out low priority goals.

    // Create a fork state -----------------------------------------------

    let prime_node = peek_group[0];
    let mut parent = 0;
    let mut items = vec![];

    for node_index in &peek_group[0..peek_group.len()] {
      let node = t.get_node(*node_index);
      items.push(node.items[0]);
      parent = t.drop_node(node_index);
    }

    let goal_hash =
      hash_id_value_u64(goals.iter().map(|i| t.get_node(*i).first_item()).collect::<Vec<_>>());

    match t.events.get(&goal_hash).to_owned() {
      Some(fork_node_index) => {
        t.get_node_mut(*fork_node_index).proxy_parents.push(parent);
      }

      None => {
        warn_of_ambiguous_productions(t, &goals);

        let mut fork_node = TGN::new(t, SymbolID::Default, parent, items);

        fork_node.set_type(TST::I_FORK);

        fork_node.parent = parent;

        let fork_node_index = t.insert_node(fork_node);

        t.events.insert(goal_hash, fork_node_index);

        for goal_index in goals {
          process_node(t, goal_index, fork_node_index, false);
        }
      }
    }
  }
}

fn warn_of_ambiguous_productions(t: &mut TPack, goals: &Vec<usize>) {
  // Look for a common production in each goal. If such production(s) exist,
  // issue warning(s) about production occlusion.
  let mut closures = goals
    .iter()
    .map(|i| {
      get_closure_cached(&t.get_node(*i).first_item(), &t.g)
        .iter()
        .map(|i| (i.get_symbol(&t.g), i))
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let smallest = closures
    .iter()
    .fold(&closures[0], |f, i| match i.len() < f.len() {
      true => i,
      false => f,
    })
    .clone();
  // Get a set of symbols that are present in all closures.
  let common_symbols = smallest
    .iter()
    .filter_map(|(sym, i)| match closures.iter().all(|c| c.iter().any(|(s, _)| s == sym)) {
      true => Some(sym),
      false => None,
    })
    .collect::<BTreeSet<_>>();
  // For each closure, remove all items that do not have a symbols that matches one in common_symbols,
  // or that is of a production whose id is in common_symbols
  closures.iter_mut().for_each(|c| {
    c.drain_filter(|(s, i)| {
      (!common_symbols.contains(s) || common_symbols.contains(&i.get_prod_as_sym_id(&t.g)))
    });
  });
  // At this point we should have isolated the items responsible for the ambiguous parse, provided
  // we have set of non-empty closures. We can now display an appropriate message to the
  // user regarding the nature of the ambiguous parse producing bodies.
  if closures.iter().all(|c| !c.is_empty()) {
    t.errors.push(crate::types::HCError::transition_err_ambiguous_production {
      source_production: t.g.get_production(t.root_prod_ids.first().unwrap()).unwrap().clone(),
      body_refs:         closures
        .iter()
        .flat_map(|c| {
          c.iter().map(|(_, i)| {
            let prod = i.get_body_ref(&t.g).unwrap();
            (prod.grammar_ref.clone(), prod.tok.clone())
          })
        })
        .collect(),
    });
  }
}

/// Set the transition type of a peeking based on whether
/// it is first node in the peek path or not. If it is the first
/// node, we do regular ASSERT action on the terminal symbol.
/// Otherwise we use a PEEK action.
fn set_transition_type(t: &mut TPack, node_index: usize, depth: u32) {
  t.get_node_mut(node_index).set_type(match depth {
    0 => TST::O_ASSERT,
    _ => TST::O_PEEK,
  })
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct LinkedItem {
  pub link: Item,
  pub item: Item,
}

struct TermAndEndItemGroups {
  pub term_items: Vec<LinkedItem>,
  pub end_items:  Vec<LinkedItem>,
}

impl TermAndEndItemGroups {
  pub fn get_all_items(&self) -> Vec<Item> {
    self.term_items.iter().chain(self.end_items.iter()).map(|t| t.item).collect()
  }

  pub fn get_term_items(&self) -> Vec<Item> {
    self.term_items.iter().map(|t| t.item).collect()
  }

  pub fn get_end_items(&self) -> Vec<Item> {
    self.end_items.iter().map(|t| t.item).collect()
  }
}

/// Retrieve terminal items derived from a
/// completed item.
///
/// ### Returns
/// A vector of Item tuples, where the first item is the
/// previous link item for closure resolution, and the second item is
/// a non-end item  produced from the reduction of the
/// `root_end_item`.
fn scan_items(
  t: &mut TPack,
  root_end_item: &Item,
  lr_state_ref: Option<usize>,
) -> TermAndEndItemGroups {
  let mut seen = BTreeSet::<LinkedItem>::new();
  let mut out = BTreeSet::<LinkedItem>::new();
  let mut fin_items = BTreeSet::<LinkedItem>::new();

  static empty_vec: Vec<Item> = Vec::new();
  // Starting at the top we grab the closure to the nearest
  // non-term link.

  // Stores the end item [1] and its immediate closure item [0]
  let mut end_items = VecDeque::from_iter(vec![(
    LinkedItem { link: t.get_closure_link(root_end_item), item: *root_end_item },
    lr_state_ref,
  )]);
  while let Some((linked, state_ref)) = end_items.pop_front() {
    if seen.insert(linked) {
      let prod = linked.item.get_prod_id(&t.g);
      // Grab all productions from the closure that match the end item's
      // production.
      match {
        let (iter, prev_state) = match (state_ref, linked) {
          (Some(prev_node), _) if prev_node >= 0 && prev_node != TGN::OrphanIndex => {
            // Use the prev_node
            let node = t.get_node(prev_node);
            (node.items.iter(), Some(node.parent))
          }
          (_, LinkedItem { link, item }) if link.is_null() => (
            if item.is_out_of_scope() {
              t.out_of_scope_closure.as_deref().unwrap_or(&empty_vec).iter()
            } else {
              t.scoped_closures
                .get(link.get_state().get_closure_index())
                .unwrap_or(&empty_vec)
                .iter()
            },
            None,
          ),
          (_, LinkedItem { link, item }) if link.is_out_of_scope() => (
            if let Some(closure) = &t.goto_scoped_closure {
              closure.iter()
            } else {
              empty_vec.iter()
            },
            None,
          ),
          _ => (get_closure_cached(&linked.link, &t.g).iter(), None),
        };
        (
          iter
            .filter(|i| i.get_production_id_at_sym(&t.g) == prod)
            .cloned()
            .map(|i| i.to_origin(linked.link.get_origin()))
            .collect::<Vec<_>>(),
          prev_state,
        )
      } {
        (empty, prev_state) if empty.is_empty() => {
          if linked.link.is_null() {
            // t_pack.set_closure_link(end_item, nonterm_closure_link);
            fin_items.insert(linked);
          } else {
            end_items.push_back((
              LinkedItem { link: t.get_closure_link(&linked.link), item: linked.item },
              prev_state,
            ));
          }
        }
        (prod_items, prev_state) => {
          for p_item in prod_items {
            let item = p_item.increment().unwrap().to_state(linked.item.get_state());

            if item.is_end() {
              end_items.push_back((LinkedItem { link: linked.link, item }, prev_state));
            } else {
              t.set_closure_link(item, linked.link);
              out.insert(LinkedItem { link: linked.link, item });
            }
          }
        }
      }
    }
  }

  TermAndEndItemGroups {
    end_items:  fin_items.into_iter().collect(),
    term_items: out.into_iter().collect(),
  }
}

fn process_terminal_node(t: &mut TPack, term_items: &Vec<Item>, parent_index: usize) -> Vec<usize> {
  let sym = term_items[0].get_symbol(&t.g);

  let new_node = TGN::new(t, sym, parent_index, term_items.to_vec());

  let node_index = t.insert_node(new_node);

  t.get_node_mut(node_index).set_type(TST::I_SHIFT | TST::O_TERMINAL);

  t.queue_node(node_index);

  vec![node_index]
}

fn create_production_call(
  t: &mut TPack,
  prod_id: ProductionId,
  nonterm_items: Vec<Item>,
  par_id: usize,
) -> Vec<usize> {
  let mut node = TGN::new(t, SymbolID::Production(prod_id, GrammarId(0)), par_id, nonterm_items);

  node.prod_sym = Some(SymbolID::Production(prod_id, GrammarId(0)));

  node.trans_type |= TST::O_PRODUCTION;

  let node_index = t.insert_node(node);

  t.queue_node(node_index);

  vec![node_index]
}

fn process_end_item(t: &mut TPack, end_item: Item, parent_index: usize) -> Vec<usize> {
  if t.is_scanner && !t.starts.contains(&end_item.to_start().to_zero_state()) {
    // We need to be in the initial closure before we can allow
    // a scanner run to exit successfully. Thus, the production of the end state
    // is used to select the next set of items to be scanned, continuing the scan process
    // until we arrive at an end_item that belongs to the root closu.

    let scanned_items = scan_items(t, &end_item, None);

    // Filter out items automatically handled by goto
    if !scanned_items.term_items.is_empty() {
      let node = TGN::new(t, SymbolID::EndOfFile, parent_index, scanned_items.get_all_items());

      let node_index = t.insert_node(node);

      let results = process_node(t, node_index, parent_index, false);

      t.drop_node(&node_index);

      return results;
    }
  }

  t.gotos.insert(end_item);

  let end_node = TGN::new(t, SymbolID::EndOfFile, parent_index, vec![end_item]);

  let node_index = t.insert_node(end_node);

  t.get_node_mut(node_index).set_type(TST::I_END | TST::O_ASSERT);

  t.leaf_nodes.push(node_index);

  vec![node_index]
}

fn add_item_closures(start_items: &Vec<Item>, t: &mut TPack) {
  for item in start_items {
    t.scoped_closures.push(get_closure_cached(item, &t.g).clone());

    if item.is_nonterm(&t.g) {
      t.set_closure_link(*item, item.to_null());
    }
  }
}
/// Constructs nodes that uses the LR strategy for parsing
/// This means that any state may have a GOTO clause, with in Hydrocarbon
/// Case means a alternative node branch that stores all GOTO nodes,
/// whose address is then pushed first pushed to the stack before transitioning
/// to sibling nodes.
fn TODO_constructLR_transitions(
  t: &mut TPack,
  starts: &Vec<Item>,
  parent_index: usize,
  start_symbol: SymbolID,
) -> HCResult<usize> {
  let closure = starts.iter().flat_map(|i| get_closure_cached(i, &t.g)).cloned().collect();

  // In this mode, we take the closure of the entry items, and create
  // transitions from each set of terminals and non-terminals to new states, which are
  // comprised of the shifted items of the previous state.
  // Non-terminal transition represent the parent states goto, and are grouped into
  // a separate state the is prefixed push to the branches of the parent state.

  let mut encountered_states = HashMap::<BTreeSet<Item>, usize>::new();

  // create the root node for the start items
  let root_node = TGN::new(t, start_symbol, parent_index, closure);
  let root_node_index = t.insert_node(root_node);
  t.get_node_mut(root_node_index).set_type(TST::I_LR_START);
  let mut to_process = VecDeque::from_iter(vec![root_node_index]);

  while let Some(parent_index) = to_process.pop_front() {
    let mut closure = t
      .get_node(parent_index)
      .items
      .iter()
      .flat_map(|i| match i.is_end() {
        true => vec![*i],
        _ => get_closure_cached(i, &t.g).clone(),
      })
      .collect::<BTreeSet<_>>();

    // debug_items(&format!("state: {}\n", parent_index), closure.clone(), &t.g);

    // let terms = closure.drain_filter(|i| i.is_nonterm(g)).collect::<BTreeSet<_>>();
    let end_items = closure.drain_filter(|i| i.is_end()).collect::<BTreeSet<_>>();
    let terms_and_non_terms = closure; // Non-terms become this state's GOTO

    for (symbol, group) in hash_group_btreemap(terms_and_non_terms, |i, v| v.get_symbol(&t.g)) {
      // Create a new transition node and add the incremented items to  it.
      let incremented_items = group.into_iter().map(|i| i.increment().unwrap()).collect::<Vec<_>>();
      let canonical_items =
        incremented_items.iter().map(|i| i.to_zero_state()).collect::<BTreeSet<_>>();

      match encountered_states.entry(canonical_items) {
        std::collections::hash_map::Entry::Occupied(e) => {
          let child_index = *e.get();
          t.get_node_mut(child_index).proxy_parents.push(parent_index);
        }
        std::collections::hash_map::Entry::Vacant(e) => {
          // If we can call into a state then we shall
          match symbol {
            SymbolID::Production(prod_id, _) => {
              let child_node = TGN::new(t, symbol, parent_index, incremented_items);
              let child_index = t.insert_node(child_node);
              t.get_node_mut(child_index).set_type(TST::I_GOTO_LR);
              to_process.push_back(child_index);
              e.insert(child_index);
            }
            _ => {
              let child_node = TGN::new(t, symbol, parent_index, incremented_items);
              let child_index = t.insert_node(child_node);
              t.get_node_mut(child_index).set_type(TST::I_SHIFT | TST::O_TERMINAL);
              to_process.push_back(child_index);
              e.insert(child_index);
            }
          }
        }
      }
      //
    }

    match end_items.len() {
      2.. => {
        let ancestor_index = Some(t.get_node(parent_index).parent);

        /// Get the follow for each node.
        let end_items =
          end_items.into_iter().map(|i| (i, scan_items(t, &i, ancestor_index))).collect::<Vec<_>>();

        if end_items
          .iter()
          .all(|(i, items)| items.end_items.is_empty() && !items.term_items.is_empty())
        {
          let symbol_groups = hash_group_btreemap(
            end_items
              .iter()
              .flat_map(|(i, items)| {
                items
                  .get_term_items()
                  .iter()
                  .flat_map(|i| get_closure_cached(i, &t.g))
                  .collect::<BTreeSet<_>>()
                  .iter()
                  .filter_map(|u| u.is_term(&t.g).then(|| (i.clone(), **u)))
                  .collect::<Vec<_>>()
              })
              .collect::<Vec<_>>(),
            |i, (_, term)| term.get_symbol(&t.g),
          );

          if symbol_groups.iter().any(|g| g.1.len() > 1) {
            return HCResult::Err(format!("Could not disambiguate grammar here:",).into());
          } else {
            for (sym, mut items) in symbol_groups {
              let (end_item, term_item) = items.pop().unwrap();
              let end_node = TGN::new(t, sym, parent_index, vec![end_item]);
              let node_index = t.insert_node(end_node);
              t.get_node_mut(node_index).set_type(TST::I_END | TST::O_ASSERT);
              t.leaf_nodes.push(node_index);
            }
          }
        } else {
          for (i, items) in &end_items {
            println!("{}", i.debug_string(&t.g));
            debug_items("term items", items.get_term_items().clone(), &t.g);
            debug_items("end items", items.get_end_items().clone(), &t.g);
          }

          return HCResult::Err(
            format!(
              "Encountered conflicting end items:\n{}\n",
              end_items
                .into_iter()
                .map(|(i, _)| "   ".to_string()
                  + &i.blame_string(&t.g)
                  + "\n"
                  + &i.get_body(&t.g).unwrap().tok.blame(0, 0, "Defined here", BlameColor::Red))
                .collect::<Vec<_>>()
                .join("\n")
            )
            .into(),
          );
        }
      }
      1 => {
        // Default State
        process_end_item(t, end_items.into_iter().next().unwrap(), parent_index);
      }
      _ => {}
    }
  }

  HCResult::Ok(root_node_index)
}

#[cfg(test)]
mod transition_tests {
  use std::collections::BTreeSet;

  use crate::grammar::get_production_start_items;
  use crate::intermediate::state::process_transition_nodes;
  use crate::types::GrammarStore;
  use crate::types::HCResult;
  use crate::types::TransitionMode;
  use crate::types::TransitionPack;
  use HCResult::*;

  use super::TODO_constructLR_transitions;

  #[test]
  fn test_TODO_constructLR_transitions() -> HCResult<()> {
    let g = GrammarStore::from_str(
      " @IGNORE g:sp 

        <> A > X\\c
             | Y \\d

        <> X > \\x X?

        <> Y > \\x Y?
      ",
    )?;

    let prod_id = g.get_production_id_by_name("A")?;
    let items = get_production_start_items(&prod_id, &g);

    let mut t = TransitionPack::new(
      g,
      TransitionMode::RecursiveDescent,
      false,
      &items,
      BTreeSet::from_iter(vec![prod_id]),
    );

    TODO_constructLR_transitions(&mut t, &items, usize::MAX, crate::types::SymbolID::Start)?;

    // t.print_nodes();

    let (states, _) = process_transition_nodes(&t, &"Titus".to_string());

    for state in states {
      println!("{}", state.to_string())
    }

    Ok(())
  }
}
