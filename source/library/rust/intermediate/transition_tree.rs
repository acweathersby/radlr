use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    hash::Hash,
    rc::Rc,
    vec,
};

use crate::{
    grammar::{get_closure, get_closure_cached, get_production_start_items},
    primitives::{
        GrammarId, GrammarStore, Item, ProductionId, SymbolID, TransitionGraphNode as TGN,
        TransitionMode, TransitionPack as TPack, TransitionStateType as TST,
    },
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum TransitionGroupSelector {
    OutOfScope(SymbolID),
    EndItem(u64),
    Symbol(SymbolID),
}

const OUT_OF_SCOPE_STATE: u32 = u32::MAX;
///
/// Constructs an initial transition tree that parses a production using a recursive
/// descent strategy. Productions that are ambiguous or are left recursive cannot be
/// parsed, so this tries to do its best to define a parse path for a production
/// before we have to resort to LR based parse strategies.
pub fn construct_recursive_descent<'a>(
    production_id: &ProductionId,
    grammar: &'a GrammarStore,
) -> TPack<'a> {
    let items: Vec<Item> = get_production_start_items(production_id, grammar)
        .into_iter()
        .enumerate()
        .map(|(i, item)| item.to_state(i as u32))
        .collect();

    let mut tpack = TPack::new(production_id, grammar, TransitionMode::RecursiveDescent);

    for item in &items {
        tpack
            .scoped_closures
            .push(get_closure_cached(item, grammar));
    }

    let root_index = tpack.insert_node(TGN::new(
        &tpack,
        SymbolID::Production(*production_id, GrammarId(0)),
        usize::MAX,
        items,
    ));

    tpack.get_node_mut(root_index).set_is(TST::I_DESCENT_START);

    tpack.queue_node(root_index);

    while let Some(node_index) = tpack.get_next_queued() {
        process_node(node_index, grammar, &mut tpack, node_index != 0);
    }

    tpack
}

#[cfg(test)]
mod transition_tree_tests {
    use crate::{debug::compile_test_grammar, grammar::get_production_by_name};

    use super::construct_recursive_descent;

    #[test]
    pub fn test_construct_descent_on_basic_grammar() {
        let grammar = compile_test_grammar("<> A > \\h \\e \\l \\l \\o");

        let prod_id = get_production_by_name("A", &grammar).unwrap();

        let result = construct_recursive_descent(&prod_id, &grammar);

        assert_eq!(result.get_node_len(), 7);
        assert_eq!(result.leaf_nodes.len(), 1);
    }
}
pub fn construct_goto<'a>(
    production_id: &ProductionId,
    grammar: &'a GrammarStore,
    goto_items: &Vec<Item>,
) -> TPack<'a> {
    let mut tpack = TPack::new(production_id, grammar, TransitionMode::GoTo);

    let closure = Rc::new(Box::<Vec<Item>>::new(if tpack.is_scanner {
        vec![]
    } else {
        get_follow_closure(goto_items, grammar, &tpack)
    }));

    let mut parent = TGN::new(
        &tpack,
        SymbolID::Production(*production_id, GrammarId(0)),
        TGN::OrphanIndex,
        goto_items.clone(),
    );
    parent.set_is(TST::I_GOTO_START);

    let parent_index = tpack.insert_node(parent);

    for (production_id, group) in get_goto_starts(&tpack.root_production, grammar, goto_items) {
        tpack.goto_scoped_closure = Some(closure.clone());

        let items: Vec<Item> = group
            .iter()
            .map(|i| {
                let stated_item = i.increment().unwrap();
                let stated_item = if stated_item.at_end() {
                    stated_item.to_state(OUT_OF_SCOPE_STATE)
                } else {
                    tpack
                        .scoped_closures
                        .push(get_closure_cached(&stated_item, grammar));
                    stated_item.to_state((tpack.scoped_closures.len() - 1) as u32)
                };

                stated_item
            })
            .collect();

        let mut goto_node = TGN::new(
            &tpack,
            SymbolID::Production(production_id, GrammarId::default()),
            parent_index,
            items.clone(),
        );

        goto_node.set_is(TST::O_GOTO);

        if (group.len() > 0 && (group.iter().any(|i| i.increment().unwrap().at_end())))
            || tpack.root_production == production_id
        {
            if tpack.root_production == production_id {
                let mut reducible: Vec<Item> = grammar
                    .lr_items
                    .get(&production_id)
                    .unwrap_or(&HashSet::new())
                    .iter()
                    .filter(|i| !group.iter().any(|g| *g == **i))
                    .map(|i| i.increment().unwrap().to_state(OUT_OF_SCOPE_STATE))
                    .collect();

                goto_node.items.append(&mut reducible);
            }

            create_peek(tpack.insert_node(goto_node), items, grammar, &mut tpack)
        } else {
            process_node(tpack.insert_node(goto_node), grammar, &mut tpack, false)
        }
    }

    tpack.scoped_closures = vec![];

    let out_tpack = tpack.clean();

    //out_tpack.nodes = tpack.nodes;
    //  out_tpack.leaf_nodes = tpack.leaf_nodes;
    //out_tpack.goto_items = tpack.goto_items;

    out_tpack
}

