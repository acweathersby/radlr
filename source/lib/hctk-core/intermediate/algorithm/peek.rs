//! Functions for resolving a set of ambiguous Items.
use super::{
  follow::get_follow_items,
  lr::construct_inline_LR,
};
use crate::{grammar::hash_id_value_u64, journal::Journal, types::*, intermediate::utils::{hash_group_btreemap, hash_group_vec, symbols_occlude}};
use std::collections::{BTreeSet, VecDeque};
use EdgeType::*;
use NodeType::*;

pub(crate) fn peek(
  t: &mut TransitionGraph,
  j: &mut Journal,
  root_par_id: NodeId,
  items: Items,
  global_depth: usize,
) -> HCResult<()> {
  let grammar = t.g.clone();
  let g = &grammar;
  let mut lane_counter = items.iter().fold(u32::MAX, |u,i| i.get_state().get_lane().min(u).max(1));

  t.get_node_mut(root_par_id).set_attribute(NodeAttributes::I_PEEK_ORIGIN);

  // t.get_node_mut(root_par_id).transition_items.clear();

  // Split items into groups based on symbol Ids.
  let goals = hash_group_btreemap(items.clone(), |index, i| {
    if i.completed() || j.config().enable_breadcrumb_parsing {
      SymbolID::DistinctGroup(index as u32)
    } else if i.is_out_of_scope() {
      SymbolID::OutOfScope
    } else{
      i.get_symbol(g)
    }
  })
  .into_iter()
  .collect::<Vec<_>>();

  // We now create a set of item lanes based on the symbol groups that will persist throughout
  // the peeking process.
  let initial_items = goals
    .iter()
    .enumerate()
    .flat_map(|(index, (_, items))| {
      items.clone().into_iter().map(move |i| i.to_state(i.get_state().to_lane(index as u32 + lane_counter)))
    })
    .collect::<Vec<_>>();


  initial_items.print_items(g, "Initial items");

  lane_counter += initial_items.len() as u32;

  t.get_node_mut(root_par_id).goto_items = initial_items.non_term_item_vec(g);
  // With our items now setup in lanes, we can start disambiguating
  let mut pending_items = VecDeque::from_iter(vec![(root_par_id, 0, initial_items)]);
 
  while let Some((par_id, depth, items)) = pending_items.pop_front() {
    let mut EXCLUSIVE_COMPLETED = false;

    // Sort the items into groups of terminal and completed items.
    let mut closure = items.closure_with_state(g);

    let completed_items = closure.completed_item_vec();

    // Resolve completed items by getting their follow set.
    if completed_items.len() > 0 {

      insert_items_into_node(completed_items.clone(), t, par_id);

      let mut terminal_completions = vec![];
      for completed_item in completed_items {

        let (follow_and_terminal_completed, l) = get_follow_items(t, &completed_item, Some(par_id), lane_counter);

        lane_counter = l;

        terminal_completions
          .append(&mut Vec::from_linked(follow_and_terminal_completed.final_completed_items));


        insert_items_into_node(Vec::from_linked(follow_and_terminal_completed.intermediate_completed_items), t, par_id);
        // Merge follow set into term_items.
        closure.append(
          &mut Vec::from_linked(follow_and_terminal_completed.uncompleted_items)
            .closure_with_state(g),
        )
      }
      // Items that truly in the completed position, that is for a completed item
      //  `I => ... *` there is no follow item `X => ... * I ...` in the closures of
      // the nodes predecessor, are then turned into completed nodes. The catch here
      // is any conflicting completed items may be unresolvable ...
      match terminal_completions.len() {
        1 if j.occlusion_tracking_mode() => {
          // We can skip further processing if in occlusion tracking mode
        }
        1 => {
          let items = terminal_completions;

          if t.is_scanner {
            let exclusive: Vec<&Item> =
              items.iter().filter(|i| i.get_origin_sym().is_exclusive()).collect();
            EXCLUSIVE_COMPLETED = exclusive.iter().any(|i| i.get_origin_sym().is_exclusive());

            #[cfg(debug_assertions)]
            {
              if EXCLUSIVE_COMPLETED {
                j.report_mut().add_note("Exclusive short-circuit", format!(
                  "Short circuiting completion of other items due to one or more exclusive symbols being completed: [\n{}\n]",
                  exclusive.iter().map(|i| format!("    {{ {} => {} }}", 
                  i.debug_string(g), i.get_origin_sym().to_string(g))).collect::<Vec<_>>().join("\n")
              ));
              }
            }
          }

          insert_items_into_node(items.clone(), t, par_id);

          if depth == 0 {
            t.queue_node(ProcessGroup {
              node_index:   par_id,
              items:        get_goal_items(&items, &goals),
              discriminant: Some((SymbolID::Default, items)),
              depth:        global_depth,
            });
          } else if t.is_scanner || j.config().enable_breadcrumb_parsing {
            items.print_items(g, &format!("peek end"));
            let node_index = create_and_insert_node(
              t,
              SymbolID::EndOfInput,
              vec![],
              if t.is_scanner { Complete } else { BreadcrumbEndCompletion},
              Default,
              Some(par_id),
              Some(par_id),
              Vec::default(),
            );
            // Submit these items to be processed.
            t.queue_node(ProcessGroup {
              node_index,
              items: items,
              discriminant: None,
              depth: global_depth,
            });
          } else {
            let node_index = create_and_insert_node(
              t,
              SymbolID::EndOfInput,
              vec![],
              PeekTransition,
              Default,
              Some(par_id),
              Some(root_par_id),
              Vec::default(),
            );

            // Submit these items to be processed.
            t.queue_node(ProcessGroup {
              node_index,
              items: get_goal_items(&items, &goals),
              discriminant: None,
              depth: global_depth,
            });
          }
        }
        2.. => {
          if t.is_scanner {
            resolveConflictingSymbols(t, j, terminal_completions, depth, global_depth, par_id);
          } else {

            items.print_items(g, "Conflicting items");
            return HCResult::Err(HCError::grammar_err_multi_location {
              message:   "Could not resolve production. Grammar has ambiguities.".to_string(),
              locations: terminal_completions
                .iter()
                .map(|i| HCError::grammar_err {
                  message: "Test".to_string(),
                  inline_message: "Test".to_string(),
                  loc: match i.get_origin() {
                    OriginData::RuleId(rule_id) => t.g.get_rule(&rule_id).unwrap().tok.clone(),
                    _ => i.decrement().unwrap().get_rule_ref(g).unwrap().tok.clone(),
                  },
                  path: i.decrement().unwrap().get_rule_ref(g).unwrap().grammar_ref.path.clone(),
                })
                .collect(),
            });
          }
        }
        _ => {}
      }
    }

    let term_items = closure.term_item_vec(g);
    
    insert_items_into_node(term_items.clone(), t, par_id);

    let mut updated_closure = closure.non_term_item_vec(g);


    t.get_node_mut(par_id).goto_items.append(&mut updated_closure);

    if EXCLUSIVE_COMPLETED {
      continue;
    }

    // Create groups of terminal items based on their symbols
    let mut groups =
      hash_group_btreemap(term_items, |_, i| i.get_symbol(g)).into_iter().collect::<Vec<_>>();

    // Merge groups whose symbols occlude.
    merge_occluding_token_groups(t, j, &mut groups);

    for (sym, items) in groups {
      // detect the number of distinct lanes present in the current group.
      // IF this value is 1, then we have successfully found a peek leaf that
      // is ambiguous.

      let peek_groups = hash_group_vec(items.clone(), |_, i| i.get_state().get_lane());

      match peek_groups.len() {
        1 if j.occlusion_tracking_mode() => {
          // We can skip further processing if in occlusions tracking mode
        }
        1 => {
          match items[0].get_state().get_origin() {
            OriginData::OutOfScope => {
              // This symbol belongs to a follow item of the production. In this
              // we simply fail to allow the production to complete using the fall
              // back function
              let index = create_and_insert_node(
                t,
                sym,
                items,
                Fail,
                Assert,
                Some(par_id),
                Some(par_id),
                vec![],
              );

              t.leaf_nodes.push(index);
            }
            //SymbolID::DistinctGroup(_) => {
            //  unreachable!("Completed items should have been taken care of at this point");
            //}
            _ => {
              if depth == 0 {
                // Reprocess the root node (which is always par_id when depth == 0)
                // with the disambiguated items.
                t.queue_node(ProcessGroup {
                  node_index:   par_id,
                  items:        get_goal_items(&items, &goals),
                  discriminant: Some((sym, items)),
                  depth:        global_depth,
                });
              } else if t.is_scanner || j.config().enable_breadcrumb_parsing {
                let node_index = create_and_insert_node(
                  t,
                  sym,
                  vec![],
                  if t.is_scanner { Shift } else { BreadcrumbShiftCompletion },
                  Assert,
                  Some(par_id),
                  Some(par_id),
                  items.try_increment().non_term_item_vec(g),
                );
                // Continue processing the now disambiguated items.
                t.queue_node(ProcessGroup {
                  node_index,
                  items: items.try_increment(),
                  discriminant: None,
                  depth: global_depth,
                });
              } else {
                let goal_items = get_goal_items(&items, &goals);
                let node_index = create_and_insert_node(
                  t,
                  sym,
                  vec![],
                  PeekTransition,
                  get_edge_type(j,t,depth),
                  Some(par_id),
                  Some(root_par_id),
                  goal_items.non_term_item_vec(g),
                );
                // Submit these items to be processed.
                t.queue_node(ProcessGroup {
                  node_index,
                  items: goal_items.clone(),
                  discriminant: None,
                  depth: global_depth,
                });
              }
            }
          }
        }

        2.. => {
          // We combine these items into a new node, then prepare their increments
          // for the next round.

          // TODO: We need to evaluate whether we can continue processing nodes.
          // The condition in which we can't continue are:
          // - Shift-Reduce conflicts

          if !t.peek_ids.insert(hash_id_value_u64(items.clone().to_zero_state().to_set())) {
            // Item set has been repeated
            let (_, goal_items) = get_goal_contents(&items, &goals);
            let lr_starts = goal_items.clone().into_iter().flatten().cloned().collect::<Vec<_>>();
            // We can try to disambiguating using LR parsing:

            // create the root node for the start items
            let root_node = create_node(
              t,
              sym,
              lr_starts.clone(),
              get_node_type(j,t),
              get_edge_type(j,t,depth),
              Some(par_id),
              Some(par_id),
              lr_starts.non_term_item_vec(g),
            );

            match construct_inline_LR(t, j, root_node) {
              HCResult::Ok(_) => {
                // Our grammar is now (RD/RAD + LR)
              }
              _ if !t.is_scanner => {
                // Otherwise, we must use a fork state to handle this situation
                let fork_node_index = create_and_insert_node(
                  t,
                  sym,
                  lr_starts.clone(),
                  Fork,
                  get_edge_type(j,t,depth),
                  Some(par_id),
                  Some(par_id),
                  lr_starts.non_term_item_vec(g),
                );

                for goal_items in goal_items {
                  let fork_base = create_and_insert_node(
                    t,
                    sym,
                    lr_starts.clone(),
                    ForkBase,
                    Default,
                    Some(fork_node_index),
                    Some(fork_node_index),
                    lr_starts.non_term_item_vec(g),
                  );

                  t.queue_node(ProcessGroup {
                    node_index:   fork_base,
                    items:        goal_items.clone(),
                    discriminant: None,
                    depth:        global_depth,
                  });
                }

                // Our grammar is now G + (RD/RAD) + LR?
              }
              _ => {
                panic!("Unable to resolve this grammar!");
              }
            }
          } else {
            if !t.is_scanner && !j.config().enable_breadcrumb_parsing && global_depth == 0 {
              // Check to see if we can issue a call instead of increment.
              // For that to work, all items need to be in an initial state,
              // and the follow items must all have shifted from the same
              // non-terminal.
    /*           let mut call_groups = hash_group_btreemap(items.clone(), |_, i| {
                let (items, _) = get_follow_items(t, &i, Some(par_id), 0);

                if items.final_completed_items.is_empty() {
                  Some(
                    Vec::from_linked(items.uncompleted_items)
                      .into_iter()
                      .map(|i| i.decrement().unwrap().get_symbol(g))
                      .collect::<BTreeSet<_>>(),
                  )
                } else {
                  None
                }
              })
              .into_iter()
              .collect::<Vec<_>>();
              if call_groups.len() == 1 && matches!(&call_groups[0].0, Some(d) if d.len() == 1) {
                let (syms, items) = call_groups.pop().unwrap();

                let mut node =
                  GraphNode::new(&t, sym, Some(par_id), items.to_end(), get_node_type(j,t));
                node.edge_type = get_edge_type(j,t,depth);
                node.prod_sym = Some(syms.unwrap().into_iter().next().unwrap());
                node.closure_parent = Some(par_id);

                // End of the line - Let goto handle the reset.
                // Submit these items to be processed.
                t.goto_seeds.append(&mut items.to_zero_state().to_set());
                continue;
              } */
            }

            // Pack items into new peek node and submit their increments for
            // a new peek round
            let incremented_items = items.try_increment();

            let node_index = create_and_insert_node(
              t,
              sym,
              vec![],
              get_node_type(j,t),
              get_edge_type(j,t,depth),
              Some(par_id),
              Some(par_id),
              incremented_items.non_term_item_vec(g),
            );

            incremented_items.print_items(g, &format!("peek transition on {}", sym.to_string(g)));
            pending_items.push_back((node_index, depth + 1, incremented_items));
          }
        }

        _ => {
          unreachable!("Groups should have at least one lane")
        }
      }
    }
  }

  HCResult::Ok(())
}

