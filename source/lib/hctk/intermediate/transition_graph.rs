use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::hash::Hash;
use std::process::id;
use std::rc::Rc;
use std::vec;

use crate::grammar::create_closure;
use crate::grammar::get_closure_cached;
use crate::grammar::get_production_start_items;
use crate::grammar::hash_id_value_u64;
use crate::types::GrammarId;
use crate::types::GrammarStore;
use crate::types::Item;
use crate::types::ItemState;
use crate::types::OriginData;
use crate::types::ProductionId;
use crate::types::SymbolID;
use crate::types::TransitionGraphNode as TGN;
use crate::types::TransitionMode;
use crate::types::TransitionPack as TPack;
use crate::types::TransitionStateType as TST;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]

enum TransitionGroupSelector {
  OutOfScope(SymbolID),
  EndItem(u64),
  Symbol(SymbolID),
}

/// Constructs an initial transition graph that parses a production
/// using a recursive descent strategy. Productions that are ambiguous
/// or are left recursive cannot be parsed, so this tries to do its
/// best to define a parse path for a production before we have to
/// resort to LR and Forking based parse strategies.

pub fn construct_recursive_descent<'a>(
  g: &'a GrammarStore,
  is_scanner: bool,
  starts: &[Item],
) -> TPack<'a> {
  let start_items = apply_state_info(starts);

  let mut t_pack = TPack::new(g, TransitionMode::RecursiveDescent, is_scanner, &start_items);

  let non_lr_start_items = start_items
    .iter()
    .filter(|i| !t_pack.root_prods.contains(&i.get_production_id_at_sym(g)))
    .cloned()
    .collect::<Vec<_>>();

  for item in &non_lr_start_items {
    t_pack.scoped_closures.push(get_closure_cached(item, g));

    if item.is_nonterm(g) {
      t_pack.set_closure_link(*item, item.to_null());
    }
  }

  let root_index =
    t_pack.insert_node(TGN::new(&t_pack, SymbolID::Undefined, usize::MAX, non_lr_start_items));

  t_pack.get_node_mut(root_index).set_type(TST::I_DESCENT_START);

  t_pack.queue_node(root_index);

  while let Some(node_id) = t_pack.get_next_queued() {
    process_node(g, &mut t_pack, node_id, node_id, node_id != 0);
  }

  t_pack.clean()
}

fn apply_state_info(starts: &[Item]) -> Vec<Item> {
  let start_items = starts
    .iter()
    .enumerate()
    .map(|(i, item)| item.to_state(ItemState::new(i as u32, 0)))
    .collect::<Vec<_>>();
  start_items
}

pub fn construct_goto<'a>(
  g: &'a GrammarStore,
  is_scanner: bool,
  starts: &[Item],
  goto_seeds: &[Item],
) -> TPack<'a> {
  let start_items = apply_state_info(starts);

  let mut t_pack = TPack::new(g, TransitionMode::GoTo, is_scanner, &start_items);

  let global_closure = Rc::new(Box::<Vec<Item>>::new(if t_pack.is_scanner {
    vec![]
  } else {
    get_follow_closure(g, goto_seeds, &t_pack.root_prods)
  }));

  let mut parent = TGN::new(&t_pack, SymbolID::Undefined, TGN::OrphanIndex, goto_seeds.to_vec());

  parent.set_type(TST::I_GOTO_START);

  let parent_index = t_pack.insert_node(parent);

  for (production_id, group) in get_goto_starts(g, &start_items, goto_seeds) {
    t_pack.goto_scoped_closure = Some(global_closure.clone());

    let items: Vec<Item> = group
      .iter()
      .map(|i| {
        let stated_item = i.increment().unwrap();

        let stated_item = if stated_item.at_end() {
          stated_item.to_state(ItemState::OUT_OF_SCOPE_STATE)
        } else {
          t_pack.scoped_closures.push(get_closure_cached(&stated_item, g));

          stated_item.to_state(ItemState::new((t_pack.scoped_closures.len() - 1) as u32, 0))
        };

        stated_item
      })
      .collect();

    let mut goto_node = TGN::new(
      &t_pack,
      SymbolID::Production(production_id, GrammarId::default()),
      parent_index,
      items.clone(),
    );

    goto_node.set_type(TST::O_GOTO);

    if (!group.is_empty() && (group.iter().any(|i| i.increment().unwrap().at_end())))
      || t_pack.root_prods.contains(&production_id)
    {
      if t_pack.root_prods.contains(&production_id) {
        let mut reducible: Vec<Item> = g
          .lr_items
          .get(&production_id)
          .unwrap_or(&BTreeSet::new())
          .iter()
          .filter(|i| !group.iter().any(|g| *g == **i))
          .map(|i| i.increment().unwrap().to_state(ItemState::OUT_OF_SCOPE_STATE))
          .collect();

        goto_node.items.append(&mut reducible);
      }

      let node_id = t_pack.insert_node(goto_node);

      create_peek(g, &mut t_pack, node_id, items);
    } else {
      let node_index = t_pack.insert_node(goto_node);
      process_node(g, &mut t_pack, node_index, node_index, false);
    }
  }

  while let Some(node_index) = t_pack.get_next_queued() {
    process_node(g, &mut t_pack, node_index, node_index, node_index != 0);
  }

  t_pack.clean()
}