pub fn get_goto_starts(
    root_production: &ProductionId,
    grammar: &GrammarStore,
    seed_items: &Vec<Item>,
) -> Vec<(ProductionId, HashSet<Item>)> {
    type GotoMap = BTreeMap<ProductionId, HashSet<Item>>;

    let mut local_lr_items = GotoMap::new();
    let mut output_items = GotoMap::new();
    let mut batch = VecDeque::from_iter(seed_items.iter().map(|i| i.get_production_id(grammar)));

    fn insert(goto_items: &mut GotoMap, production_id: &ProductionId, item: Item) {
        if !goto_items.contains_key(production_id) {
            goto_items.insert(*production_id, HashSet::<Item>::new());
        }

        goto_items.get_mut(production_id).unwrap().insert(item);
    }

    for start_item in get_production_start_items(root_production, grammar) {
        for item in get_closure_cached(&start_item, grammar) {
            if !item.at_end() && item.is_nonterm(grammar) {
                insert(
                    &mut local_lr_items,
                    &item.get_production_id_at_sym(grammar),
                    *item,
                )
            }
        }
    }

    while let Some(production_id) = batch.pop_front() {
        if !output_items.contains_key(&production_id) && local_lr_items.contains_key(&production_id)
        {
            let items = local_lr_items.get(&production_id).unwrap();

            for item in items {
                batch.push_back(item.get_production_id(grammar));
            }

            output_items.insert(production_id, items.clone());
        }
    }

    output_items.into_iter().collect()
}

pub fn get_follow_closure(
    goto_items: &Vec<Item>,
    grammar: &GrammarStore,
    tpack: &TPack,
) -> Vec<Item> {
    let mut productions_to_process = VecDeque::<ProductionId>::new();
    let mut seen_productions = BTreeSet::<ProductionId>::new();
    productions_to_process.push_back(tpack.root_production);

    let mut output = HashSet::new();

    while let Some(production_id) = productions_to_process.pop_front() {
        if !seen_productions.insert(production_id) {
            continue;
        }

        let items: Vec<Item> = grammar
            .lr_items
            .get(&production_id)
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|i| i.increment().unwrap())
            .collect();

        for item in items {
            if item.is_end() {
                productions_to_process.push_back(item.get_production_id(grammar));
            }
            output.insert(item.decrement().unwrap());
        }

        seen_productions.insert(production_id);
    }

    output.into_iter().collect()
}

