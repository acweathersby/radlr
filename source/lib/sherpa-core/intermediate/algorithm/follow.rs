use crate::{
  intermediate::utils::hash_group_btreemap,
  types::{TransitionGraph as TPack, *},
};
use std::{
  collections::{BTreeSet, VecDeque},
  vec,
};

/// Retrieve items following the reduction of the `root_completed_item`. This is similar to an [Earley
/// parser](https://en.wikipedia.org/wiki/Earley_parser)'s complete action, where completed productions are
/// matched to items in previous states, leading to new items to process.
pub(super) fn get_follow_items(
  t: &mut TPack,
  root_completed_item: &Item,
  prev_state_ref: MaybeNodeId,
) -> FollowItemGroups {
  let mut seen = BTreeSet::<(ItemState, LinkedItem)>::new();
  let mut out = BTreeSet::<LinkedItem>::new();
  let mut fin_items = BTreeSet::<LinkedItem>::new();
  let mut intermediate = BTreeSet::<LinkedItem>::new();
  let grammar = t.g.clone();
  let g = &grammar;
  let empty = vec![];

  let __print_debug__ = false;
  #[cfg(follow_tracking)]
  let __print_debug__ = root_completed_item.get_state().get_lane() == 0;

  if __print_debug__ {
    eprintln!("\n\n---- Follow start on {} ----", root_completed_item.debug_string(g));
  }

  static empty_vec: Vec<Item> = Vec::new();
  // Starting at the top we grab the closure to the nearest
  // non-term link.

  // Stores the end item [1] and its immediate closure item [0]
  let mut completed_items = VecDeque::from_iter(vec![(
    root_completed_item.get_state(),
    (LinkedItem { item: *root_completed_item, closure_node: prev_state_ref }),
  )]);
  while let Some((state, linked)) = completed_items.pop_front() {
    if __print_debug__ {
      eprintln!(
        "\nLooking for matches for  {} in {:?} with state {}",
        linked.item.debug_string(g),
        linked.closure_node,
        state.debug_string(g)
      );
    }
    let completed_item = linked.item.clone();
    let current_node = linked.closure_node;

    if seen.insert((state, linked.clone())) {
      let (iter, prev_node) = match linked {
        LinkedItem { .. } if state.is_out_of_scope() => {
          let global_lr = g.lr_items.get(&linked.item.get_prod_id(g)).unwrap_or(&empty);
          (global_lr.clone().to_state(state).into_iter(), Some(NodeId::new(0)))
        }
        LinkedItem { closure_node: Some(curr_node), .. } => {
          let node = t.get_node(curr_node);
          if __print_debug__ {
            node
              .goto_items
              .closure_with_state(&t.g)
              .to_vec()
              .__print_items__(&t.g, &format!("\n Closure {:?}", curr_node));
          }
          (node.goto_items.closure_with_state(&t.g).to_vec().into_iter(), node.closure_parent)
        }
        LinkedItem { item, closure_node: None, .. } => (
          if item.is_out_of_scope() {
            t.out_of_scope_closure.as_ref().unwrap_or(&empty_vec).clone().into_iter()
          } else {
            vec![].into_iter()
          },
          None,
        ),
      };

      let local_closure_lookup =
        hash_group_btreemap(iter.collect::<Items>(), |_, i| i.get_production_id_at_sym(&t.g));

      let prod = linked.item.get_prod_id(g);

      let null_items =
        local_closure_lookup.get(&ProductionId(0)).unwrap_or(&empty).iter().filter(|i| {
          if i.is_null() {
            if state.is_null() {
              i.get_state().get_lanes().0 == state.get_lanes().1
            } else {
              i.get_state().same_curr_lane(&state)
            }
          } else {
            false
          }
        });

      let goto_items = local_closure_lookup.get(&prod).unwrap_or(&empty).iter().filter(|i| {
        if __print_debug__ {
          println!("{} {} s:{}", i.debug_string(g), state, i.get_state().in_either_lane(&state))
        }
        i.get_state().in_either_lane(&state)
      });

      let mut closure: Items = null_items.cloned().chain(goto_items.cloned()).collect();

      // Grab all productions from the closure that match the end item's
      // production.
      match { (completed_item, closure.is_empty(), prev_node, current_node) } {
        (completed_item, true, Some(prev_node), _) => {
          if __print_debug__ {
            eprintln!("no closure for Node [{:?}] - Selecting previous node", current_node);
          }
          completed_items.push_back((state, LinkedItem {
            item:         completed_item,
            closure_node: Some(prev_node),
          }));
        }
        (completed_item, true, None, Some(root_node)) => {
          debug_assert!(
            root_node.usize() == 0,
            "The root node should be the only one that ends up in this branch"
          );
          if t.item_is_goal(completed_item) {
            fin_items.insert(LinkedItem {
              item:         completed_item.to_origin_only_state(),
              closure_node: None,
            });
          }
          if __print_debug__ {
            eprintln!("no closure for Node [{:?}] - Should be at root node.", root_node);
          }
          // This item should match one of the root items when set to completed
          if completed_item == *root_completed_item {
            fin_items.insert(LinkedItem { item: completed_item, closure_node: None });
          }
        }
        (completed_item, false, prev_node, Some(current_node)) => {
          let proxy_state = completed_item.get_state();
          if __print_debug__ {
            closure.__print_items__(g, &format!("Node [{:?}] closure:", current_node));
          }
          let null_items: Items = closure.drain_filter(|i| i.is_null()).collect();
          if !null_items.is_empty() {
            for null_item in null_items {
              completed_items.push_back((null_item.get_state().to_prev_lane(), LinkedItem {
                item:         completed_item,
                closure_node: Some(current_node),
              }));
            }
          } else {
            let mut uncompleted_items = closure.try_increment();
            let completed = uncompleted_items
              .drain_filter(|i| i.completed())
              .map(|i| (proxy_state, i))
              .collect::<Vec<_>>();
            let mut uncompleted_items = uncompleted_items.to_state(proxy_state);
            let mut seen = ItemSet::new();
            let mut completed_queue = VecDeque::from_iter(completed);

            while let Some((proxy_state, item)) = completed_queue.pop_front() {
              if seen.insert(item.to_empty_state().to_start()) {
                if __print_debug__ {
                  eprintln!("---- {}", item.debug_string(&t.g));
                }
                // Preserve the item's original state
                let original_state = item.get_state();
                let fork_state = if state.is_out_of_scope() {
                  // Keep out of scope states in the same lane.
                  state
                } else {
                  let lane = t.increment_lane(1);
                  proxy_state.to_lane_fork(lane)
                };

                // Put the completed item into a new lane.
                let forked_item = item.to_state(fork_state);

                // Grab the production the item reduces to.
                let prod = forked_item.get_prod_id(&t.g);

                // Based on the item's original state, find all other
                // items that are "goto" on the production.
                let goto_items: Items = local_closure_lookup
                  .get(&prod)
                  .unwrap_or(&empty)
                  .iter()
                  .filter(|i| {
                    !seen.contains(&i.to_empty_state().to_start())
                      && original_state.in_either_lane(&i.get_state())
                      && i.get_production_id_at_sym(&t.g) == prod
                  })
                  .cloned()
                  .collect();

                if __print_debug__ {
                  local_closure_lookup
                    .get(&prod)
                    .unwrap_or(&empty)
                    .__print_items__(&t.g, "debug gotos");
                  goto_items.__print_items__(&t.g, "results");
                }

                // Place a null slide into the active state's goto closure.
                t.get_node_mut(current_node).goto_items.push(forked_item.to_null());

                if goto_items.len() > 0 {
                  let incremented = goto_items.into_iter().map(|i| i.try_increment());

                  completed_queue.append(
                    &mut incremented
                      .clone()
                      .filter(|i| i.completed())
                      .map(|i| (fork_state, i))
                      .collect(),
                  );

                  uncompleted_items.append(
                    &mut incremented
                      .filter(|i| !i.completed())
                      .map(|i| i.to_state(fork_state))
                      .collect(),
                  );

                  // Preserve
                  intermediate.insert(LinkedItem {
                    item:         forked_item,
                    closure_node: Some(current_node),
                  });
                } else {
                  // Let the item "fall into" the previous state's closure
                  completed_items.push_back((original_state, LinkedItem {
                    item:         forked_item,
                    closure_node: prev_node,
                  }));
                }
              }
            }

            for item in uncompleted_items {
              out.insert(LinkedItem {
                item:         item.to_local_state(),
                closure_node: Some(current_node),
              });
            }
          }
        }
        (completed_item, ..) => {
          // Check to see if we have an accept item
          if __print_debug__ {
            eprintln!("Evaluating potential leaf node ------------------");
            eprintln!("---- {}", completed_item.to_state(state).debug_string(&t.g));
            t.accept_items().__print_items__(g, "Accepting Items");
          }

          // Remap item to proxy state.
          let candidate_state = completed_item.to_state(state).to_origin_only_state();

          if t.accept_items().contains(&candidate_state) {
            fin_items.insert(LinkedItem { item: completed_item, closure_node: None });
          } else {
            eprintln!("All possible conditions should be covered by the above: ");
            eprintln!("completed_items: {} {}", completed_item.debug_string(g), state);
            t.accept_items().__print_items__(g, "Accept Items");
          }
        }
      }
    }
  }
  if __print_debug__ {
    Items::from_linked(fin_items.clone()).__print_items__(g, "Completed Final Items");
    Items::from_linked(intermediate.clone()).__print_items__(g, "Intermediate Items");
    Items::from_linked(out.clone()).__print_items__(g, "Uncompleted Items");
    eprintln!("---- Follow end on {} ----\n\n", root_completed_item.debug_string(g));
  }

  FollowItemGroups {
    final_completed_items: fin_items.into_iter().collect(),
    intermediate_completed_items: intermediate.into_iter().collect(),
    uncompleted_items: out.into_iter().collect(),
  }
}