fn get_goal_items(items: &Vec<Item>, goals: &Vec<(SymbolID, Vec<Item>)>) -> Vec<Item> {
    let (_, goal_items) = get_goal_contents(items, goals);
    let goal_items = goal_items.into_iter().flatten().cloned().collect();
    goal_items
}

fn get_node_type(j: &Journal, t: &TransitionGraph) -> NodeType {
  if t.is_scanner {
    NodeType::Shift
  } else if j.config().enable_breadcrumb_parsing  {
    NodeType::BreadcrumbTransition
  } else {
    NodeType::PeekTransition
  }
}

fn get_edge_type(j:&Journal, t: &TransitionGraph, depth: usize, ) -> EdgeType {
  if depth > 0 && !t.is_scanner && !j.config().enable_breadcrumb_parsing {
    EdgeType::Peek
  } else {
    EdgeType::Assert
  }
}

fn resolveConflictingSymbols(
  t: &mut TransitionGraph,
  j: &mut Journal,
  completed_symbol_items: Items,
  peek_depth: usize,
  global_depth: usize,
  par_id: NodeId,
) {
  #[cfg(debug_assertions)]
  {
    if !j.occlusion_tracking_mode() {
      // Ensure all items have symbol origins
      debug_assert!(completed_symbol_items.iter().all(|i| {
        match i.get_origin() {
          OriginData::Symbol(_) => true,
          _ => false,
        }
      }));

      println!(
        "\nScan Mode: Conflicting items and their symbols:\n{}\n",
        completed_symbol_items
          .iter()
          .map(|i| match i.get_origin() {
            OriginData::Symbol(sym) =>
              format!("{{ {} => {} }}", i.debug_string(&t.g), sym.to_string(&t.g)),
            _ => String::new(),
          })
          .collect::<Vec<_>>()
          .join("\n")
      );
    }
  }

  if j.occlusion_tracking_mode() {
    let symbols =
      completed_symbol_items.iter().map(|i| i.get_origin_sym()).collect::<BTreeSet<_>>();
    j.add_occlusions(symbols);
  } else {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
    enum SymbolPriorities {
      ExclusiveDefined,
      Defined,
      Production,
      Generic,
    }
    use SymbolPriorities::*;

    // Map items according to their symbols
    let symbol_groups = hash_group_btreemap(completed_symbol_items, |_, i| i.get_origin_sym());
    let priority_groups = hash_group_btreemap(symbol_groups, |_, (sym, _)| match sym {
      sym if sym.is_exclusive() => ExclusiveDefined,
      sym if sym.is_defined() => Defined,
      sym if sym.is_production() => Production,
      _ => Generic,
    });
    let mut complete_items: Option<&Items> = None;

    for (priority, groups) in priority_groups {
      match priority {
        ExclusiveDefined => {
          if groups.len() > 1 {
            panic!(
              "Found {} conflicting Exclusive Defined symbols. Grammar is ambiguous",
              groups.len()
            );
          } else {
            complete_items = Some(groups.values().next().unwrap());
          }
        }
        Defined => {
          if groups.len() > 1 {
            panic!("Found {} conflicting Defined symbols. Grammar is ambiguous", groups.len());
          } else {
            complete_items = Some(groups.values().next().unwrap());
          }
        }
        Production => {
          if groups.len() > 1 {
            panic!(
              "\nFound {} conflicting Token Production symbols. Grammar is ambiguous:\n{}",
              groups.len(),
              groups
                .iter()
                .map(|(s, _)| match s {
                  SymbolID::TokenProduction(prod_id, _) => {
                    t.g.get_production(prod_id).unwrap().loc.blame(
                      1,
                      1,
                      &format!("[ {} ] first defined here", s.to_string(&t.g)),
                      BlameColor::Red,
                    )
                  }
                  _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n")
            );
          } else {
            complete_items = Some(groups.values().next().unwrap());
          }
        }
        Generic => {
          if groups.len() > 1 {
            panic!("Found {} conflicting Generic symbols. Grammar is ambiguous", groups.len());
          } else {
            complete_items = Some(groups.values().next().unwrap());
          }
        }
      }

      if let Some(completed_items) = complete_items {
        if peek_depth == 0 {
          t.queue_node(ProcessGroup {
            node_index:   par_id,
            items:        completed_items.clone(),
            discriminant: Some((SymbolID::Default, completed_items.clone())),
            depth:        global_depth,
          });
        } else {
          let node_index = create_and_insert_node(
            t,
            SymbolID::EndOfInput,
            vec![],
            Complete,
            Default,
            Some(par_id),
            Some(par_id),
            Vec::default(),
          );
          // Submit these items to be processed.
          t.queue_node(ProcessGroup {
            node_index,
            items: completed_items.clone(),
            discriminant: None,
            depth: global_depth,
          });
        }
        return;
      } else {
        panic!("Could not resolve Symbol ambiguities!")
      }
    }
  }
}

fn get_goal_contents<'a>(
  items: &Items,
  goals: &'a Vec<(SymbolID, Items)>,
) -> (Vec<&'a SymbolID>, Vec<&'a Items>) {
  hash_group_btreemap(items.clone(), |_, i| i.get_state().get_lane() - 1)
    .into_iter()
    .map(|(g, i)| {
      let (goal_sym, goal_items) = &goals[g as usize];
      (goal_sym, goal_items)
    })
    .unzip()
}

pub(super) fn insert_items_into_node(mut items: Items, t: &mut TransitionGraph, node_id: NodeId) {
  // par_node.transition_items.append(&mut items.to_origin_only_state());
  // let reduced = par_node.transition_items.clone().to_origin_only_state().to_set().to_vec();
  t.get_node_mut(node_id).transition_items.append(&mut items);
}

/// Compares the terminal symbols of node groups and merges those
/// groups whose terminal symbols occlude each other.
///
/// For instance, given a group `A` with the symbol `g:id` and an
/// other group `B` with symbol `\g`, the character `g` could be
/// accepted by either group. As long as group `A` (the "defined"
/// group) is not exclusive, we merge group `B` into `A` to into
/// account the ambiguous nature of the groups.

fn merge_occluding_token_groups(
  t: &mut TransitionGraph,
  journal: &mut Journal,
  groups: &mut [(SymbolID, Items)],
) {
  // Clone the from_group store so we are able
  // to merge its members into to_groups without
  // going fowl of the borrow checker.

  if !t.is_scanner && !journal.config().allow_occluding_symbols {
    return;
  }

  for i in 0..groups.len() {
    for j in 0..groups.len() {
      if i == j {
        continue;
      }

      let (from_sym, from_group) = &groups[i];
      let (to_sym, to_group) = &groups[j];

      let from_item = from_group[0];
      let to_item = to_group[0];

      let from_origin = from_item.get_origin();
      let to_origin = to_item.get_origin();

      // Scanner items that originate from the same symbol do not require occlusion
      // checking.
      if matches!(from_origin, OriginData::Symbol(..)) && from_origin == to_origin {
        continue;
      }

      if {
        if t.is_scanner {
          symbols_occlude(&to_sym, &from_sym, &t.g)
            && ((!from_item.is_out_of_scope()) || (from_sym.is_defined() || to_sym.is_defined()))
        } else {
          let occlusion_table = journal.get_occlusion_table();

          occlusion_table.get(to_sym).and_then(|f| f.contains(from_sym).then(|| 1)).is_some()
        }
      } {
        #[cfg(debug_assertions)]
        {
          if !journal.occlusion_tracking_mode() {
            journal.report_mut().add_note("Symbol Group Merge", 
            format!(
            "\nDue to the ambiguous symbols [{} ≈ {}] the peek group [\n\n{}\n\n] will be merged into [\n\n{}\n\n]\n",
            to_sym.to_string(&t.g),
            from_sym.to_string(&t.g),
            from_group.to_debug_string(&t.g, "\n"),
            
            groups[j].1.to_debug_string(&t.g, "\n")
          ));
          }
        }
        let mut clone = from_group.clone();
        groups[j].1.append(&mut clone);
      }
    }
  }
}

pub(super) fn create_node(
  t: &TransitionGraph,
  symbol: SymbolID,
  items: Items,
  node_type: NodeType,
  edge_type: EdgeType,
  parent: MaybeNodeId,
  closure_parent: MaybeNodeId,
  goto_items: Items,
) -> GraphNode {
  let mut node = GraphNode::new(t, symbol, parent, items, node_type);

  node.edge_type = edge_type;
  node.closure_parent = closure_parent;
  node.goto_items = goto_items;

  node
}

pub(super) fn create_and_insert_node(
  t: &mut TransitionGraph,
  symbol: SymbolID,
  items: Items,
  node_type: NodeType,
  edge_type: EdgeType,
  parent: MaybeNodeId,
  closure_parent: MaybeNodeId,
  goto_items: Items,
) -> NodeId {
  let node =
    create_node(t, symbol, items, node_type, edge_type, parent, closure_parent, goto_items);
  t.insert_node(node)
}