fn process_node(
    parent_index: usize,
    grammar: &GrammarStore,
    tpack: &mut TPack,
    allow_increment: bool,
) {
    let node = &mut tpack.get_node(parent_index);

    let mut items: Vec<Item> = node.items.iter().cloned().collect();

    if allow_increment {
        items = items
            .into_iter()
            .map(|i| {
                if i.at_end() {
                    i
                } else {
                    i.increment().unwrap()
                }
            })
            .collect();
    }

    //Remove direct left recursive items
    items = items
        .into_iter()
        .filter(|i| {
            i.get_offset() > 0 || i.get_production_id_at_sym(grammar) != tpack.root_production
        })
        .collect();

    let n_term: Vec<Item> = items
        .iter()
        .filter(|i| i.is_term(grammar))
        .cloned()
        .collect();

    let n_nonterm: Vec<Item> = items
        .iter()
        .filter(|i| i.is_nonterm(grammar))
        .cloned()
        .collect();

    let n_end: Vec<Item> = items.iter().filter(|i| i.is_end()).cloned().collect();

    if n_nonterm.len() > 0 {
        let production_ids = HashSet::<ProductionId>::from_iter(
            n_nonterm
                .iter()
                .map(|i| i.get_production_id_at_sym(grammar)),
        );

        if n_term.len() == 0
            && n_end.len() == 0
            && production_ids.len() == 1
            && (node.depth > 1 || non_recursive(&n_nonterm, &tpack.root_production, grammar))
        {
            create_production_call(
                *production_ids.iter().next().unwrap(),
                grammar,
                tpack,
                n_nonterm,
                parent_index,
            );
        } else {
            create_peek(parent_index, items, grammar, tpack);
        }
    } else if n_end.len() == 1 && n_term.len() == 0 {
        // A single end item
        process_end_item(grammar, tpack, n_end[0], parent_index);
    } else if n_end.len() == 0 && n_term.len() == 1 {
        // A single terminal item
        process_terminal_node(grammar, tpack, n_term[0], parent_index);
    } else if n_end.len() == 0 && n_term.len() > 1 {
        // Multiple terminal items
        // TODO: Attempt to disambiguate symbols and create transitions.
        create_peek(parent_index, items, grammar, tpack);
    } else {
        // Multiple terminal and end items
        create_peek(parent_index, items, grammar, tpack);
    }
}
///
/// True if the closure of the givin items does not include
/// the root production on the right of the cursor
fn non_recursive(
    items: &Vec<Item>,
    target_production: &ProductionId,
    grammar: &GrammarStore,
) -> bool {
    for item in get_closure(items, grammar) {
        match item.get_symbol(grammar) {
            SymbolID::Production(production, _) => {
                if production == *target_production {
                    return false;
                }
            }
            _ => {}
        }
    }

    true
}

fn debug_items(comment: &str, items: &[Item], grammar: &GrammarStore) {
    println!("{}", comment);
    for item in items {
        println!("{}", item.debug_string(grammar));
    }
}