pub fn get_goto_starts(
  // root_production: &ProductionId,
  g: &GrammarStore,
  starts: &Vec<Item>,
  seeds: &[Item],
) -> Vec<(ProductionId, BTreeSet<Item>)> {
  type GotoMap = BTreeMap<ProductionId, BTreeSet<Item>>;

  let mut local_lr_items = GotoMap::new();

  let mut output_items = GotoMap::new();

  let mut batch = VecDeque::from_iter(seeds.iter().map(|i| i.get_prod_id(g)));

  fn insert(goto_items: &mut GotoMap, production_id: &ProductionId, item: Item) {
    if !goto_items.contains_key(production_id) {
      goto_items.insert(*production_id, BTreeSet::<Item>::new());
    }

    goto_items.get_mut(production_id).unwrap().insert(item);
  }

  for start_item in starts {
    for item in get_closure_cached(start_item, g) {
      if !item.at_end() && item.is_nonterm(g) {
        insert(&mut local_lr_items, &item.get_production_id_at_sym(g), *item)
      }
    }
  }

  while let Some(production_id) = batch.pop_front() {
    if !output_items.contains_key(&production_id) && local_lr_items.contains_key(&production_id) {
      let items = local_lr_items.get(&production_id).unwrap();

      for item in items {
        batch.push_back(item.get_prod_id(g));
      }

      output_items.insert(production_id, items.clone());
    }
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
      .unwrap_or(&BTreeSet::new())
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

fn process_node(
  g: &GrammarStore,
  t_pack: &mut TPack,
  node_id: usize,
  par_id: usize,
  allow_increment: bool,
) -> Vec<usize> {
  let node = t_pack.get_node(node_id).clone();
  let mut items: Vec<Item> = node.items.clone();

  if allow_increment {
    items = items
      .into_iter()
      .map(|i| {
        if i.at_end() {
          i
        } else {
          let item_next = i.increment().unwrap();
          let item_link = t_pack.get_closure_link(&i);
          t_pack.set_closure_link(item_next, item_link);
          item_next
        }
      })
      .collect();
  }

  let n_term: Vec<Item> = items.iter().filter(|i| i.is_term(g)).cloned().collect();

  let n_nonterm: Vec<Item> = items.iter().filter(|i| i.is_nonterm(g)).cloned().collect();

  let n_end: Vec<Item> = items.iter().filter(|i| i.is_end()).cloned().collect();

  if !n_nonterm.is_empty() {
    let production_ids =
      BTreeSet::<ProductionId>::from_iter(n_nonterm.iter().map(|i| i.get_production_id_at_sym(g)));

    if n_term.is_empty()
      && n_end.is_empty()
      && production_ids.len() == 1
      && (node.depth > 0 || non_recursive(&n_nonterm, &t_pack.root_prods, g))
    {
      create_production_call(g, t_pack, *production_ids.iter().next().unwrap(), n_nonterm, par_id)
    } else {
      create_peek(g, t_pack, par_id, items)
    }
  } else if n_end.len() == 1 && n_term.is_empty() {
    // A single end item
    process_end_item(g, t_pack, n_end[0], par_id)
  } else if n_end.is_empty() && n_term.len() == 1 {
    // A single terminal item
    process_terminal_node(g, t_pack, &vec![n_term[0]], par_id)
  } else if n_end.is_empty()
    // The terminal symbol is identical in all items.
    && n_term.iter().map(|t| t.get_symbol(g)).collect::<BTreeSet<_>>().len() == 1
  {
    process_terminal_node(g, t_pack, &n_term, par_id)
  } else {
    // else if n_end.is_empty() && n_term.len() > 1  {
    // Multiple terminal items
    create_peek(g, t_pack, par_id, items)
  }
}

/// True if the closure of the givin items does not include
/// the goal production on the right of the cursor

fn non_recursive(items: &[Item], target_prod: &BTreeSet<ProductionId>, g: &GrammarStore) -> bool {
  for item in create_closure(items, g) {
    if let SymbolID::Production(production, _) = item.get_symbol(g) {
      if target_prod.contains(&production) {
        return false;
      }
    }
  }

  true
}

fn create_peek(
  g: &GrammarStore,
  t_pack: &mut TPack,
  par_id: usize,
  items: Vec<Item>,
) -> Vec<usize> {
  let mut cached_depth = 0;

  {
    let parent = t_pack.get_node_mut(par_id);
    parent.set_type(TST::I_PEEK_ORIGIN);
    cached_depth = parent.depth;
  }

  let goals: Vec<usize> = if t_pack.is_scanner {
    hash_group(items, |_, i| i)
  } else {
    hash_group(items, |ind, i| {
      if i.at_end() {
        format!("at_end_{}", ind)
      } else {
        format!("{:?} {}", i.get_symbol(g), i.get_state().is_out_of_scope())
      }
    })
  }
  .iter()
  .enumerate()
  .map(|(i, item)| {
    let mut root_node =
      TGN::new(t_pack, item[0].get_symbol(g).to_owned(), usize::MAX, item.to_vec());

    root_node.depth = cached_depth;

    if t_pack.mode == TransitionMode::GoTo && item[0].at_end() {
      root_node.trans_type |= TST::I_COMPLETE;
    }

    t_pack.insert_node(root_node)
  })
  .collect();

  let mut peek_nodes = vec![];

  for (i, goal_index) in goals.iter().cloned().enumerate() {
    let items = t_pack
      .get_node(goal_index)
      .items
      .to_owned()
      .iter()
      .map(|i| (t_pack.get_closure_link(i), *i))
      .collect::<Vec<_>>();

    for mut node in create_term_nodes_from_items(g, t_pack, &items, par_id, &BTreeSet::new()) {
      node.goal = goal_index;

      node.depth = t_pack.get_node(goal_index).depth;

      peek_nodes.push(t_pack.insert_node(node));
    }
  }

  let mut leaves = vec![];

  disambiguate(g, t_pack, peek_nodes, &mut leaves, 0);

  let resolved_leaves = process_peek_leaves(g, t_pack, leaves);

  // All goal nodes can be recycled, as copy operations where used to
  // insert goal nodes as children of peek leaves
  for goal in goals {
    t_pack.drop_node(&goal);
  }

  t_pack.clear_peek_data();

  resolved_leaves
}

fn create_term_nodes_from_items(
  g: &GrammarStore,
  t_pack: &mut TPack,
  items: &Vec<(Item, Item)>,
  par_id: usize,
  discarded: &BTreeSet<Item>,
) -> Vec<TGN> {
  let mut term_nodes = vec![];

  for (link_item, item) in items {
    if item.is_term(g) || item.is_end() {
      t_pack.set_closure_link(*item, *link_item);
      let node = TGN::new(t_pack, item.get_symbol(g), par_id, vec![*item]);

      term_nodes.push(node);
    } else {
      for closure_item in create_closure(&[*item], g) {
        let closure_item = closure_item.to_origin(item.get_origin());

        if !closure_item.is_nonterm(g) && !discarded.contains(item) {
          let closure_item = closure_item.to_state(item.get_state());

          t_pack.set_closure_link(closure_item, *item);

          let node = TGN::new(t_pack, closure_item.get_symbol(g), par_id, vec![closure_item]);

          term_nodes.push(node);
        }
      }
    }
  }

  term_nodes
}
/// Constructs a Vector of Vectors, each of which contains a set items from the
/// original vector that have been grouped by the hash of a common distinguisher.
pub fn hash_group<T: Copy + Sized, R: Hash + Sized + Ord + Eq, Function: Fn(usize, T) -> R>(
  vector: Vec<T>,
  hash_yielder: Function,
) -> Vec<Vec<T>> {
  let mut hash_groups = BTreeMap::<R, Vec<T>>::new();

  for (i, val) in vector.iter().enumerate() {
    match hash_groups.entry(hash_yielder(i, *val)) {
      Entry::Vacant(e) => {
        e.insert(vec![*val]);
      }
      Entry::Occupied(mut e) => e.get_mut().push(*val),
    }
  }

  hash_groups.into_values().collect()
}

#[inline]
fn disambiguate(
  g: &GrammarStore,
  t_pack: &mut TPack,
  node_ids: Vec<usize>,
  leaves: &mut Vec<usize>,
  depth: u32,
) {
  let mut term_nodes = vec![];
  let mut end_nodes = vec![];

  // We must first complete end-items and generate new
  // nodes that arise from the completion of a production.

  for node_index in node_ids {
    let node = t_pack.get_node(node_index);
    let item = node.items[0];
    let goal = node.goal;
    let parent_index = node.parent;

    if !item.is_end() {
      term_nodes.push(node_index)
    } else {
      let (mut terms, mut final_ends) =
        get_continue_items(g, t_pack, item, parent_index, depth, goal);

      if terms.is_empty() && final_ends.is_empty() {
        end_nodes.push(node_index);
      } else {
        term_nodes.append(&mut terms);
        end_nodes.append(&mut final_ends);

        t_pack.drop_node(&node_index);
      }
    }
  }

  match end_nodes.len() {
    0 => {}
    1 => {
      set_transition_type(t_pack, end_nodes[0], depth);
      leaves.push(end_nodes[0]);
    }
    _ => {
      if get_goals(&end_nodes, t_pack).len() == 1 {
        for end_node in &end_nodes[1..] {
          t_pack.drop_node(end_node);
        }

        set_transition_type(t_pack, end_nodes[0], depth);
        leaves.push(end_nodes[0]);
      } else {
        if t_pack.is_scanner {
        } else {
          handle_unresolved_nodes(g, t_pack, end_nodes, leaves);
        }
      }
    }
  }

  let mut groups = hash_group(term_nodes, |_, n| t_pack.get_node(n).terminal_symbol);

  let mut next_peek_groups = vec![];

  merge_occluding_groups(g, t_pack, &mut groups);

  let mut primary_nodes = BTreeSet::new();
  let mut drop_nodes = BTreeSet::new();

  for group in groups.iter() {
    let prime_node_index = group[0];

    let goals = get_goals(group, t_pack);

    set_transition_type(t_pack, prime_node_index, depth);

    if goals.len() > 1 && group.len() > 1 {
      let mut peek_transition_group = vec![];

      for node_index in group.iter().cloned() {
        let transition_on_skipped_symbol = t_pack.get_node(node_index).is(TST::I_SKIPPED_COLLISION);

        let goal = t_pack.get_node(node_index).goal;

        for mut node in create_term_nodes_from_items(
          g,
          t_pack,
          &t_pack
            .get_node(node_index)
            .items
            .clone()
            .into_iter()
            .map(|i| {
              let link = t_pack.get_closure_link(&i);
              if transition_on_skipped_symbol {
                (link, i)
              } else {
                (link, i.increment().unwrap())
              }
            })
            .collect::<Vec<_>>(),
          prime_node_index,
          &BTreeSet::new(),
        ) {
          node.goal = goal;
          peek_transition_group.push(t_pack.insert_node(node));
        }
      }

      if peek_transition_group.is_empty() {
        if get_goals(group, t_pack).iter().all(|i| t_pack.get_node(*i).is(TST::I_OUT_OF_SCOPE)) {
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

    for node_index in group.iter() {
      if *node_index != prime_node_index {
        let items = t_pack.get_node(*node_index).items.clone();
        drop_nodes.insert(node_index);
        t_pack.get_node_mut(prime_node_index).items.append(&mut items.clone());
      } else {
        primary_nodes.insert(prime_node_index);
      }
    }
  }

  for drop_node in drop_nodes {
    if !primary_nodes.contains(&drop_node) {
      t_pack.drop_node(&drop_node);
    }
  }

  for peek_group in next_peek_groups {
    if !peek_group.is_empty() {
      if handle_shift_reduce_conflicts(g, t_pack, &peek_group, leaves) {
        continue;
      } else if groups_are_aliased(&peek_group, t_pack) || group_is_repeated(&peek_group, t_pack) {
        handle_unresolved_nodes(g, t_pack, peek_group, leaves);
      } else {
        disambiguate(g, t_pack, peek_group, leaves, depth + 1);
      }
    }
  }
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
/// ASSERT_CONSUME.
#[inline]
fn process_peek_leaves(g: &GrammarStore, t_pack: &mut TPack, leaves: Vec<usize>) -> Vec<usize> {
  let mut resolved_leaves = Vec::<usize>::new();

  if t_pack.is_scanner {
    for node_index in leaves {
      // for node_index in peek_leaf_group {
      // Instead of resetting our position back to
      // the goal item, we simply continue parsing
      // from whatever position we are at.

      {
        let node = t_pack.get_node_mut(node_index);

        node.unset_type(TST::O_PEEK);
      }

      let mut iter_index = node_index;

      while !t_pack.get_node(iter_index).is(TST::I_PEEK_ORIGIN) {
        let node = t_pack.get_node_mut(iter_index);

        node.unset_type(TST::O_PEEK);

        node.set_type(TST::I_CONSUME);

        iter_index = node.parent;
      }

      let node = t_pack.get_node_mut(node_index);

      if node.items.iter().any(|i| i.at_end()) {
        node.unset_type(TST::I_CONSUME);
      } else {
        node.set_type(TST::I_CONSUME);
      }

      process_node(g, t_pack, node_index, node_index, true);
    }
  } else {
    for peek_leaf_group in hash_group(leaves, |_, leaf| t_pack.get_node(leaf).goal).iter().cloned()
    {
      let primary_peek_parent_index = peek_leaf_group[0];

      let prime_node = t_pack.get_node(primary_peek_parent_index).to_owned();

      let goal_index = prime_node.goal;

      let goal_node = t_pack.get_node(goal_index);

      if goal_node.is(TST::I_OUT_OF_SCOPE) {
        for node_index in peek_leaf_group {
          t_pack.get_node_mut(node_index).set_type(TST::I_FAIL | TST::I_OUT_OF_SCOPE)
        }
      } else {
        // Use the goal node as a proxy to generate child nodes that
        // are then linked to the current peek leaf nodes.

        let parent_node_indices = peek_leaf_group
          .into_iter()
          .map(|n| {
            let node_depth = t_pack.get_node(n).depth;
            let node_parent = t_pack.get_node(n).parent;

            if node_depth == 0 {
              t_pack.drop_node(&n);

              node_parent
            } else {
              n
            }
          })
          .collect::<Vec<_>>();

        let primary_parent = parent_node_indices[0];

        let proxy_parents = parent_node_indices[1..].to_owned();

        let have_proxy_parents = !proxy_parents.is_empty();

        for child_index in process_node(g, t_pack, goal_index, primary_parent, false) {
          let mut child_node = t_pack.get_node_mut(child_index);

          child_node.parent = primary_parent;

          if have_proxy_parents {
            child_node.proxy_parents.append(&mut proxy_parents.to_owned());
          }

          resolved_leaves.push(child_index);

          if child_node.prod_sym.is_some() {
            child_node.terminal_symbol = prime_node.terminal_symbol
          }
        }

        // Note: Remember all goal nodes are DROPPED at the
        // end of the peek resolution process
      }
    }
  }

  resolved_leaves
}
fn get_continue_items(
  g: &GrammarStore,
  t_pack: &mut TPack,
  end_item: Item,
  parent_index: usize,
  peek_depth: u32,
  goal: usize,
) -> (Vec<usize>, Vec<usize>) {
  let mut term_nodes = vec![];
  let mut final_nodes = vec![];

  let discard = BTreeSet::<Item>::new();

  if peek_depth == 0
    && t_pack.mode == TransitionMode::GoTo
    && end_item.decrement().unwrap().is_nonterm(g)
  {
    // TODO: Remove items that present in other branches
    // to prevent item aliasing.
  }
  let mut need_to_prune = false;

  let (scan_items, final_items) = scan_items(g, t_pack, &end_item);

  for mut term_node in create_term_nodes_from_items(g, t_pack, &scan_items, parent_index, &discard)
  {
    term_node.goal = goal;
    term_nodes.push(t_pack.insert_node(term_node));
  }

  for mut final_node in
    create_term_nodes_from_items(g, t_pack, &final_items, parent_index, &discard)
  {
    final_node.goal = goal;
    final_node.set_type(TST::I_END | TST::O_ASSERT);
    final_nodes.push(t_pack.insert_node(final_node));
  }

  (term_nodes, final_nodes)
}

fn group_is_repeated(peek_group: &[usize], t_pack: &mut TPack) -> bool {
  let group_id = peek_group
    .iter()
    .flat_map(|i| {
      let node = t_pack.get_node(*i);
      node.items.iter().map(|i| i.to_zero_state().to_hash())
    })
    .collect::<BTreeSet<_>>();

  let ids = group_id.iter().collect::<Vec<_>>();

  let hash_id = hash_id_value_u64(ids.clone());

  !t_pack.peek_ids.insert(hash_id)
}

fn handle_shift_reduce_conflicts(
  g: &GrammarStore,
  t_pack: &mut TPack,
  peek_group: &Vec<usize>,
  leaves: &mut Vec<usize>,
) -> bool {
  let goals = get_goals(peek_group, t_pack)
    .into_iter()
    .map(|i| (i, &t_pack.get_node(i).items))
    .collect::<Vec<_>>();

  if goals.len() == 2
    && (goals[0].1.len() == 1 && goals[1].1.len() == 1 && (t_pack.mode == TransitionMode::GoTo)
      || (goals[0].1[0].get_prod_id(g) == goals[1].1[0].get_prod_id(g)))
  {
    let shift = goals.iter().filter(|(_, i)| !i[0].is_end()).collect::<Vec<_>>();

    let mut reduce = goals.iter().filter(|(_, i)| i[0].is_end());

    if !shift.is_empty() && reduce.next().is_some() {
      let shift_goal = shift[0].0;

      for node_index in peek_group {
        if t_pack.get_node(*node_index).goal == shift_goal {
          leaves.push(*node_index)
        } else {
          t_pack.drop_node(node_index);
        }
      }

      return true;
    }
  }

  false
}

fn groups_are_aliased(peek_group: &Vec<usize>, t_pack: &mut TPack) -> bool {
  return false;

  hash_group(peek_group.to_owned(), |_, n| {
    t_pack.get_node(n).items.clone().iter().map(|i| i.to_zero_state()).collect::<Vec<_>>().sort()
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

fn merge_occluding_groups(g: &GrammarStore, t_pack: &mut TPack, groups: &mut [Vec<usize>]) {
  // Clone the from_group store so we are able
  // to merge its members into to_groups without
  // going fowl of the borrow checker.

  for i in 0..groups.len() {
    for j in 0..groups.len() {
      if i == j {
        continue;
      }

      let from_node = t_pack.get_node(groups[i][0]);
      let to_node = t_pack.get_node(groups[j][0]);

      let from_origin = from_node.items[0].get_origin();
      let to_origin = from_node.items[0].get_origin();

      // Scanner items that originate from the symbol do not require occlusion
      // checking.
      if matches!(from_origin, OriginData::Symbol(..)) && from_origin == to_origin {
        continue;
      }

      continue;

      let from_sym = from_node.terminal_symbol;
      let to_sym = to_node.terminal_symbol;

      if symbols_occlude(&to_sym, &from_sym, g) {
        let mut clone = groups[i].clone();
        groups[j].append(&mut clone);
        t_pack.get_node_mut(groups[j][0]).set_type(TST::I_MERGE_ORIGIN);
      }
    }
  }
}

/// Compares whether symbolB occludes symbolA
/// ( produces an ambiguous parse path )
///
/// Symbols that can occlude are as follows
///
/// - `g:id` and any single identifier character.
/// - `g:num` and any single numeric character.
/// - `g:sym` and any single character thats not a numeric,
///   identifier, space, newline, or tab.

fn symbols_occlude(symA: &SymbolID, symB: &SymbolID, g: &GrammarStore) -> bool {
  match symA {
    SymbolID::DefinedSymbol(_) => match symB {
      SymbolID::GenericSymbol => g.symbols.get(symA).unwrap().cp_len == 1,
      _ => false,
    },
    SymbolID::DefinedIdentifier(_) => match symB {
      SymbolID::GenericIdentifier => g.symbols.get(symA).unwrap().cp_len == 1,
      _ => false,
    },
    SymbolID::DefinedNumeric(_) => match symB {
      SymbolID::GenericNumber => g.symbols.get(symA).unwrap().cp_len == 1,
      _ => false,
    },
    _ => false,
  }
}

fn get_goals(e_nodes: &[usize], t_pack: &TPack) -> Vec<usize> {
  BTreeSet::<usize>::from_iter(e_nodes.iter().map(|n| t_pack.get_node(*n).goal))
    .into_iter()
    .collect::<Vec<_>>()
}

fn handle_unresolved_nodes(
  g: &GrammarStore,
  t_pack: &mut TPack,
  peek_group: Vec<usize>,
  leaves: &mut Vec<usize>,
) {
  if t_pack.is_scanner {
    //   if (peek_group.iter().map(|i| t_pack.get_node(*i)).all(|n| n.items[0].is_end())) {
    // leaves.push(peek_group[0]);
    // } else {
    // panic!("WTF");
    eprintln!(
      "AA {:?}",
      peek_group.iter().map(|n| t_pack.get_node(*n)).map(|n| (n.id, n.parent)).collect::<Vec<_>>()
    );
    // }
  } else {
    let goals = get_goals(&peek_group, t_pack);

    if goals.len() < 2 {
      panic!("Unexpectedly ended up here with only one goal!");
    }

    // TODO: Filter out low priority goals.

    // Create a fork state -----------------------------------------------

    let prime_node = peek_group[0];
    let mut parent = 0;
    let mut items = vec![];

    for node_index in &peek_group[0..peek_group.len()] {
      items.push(t_pack.get_node(*node_index).items[0]);
      parent = t_pack.drop_node(node_index);
    }

    let mut fork_node = TGN::new(t_pack, SymbolID::Default, parent, items);

    fork_node.set_type(TST::I_FORK);

    fork_node.parent = parent;

    let fork_node_index = t_pack.insert_node(fork_node);

    for goal_index in goals {
      process_node(g, t_pack, goal_index, fork_node_index, false);
    }
  }
}

/// Set the transition type of a peeking based on whether
/// it is first node in the peek path or not. If it is the first
/// node, we do regular ASSERT action on the terminal symbol.
/// Otherwise we use a PEEK action.

fn set_transition_type(t_pack: &mut TPack, node_index: usize, depth: u32) {
  t_pack.get_node_mut(node_index).set_type(match depth {
    0 => TST::O_ASSERT,
    _ => TST::O_PEEK,
  })
}

type TermAndEndItemGroups = (Vec<(Item, Item)>, Vec<(Item, Item)>);

/// Retrieve terminal items derived from a
/// completed item.
///
/// ### Returns
/// A vector of Item tuples, where the first item is the
/// previous link item for closure resolution, and the second item is
/// a non-end item  produced from the reduction of the
/// `root_end_item`.
fn scan_items(g: &GrammarStore, t_pack: &mut TPack, root_end_item: &Item) -> TermAndEndItemGroups {
  let mut seen = BTreeSet::<(Item, Item)>::new();
  let mut out = BTreeSet::<(Item, Item)>::new();
  let mut fin_items = BTreeSet::<(Item, Item)>::new();

  static empty_vec: Vec<Item> = Vec::new();
  // Starting at the top we grab the closure to the nearest
  // non-term link.

  // Stores the end item [1] and its immediate closure item [0]
  let mut end_items: VecDeque<(Item, Item)> =
    VecDeque::from_iter(vec![(t_pack.get_closure_link(root_end_item), *root_end_item)]);
  while let Some((closure_link, end_item)) = end_items.pop_front() {
    if seen.insert((closure_link, end_item)) {
      let prod = end_item.get_prod_id(g);
      // Grab all productions from the closure that match the end item's
      // production.
      match {
        if closure_link.is_null() {
          if let Some(closures) =
            t_pack.scoped_closures.get(closure_link.get_state().get_closure_index())
          {
            closures.iter()
          } else {
            empty_vec.iter()
          }
        } else if closure_link.is_out_of_scope() {
          if let Some(closure) = &t_pack.goto_scoped_closure {
            closure.iter()
          } else {
            empty_vec.iter()
          }
        } else {
          get_closure_cached(&closure_link, g).iter()
        }
        .filter(|i| i.get_production_id_at_sym(g) == prod)
        .cloned()
        .map(|i| i.to_origin(closure_link.get_origin()))
        .collect::<Vec<_>>()
      } {
        empty if empty.is_empty() => {
          if closure_link.is_null() {
            // t_pack.set_closure_link(end_item, nonterm_closure_link);
            fin_items.insert((closure_link, end_item));
          } else {
            end_items.push_back((t_pack.get_closure_link(&closure_link), end_item));
          }
        }
        prod_items => {
          for p_item in prod_items {
            let inc_item = p_item.to_state(end_item.get_state()).increment().unwrap();

            if inc_item.is_end() {
              end_items.push_back((closure_link, inc_item));
            } else {
              t_pack.set_closure_link(inc_item, closure_link);
              out.insert((closure_link, inc_item));
            }
          }
        }
      }
    }
  }
  (out.into_iter().collect::<Vec<_>>(), fin_items.into_iter().collect::<Vec<_>>())
}

fn process_terminal_node(
  g: &GrammarStore,
  t_pack: &mut TPack,
  term_items: &Vec<Item>,
  parent_index: usize,
) -> Vec<usize> {
  let sym = term_items[0].get_symbol(g);

  let new_node = TGN::new(t_pack, sym, parent_index, term_items.to_vec());

  let node_index = t_pack.insert_node(new_node);

  t_pack.get_node_mut(node_index).set_type(TST::I_CONSUME | TST::O_TERMINAL);

  t_pack.queue_node(node_index);

  vec![node_index]
}

fn create_production_call(
  g: &GrammarStore,
  t_pack: &mut TPack,
  prod_id: ProductionId,
  nonterm_items: Vec<Item>,
  par_id: usize,
) -> Vec<usize> {
  let mut node =
    TGN::new(t_pack, SymbolID::Production(prod_id, GrammarId(0)), par_id, nonterm_items);

  node.prod_sym = Some(SymbolID::Production(prod_id, GrammarId(0)));

  node.trans_type |= TST::O_PRODUCTION;

  let node_index = t_pack.insert_node(node);

  t_pack.queue_node(node_index);

  vec![node_index]
}

fn process_end_item(
  g: &GrammarStore,
  t_pack: &mut TPack,
  end_item: Item,
  parent_index: usize,
) -> Vec<usize> {
  t_pack.gotos.insert(end_item);

  if t_pack.is_scanner && !t_pack.starts.contains(&end_item.to_start().to_zero_state()) {
    // We need to be in the initial closure before we can allow
    // a complete scanner run. Thus, the production of the end state
    // is used to select the next set of items, continuing the scan process
    // the states until we arrive at an end_item that is indeed
    // directly connected to the initial closure.

    let (term_items, end_items) = scan_items(g, t_pack, &end_item);

    // Filter out items automatically handled by goto
    let non_goto_items =
      term_items.iter().filter(|(_, i)| i.get_offset() > 1).cloned().collect::<Vec<_>>();

    if !non_goto_items.is_empty() {
      let items = non_goto_items
        .into_iter()
        .map(|(_, i)| i)
        .chain(end_items.into_iter().map(|(_, i)| i))
        .collect::<Vec<_>>();

      let node = TGN::new(t_pack, SymbolID::EndOfFile, parent_index, items);

      let node_index = t_pack.insert_node(node);

      let results = process_node(g, t_pack, node_index, parent_index, false);

      t_pack.drop_node(&node_index);

      return vec![node_index];
    }
  }

  let end_node = TGN::new(t_pack, SymbolID::EndOfFile, parent_index, vec![end_item]);

  let node_index = t_pack.insert_node(end_node);

  t_pack.get_node_mut(node_index).set_type(TST::I_END | TST::O_ASSERT);
  t_pack.leaf_nodes.push(node_index);

  vec![node_index]
}