fn create_peek(parent_index: usize, items: Vec<Item>, grammar: &GrammarStore, tpack: &mut TPack) {
    let mut depth = 0;

    debug_items("create_peek start", &items, grammar);

    {
        let parent = tpack.get_node_mut(parent_index);
        parent.set_is(TST::I_PEEK_ROOT);
        depth = parent.depth;
        parent.depth = 99999;
    }

    let goals: Vec<usize> = hash_group(items, |i, item| {
        if item.get_state() == 5000 {
            TransitionGroupSelector::OutOfScope(item.get_symbol(grammar))
        } else if item.at_end() {
            TransitionGroupSelector::EndItem(i as u64)
        } else {
            TransitionGroupSelector::EndItem(i as u64)
            //TransitionGroupSelector::Symbol(item.get_symbol(grammar))
        }
    })
    .iter()
    .enumerate()
    .map(|(i, group)| {
        debug_items("peek root group", &group, grammar);
        let root_closure_index = group[0].get_state();

        let items: Vec<Item> = group
            .into_iter()
            .map(|i| i.to_state(root_closure_index))
            .collect();

        let mut root_node = TGN::new(
            tpack,
            items[0].get_symbol(grammar).to_owned(),
            usize::MAX,
            items.clone(),
        );

        root_node.depth = 0;

        if tpack.mode == TransitionMode::GoTo {
            if items[0].at_end() {
                root_node.transition_type |= TST::I_COMPLETE;
            }
        }

        tpack.insert_node(root_node)
    })
    .collect();

    let mut peek_nodes = vec![];

    for (i, goal_index) in goals.iter().cloned().enumerate() {
        let items = tpack.get_node(goal_index).items.to_owned();
        let state = 10000 * (i as u32 + 1);
        let mut term_items = vec![];

        for item in items {
            for closure_item in get_closure(&vec![item], grammar) {
                if !closure_item.is_nonterm(grammar) {
                    tpack.link_peek_closure(closure_item.to_state(state), item);
                    term_items.push(closure_item.to_state(state));
                }
            }
        }

        for item in term_items {
            let mut node = TGN::new(tpack, item.get_symbol(grammar), parent_index, vec![item]);
            node.goal = goal_index;
            node.depth = tpack.get_node(goal_index).depth;
            peek_nodes.push(tpack.insert_node(node));
        }
    }

    let mut leaves = vec![];

    disambiguate(grammar, tpack, peek_nodes, &mut leaves, 0, parent_index);

    tpack.get_node_mut(parent_index).depth = depth;

    process_peek_leaves(grammar, tpack, leaves);

    // All goal nodes can be recycled, as copy operations where used to insert
    // goal nodes as children of peek leaves
    for goal in goals {
        tpack.prune_leaf(&goal);
    }

    tpack.clear_peek_closures();
}
///
/// Constructs a Vector of Vectors, each of which contain items that have been grouped by
/// the hash of a common distinguisher.
fn hash_group<T: Copy + Sized, R: Hash + Sized + Eq, Function: Fn(usize, T) -> R>(
    vector: Vec<T>,
    hash_yielder: Function,
) -> Vec<Vec<T>> {
    let mut hash_groups = HashMap::<R, Vec<T>>::new();

    for (i, val) in vector.iter().enumerate() {
        let hash_value = hash_yielder(i, *val);
        if !hash_groups.contains_key(&hash_value) {
            hash_groups.insert(hash_value, vec![*val]);
        } else {
            hash_groups.get_mut(&hash_value).unwrap().push(*val)
        }
    }

    hash_groups.into_values().collect()
}

///
/// Places goal nodes onto the tips of peek leaf nodes to complete
/// a peek parse path. We then resume constructing the transition graph from
/// the goal nodes onward.
///
/// ### Notes
/// - #### Graph Structure
/// Multiple peek leaves may resolve to a single goal, so we make sure we only
/// continue construction of the transition graph using a single goal node by
/// allowing that node to point to multiple parent nodes by way of the
/// `proxy_parent` vector.
///
/// - #### Scanner Nodes
/// Scanners do not peek, but instead greedily match characters
/// as they traverse the disambiguating path, yielding the lengths of disambiguated
/// tokens. Thus, when we reach leaf peek nodes of a scanner scope, we simply complete
/// any outstanding items and yield goto productions to be resolved within the goto
/// states. The transitions of all parent nodes are reconfigured as ASSERT_CONSUME.
fn process_peek_leaves(grammar: &GrammarStore, tpack: &mut TPack, leaves: Vec<usize>) {
    for peek_leaf_group in hash_group(leaves, |_, leaf| tpack.get_node(leaf).goal)
        .iter()
        .cloned()
    {
        let primary_peek_parent_index = peek_leaf_group[0];
        let prime_node = tpack.get_node(primary_peek_parent_index);
        let goal_index = prime_node.goal;
        let goal_node = tpack.get_node(goal_index);

        if goal_node.is(TST::I_OUT_OF_SCOPE) {
            for node_index in peek_leaf_group {
                tpack
                    .get_node_mut(node_index)
                    .set_is(TST::I_FAIL | TST::I_OUT_OF_SCOPE)
            }
        } else if tpack.is_scanner {
            for node_index in peek_leaf_group {
                {
                    let node = tpack.get_node_mut(node_index);
                    node.unset_is(TST::O_PEEK);
                }

                let mut iter_index = node_index;

                while !tpack.get_node(iter_index).is(TST::I_PEEK_ROOT) {
                    let node = tpack.get_node_mut(iter_index);
                    node.unset_is(TST::O_PEEK);
                    node.set_is(TST::I_CONSUME);
                    iter_index = node.parent;
                }

                let node = tpack.get_node_mut(node_index);

                if node.items.iter().any(|i| i.at_end()) {
                    node.unset_is(TST::I_CONSUME);
                } else {
                    node.set_is(TST::I_CONSUME);
                }

                process_node(node_index, grammar, tpack, false);
            }
        } else {
            let mut copy = (*goal_node).to_owned();

            copy.parent = primary_peek_parent_index;

            // Add the remaining peek leaves as additional parents of the goal
            // node.
            copy.proxy_parents
                .append(&mut peek_leaf_group[1..].to_owned());

            process_node(tpack.insert_node(copy), grammar, tpack, true);
        }
    }
}

fn disambiguate(
    grammar: &GrammarStore,
    tpack: &mut TPack,
    node_ids: Vec<usize>,
    leaves: &mut Vec<usize>,
    depth: u32,
    parent_index: usize,
) {
    let mut term_nodes = vec![];
    let mut end_nodes = vec![];

    // We must first complete end-items and generate new
    // nodes that arise from the completion of a production.

    for node_index in node_ids {
        let item = &tpack.get_node(node_index).items[0].clone();

        if !item.is_end() {
            term_nodes.push(node_index)
        } else {
            let discard = HashSet::<Item>::new();
            if depth == 0
                && tpack.mode == TransitionMode::GoTo
                && item.decrement().unwrap().is_nonterm(grammar)
            {
                // TODO: Remove items that present in other branches to prevent
                // item aliasing.
            }

            let mut need_to_prune = false;
            let scan_items = scan_items(item, grammar, tpack);
            for term_node in
                create_term_nodes_from_items(&scan_items, grammar, tpack, node_index, &discard)
            {
                term_nodes.push(tpack.insert_node(term_node));
                need_to_prune = true;
            }

            if need_to_prune {
                tpack.prune_leaf(&node_index);
            } else {
                end_nodes.push(node_index);
            }
        }
    }

    match end_nodes.len() {
        0 => {}
        1 => {
            set_transition_type(end_nodes[0], tpack);
            leaves.push(end_nodes[0]);
        }
        _ => {
            let roots = get_roots(&end_nodes, &tpack);
            if roots.len() == 1 {
                for end_node in &end_nodes[1..] {
                    tpack.prune_leaf(end_node);
                }
                set_transition_type(end_nodes[0], tpack);
                leaves.push(end_nodes[0]);
            } else {
                let mut parent = 0;
                let sym = tpack.get_node(end_nodes[0]).sym;
                for end_node in &end_nodes {
                    parent = tpack.prune_leaf(end_node)
                }
                handle_unresolved_roots(parent, sym, roots, leaves, grammar, tpack);
            }
        }
    }

    let mut groups = hash_group(term_nodes, |_, n| tpack.get_node(n).sym.clone());
    let mut next_peek_groups = vec![];

    merge_occluding_groups(&mut groups, tpack, grammar);

    for group in groups.iter() {
        let prime_node_index = group[0];
        let roots = get_roots(&group, tpack);

        set_transition_type(prime_node_index, tpack);

        if roots.len() > 1 && groups.len() > 1 {
            let mut peek_transition_group = vec![];
            for node_index in group.iter().cloned() {
                let transition_on_skipped_symbol =
                    tpack.get_node(node_index).is(TST::I_SKIPPED_COLLISION);
                for node in create_term_nodes_from_items(
                    &tpack
                        .get_node(node_index)
                        .items
                        .clone()
                        .into_iter()
                        .map(|i| {
                            if transition_on_skipped_symbol {
                                i
                            } else {
                                i.increment().unwrap()
                            }
                        })
                        .collect::<Vec<Item>>(),
                    grammar,
                    tpack,
                    prime_node_index,
                    &HashSet::new(),
                ) {
                    peek_transition_group.push(tpack.insert_node(node));
                }
            }

            if peek_transition_group.is_empty() {
                if get_roots(&group, tpack)
                    .iter()
                    .all(|i| tpack.get_node(*i).is(TST::I_OUT_OF_SCOPE))
                {
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

        for node_index in group.iter().cloned() {
            if node_index != prime_node_index {
                let items = tpack.get_node(node_index).items.clone();
                tpack.prune_leaf(&node_index);
                tpack
                    .get_node_mut(prime_node_index)
                    .items
                    .append(&mut items.clone());
            }
        }
    }

    for peek_group in next_peek_groups {
        if peek_group.len() > 0 {
            if handle_shift_reduce_conflicts(&peek_group, grammar, tpack, leaves) {
                continue;
            } else if groups_are_aliased(&peek_group, tpack)
                || group_is_repeated(&peek_group, tpack)
            {
                handle_transition_collision(&peek_group, grammar, tpack, leaves);
            } else {
                disambiguate(grammar, tpack, peek_group, leaves, depth + 1, parent_index);
            }
        }
    }
}

fn group_is_repeated(peek_group: &Vec<usize>, tpack: &mut TPack) -> bool {
    //panic!("Todo: group_is_repeated")
    false
}

fn handle_transition_collision(
    peek_group: &Vec<usize>,
    grammar: &GrammarStore,
    tpack: &mut TPack,
    leaves: &mut Vec<usize>,
) {
    panic!("Todo: handle_shift_reduce_conflicts")
}

fn handle_shift_reduce_conflicts(
    peek_group: &Vec<usize>,
    grammar: &GrammarStore,
    tpack: &mut TPack,
    leaves: &mut Vec<usize>,
) -> bool {
    let goals = get_roots(peek_group, tpack)
        .into_iter()
        .map(|i| (i, &tpack.get_node(i).items))
        .collect::<Vec<_>>();

    if goals.len() == 2 {
        if goals[0].1.len() == 1 && goals[1].1.len() == 1 && (tpack.mode == TransitionMode::GoTo)
            || (goals[0].1[0].get_production_id(grammar)
                == goals[1].1[0].get_production_id(grammar))
        {
            let shift = goals
                .iter()
                .filter(|(_, i)| !i[0].is_end())
                .collect::<Vec<_>>();
            let reduce = goals
                .iter()
                .filter(|(_, i)| i[0].is_end())
                .collect::<Vec<_>>();
            if !shift.is_empty() && !reduce.is_empty() {
                let shift_goal = shift[0].0;
                for node_index in peek_group {
                    if tpack.get_node(*node_index).goal == shift_goal {
                        leaves.push(*node_index)
                    } else {
                        tpack.prune_leaf(node_index);
                    }
                }
                return true;
            }
        }
    }

    false
}

fn groups_are_aliased(peek_group: &Vec<usize>, tpack: &mut TPack) -> bool {
    hash_group(peek_group.to_owned(), |_, n| {
        tpack
            .get_node(n)
            .items
            .clone()
            .iter()
            .map(|i| i.to_state(0))
            .collect::<Vec<_>>()
            .sort()
    })
    .iter()
    .any(|g| g.len() > 1)
}

///
/// Compares the terminal symbols of node groups and merges those
/// groups whose terminal symbols occlude each other.
///
/// For instance, given a group `A` with the symbol `g:id` and an other group `B` with symbol `\g`,
/// the character `g` could be accepted by either group. As long as group `A` (the "defined" group)
/// is not exclusive, we merge group `B` into `A` to into account the ambiguous nature of the groups.
fn merge_occluding_groups(groups: &mut Vec<Vec<usize>>, tpack: &TPack, grammar: &GrammarStore) {
    // Clone the from_group store so we are able
    // to merge its members into to_groups without
    // going fowl of the borrow checker.
    for from_group in groups.clone() {
        let from_sym = tpack.get_node(from_group[0]).sym;
        for to_group in groups.iter_mut() {
            let to_sym = tpack.get_node(to_group[0]).sym;

            if from_sym == to_sym {
                continue;
            }

            if symbols_occlude(&to_sym, &from_sym, grammar) {
                to_group.append(&mut from_group.clone());
            }
        }
    }
}
///
/// Compares whether symbolB occludes symbolA
/// ( produces an ambiguous parse path )
///
/// Symbols that can occlude are as follows
///
/// - `g:id` and any single identifier character.
/// - `g:num` and any single numeric character.
/// - `g:sym` and any single character thats not a numeric, identifier, space, newline, or tab.
///
fn symbols_occlude(symA: &SymbolID, symB: &SymbolID, grammar: &GrammarStore) -> bool {
    match symA {
        SymbolID::DefinedGeneric(_) => match symB {
            SymbolID::GenericSymbol => {
                grammar.symbols_table.get(symA).unwrap().code_point_length == 1
            }
            _ => false,
        },
        SymbolID::DefinedIdentifier(_) => match symB {
            SymbolID::GenericIdentifier => {
                grammar.symbols_table.get(symA).unwrap().code_point_length == 1
            }
            _ => false,
        },
        SymbolID::DefinedNumeric(_) => match symB {
            SymbolID::GenericNumber => {
                grammar.symbols_table.get(symA).unwrap().code_point_length == 1
            }
            _ => false,
        },
        _ => false,
    }
}

fn get_roots(end_nodes: &Vec<usize>, tpack: &TPack) -> Vec<usize> {
    HashSet::<usize>::from_iter(end_nodes.iter().map(|n| tpack.get_node(*n).goal))
        .into_iter()
        .collect::<Vec<usize>>()
}

fn handle_unresolved_roots(
    parent_index: usize,
    sym: SymbolID,
    roots: Vec<usize>,
    leaves: &Vec<usize>,
    grammar: &GrammarStore,
    tpack: &TPack,
) {
    panic!("TODO: handle_unresolved_roots")
}

///
/// Set the transition type of a peeking based on whether
/// it is first node in the peek path or not. If it is the first
/// node, we do regular ASSERT action on the terminal symbol. Otherwise we use
/// a PEEK action.
fn set_transition_type(node_index: usize, tpack: &mut TPack) {
    let mut node = tpack.get_node(node_index);

    if (node.depth % 10000) > 0 && node.depth >= 10000 {
        tpack.get_node_mut(node_index).set_is(TST::O_PEEK);
    } else {
        tpack.get_node_mut(node_index).set_is(TST::O_ASSERT);
    }
}

fn create_term_nodes_from_items(
    items: &Vec<Item>,
    grammar: &GrammarStore,
    tpack: &mut TPack,
    parent_index: usize,
    discard: &HashSet<Item>,
) -> Vec<TGN> {
    let mut term_nodes = vec![];

    for item in items {
        debug_items(
            &format!("item:: {} :", &item.debug_string(grammar)),
            &get_closure(&vec![*item], grammar),
            grammar,
        );
        for closure_item in get_closure(&vec![*item], grammar) {
            if !closure_item.is_nonterm(grammar) && !discard.contains(item) {
                let closure_item = closure_item.to_state(item.get_state());

                tpack.link_peek_closure(closure_item, *item);

                let node = TGN::new(
                    tpack,
                    item.get_symbol(grammar),
                    parent_index,
                    vec![closure_item],
                );
                term_nodes.push(node);
            }
        }
    }

    term_nodes
}

///
/// Retrieve terminal items derived from a
/// completed item.
fn scan_items(root_end_item: &Item, grammar: &GrammarStore, tpack: &mut TPack) -> Vec<Item> {
    let mut items = VecDeque::<Item>::new();
    let mut seen = HashSet::<Item>::new();
    let mut out = HashSet::<Item>::new();

    items.push_back(*root_end_item);

    // Compile closure data

    let mut local_closure = Vec::<Item>::with_capacity(64);
    let mut lookup_item = root_end_item;
    let empty = vec![];

    loop {
        if let Some(closure_item) = tpack.peek_scoped_closures_linked_lookups.get(lookup_item) {
            for item in get_closure_cached(&closure_item.to_zero_state(), grammar) {
                if item.is_end() || item.is_nonterm(grammar) {
                    local_closure.push(*item)
                }
            }
            lookup_item = closure_item
        } else {
            for item in if tpack.mode == TransitionMode::GoTo
                && lookup_item.get_state() == OUT_OF_SCOPE_STATE
            {
                match &tpack.goto_scoped_closure {
                    Some(closure) => closure.iter(),
                    None => empty.iter(),
                }
            } else {
                tpack.scoped_closures[lookup_item.get_state() as usize].iter()
            } {
                if item.is_end() || item.is_nonterm(grammar) {
                    local_closure.push(*item)
                }
            }
            break;
        }
    }

    while let Some(end_item) = items.pop_front() {
        if !seen.insert(end_item) {
            continue;
        }

        let production = end_item.get_production_id(grammar);

        let production_items = local_closure
            .iter()
            .filter(|i| i.get_production_id_at_sym(grammar) == production)
            .cloned();

        for non_term in production_items {
            let non_term = non_term.to_state(root_end_item.get_state());
            let target_item = non_term
                .increment()
                .unwrap()
                .to_state(root_end_item.get_state());
            tpack.link_peek_closure(non_term, *root_end_item);
            if target_item.at_end() {
                items.push_back(target_item);
            } else {
                tpack.link_peek_closure(target_item, non_term);
                out.insert(target_item);
            }
        }
    }

    out.into_iter().collect()
}

fn process_end_item(
    grammar: &GrammarStore,
    tpack: &mut TPack,
    end_item: Item,
    parent_index: usize,
) {
    let end_node = TGN::new(tpack, SymbolID::EndOfFile, parent_index, vec![end_item]);
    let node_index = tpack.insert_node(end_node);
    tpack
        .get_node_mut(node_index)
        .set_is(TST::I_END | TST::O_ASSERT);
    tpack.goto_items.insert(end_item);
    tpack.leaf_nodes.push(node_index);
}

fn process_terminal_node(
    grammar: &GrammarStore,
    tpack: &mut TPack,
    term_item: Item,
    parent_index: usize,
) -> usize {
    let sym = term_item.get_symbol(grammar);
    let new_node = TGN::new(tpack, sym, parent_index, vec![term_item]);
    let node_index = tpack.insert_node(new_node);
    tpack
        .get_node_mut(node_index)
        .set_is(TST::I_CONSUME | TST::O_ASSERT);
    tpack.queue_node(node_index);
    node_index
}
fn create_production_call(
    production: ProductionId,
    grammar: &GrammarStore,
    tpack: &mut TPack,
    nonterm_items: Vec<Item>,
    parent_index: usize,
) {
    let mut node = TGN::new(
        tpack,
        SymbolID::Production(production, GrammarId(0)),
        parent_index,
        nonterm_items,
    );

    node.transition_type |= TST::O_PRODUCTION;
    let node_index = tpack.insert_node(node);
    tpack.queue_node(node_index);
}
