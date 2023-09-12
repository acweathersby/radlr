//! Functions for translating parse graphs into Sherpa IR code.
use crate::{
  compile::build_graph::graph::{GraphBuilder, GraphIterator, GraphState, StateId, StateType},
  journal::Journal,
  types::*,
  utils::hash_group_btreemap,
  writer::code_writer::CodeWriter,
};
use sherpa_rust_runtime::types::bytecode::InputType;

use super::build_graph::{graph::GraphHost, items::get_follow_internal};

pub(crate) fn build_ir<'db>(
  j: &mut Journal,
  graph: &GraphHost<'db>,
  entry_name: IString,
) -> SherpaResult<Array<Box<ParseState>>> {
  debug_assert!(entry_name.as_u64() != 0);

  let mut output = OrderedMap::<StateId, Box<ParseState>>::new();
  let mut iter = GraphIterator::new(graph);

  while let Some((graph, state, successors)) = iter.next() {
    let goto_name = if let Some(goto) = state.get_goto_state() {
      let goto_pair = convert_nonterm_shift_state_to_ir(j, graph, &goto, successors)?;
      let out = Some(goto_pair.1.hash_name.clone());
      output.insert(goto_pair.0, goto_pair.1);
      out
    } else {
      None
    };

    for (id, ir_state) in convert_state_to_ir(j, graph, state, successors, entry_name, goto_name)? {
      output.entry(id).or_insert(ir_state);
    }
  }
  #[cfg(debug_assertions)]
  debug_assert!(!output.is_empty(), "This graph did not yield any states! \n{}", graph._debug_string_());

  j.report_mut().wrap_ok_or_return_errors(output.into_values().collect())
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SType {
  GotoSuccessors,
  SymbolSuccessors,
}

fn convert_nonterm_shift_state_to_ir<'db>(
  _j: &mut Journal,
  graph: &GraphHost<'db>,
  state: &GraphState,
  successors: &OrderedSet<&GraphState>,
) -> SherpaResult<(StateId, Box<ParseState>)> {
  let db = graph.get_db();
  let successors = successors.iter().filter(|s| {
    matches!(s.get_type(), StateType::NonTerminalComplete | StateType::NonTerminalShiftLoop | StateType::NonTerminalResolve)
  });

  let mut w = CodeWriter::new(vec![]);

  (&mut w + "match: " + InputType::NONTERMINAL_STR + " {").increase_indent();

  for (bc_id, (_nterm_name, s_name, transition_type)) in successors
    .into_iter()
    .map(|s| {
      if let SymbolId::DBNonTerminal { key: index } = s.get_symbol().sym() {
        let nterm: usize = index.into();
        let nterm_name = db.nonterm_guid_name(index);
        (nterm, (nterm_name, create_ir_state_name(graph, None, s), s.get_type()))
      } else {
        #[cfg(debug_assertions)]
        unreachable!("Invalid non-terminal type: {:?}  {}", s.get_symbol().sym(), s.get_symbol().sym().debug_string(db));
        #[cfg(not(debug_assertions))]
        unreachable!()
      }
    })
    .collect::<OrderedMap<_, _>>()
  {
    let bc_id = bc_id.to_string();
    match transition_type {
      StateType::NonTerminalResolve => {
        let _ = (&mut w) + "\n( " + bc_id + " ){ goto " + s_name + " }";
      }
      StateType::NonTerminalShiftLoop => {
        let _ = (&mut w) + "\n( " + bc_id + " ){ push %%%% then goto " + s_name + " }";
      }
      StateType::NonTerminalComplete => {
        let _ = (&mut w) + "\n( " + bc_id + " ){ pass }";
      }
      _ => unreachable!(),
    }
  }

  let _ = w.dedent() + "\n}";

  let mut goto = Box::new(create_ir_state(graph, w, state, None)?);

  goto.hash_name = create_ir_state_name(graph, None, state).intern(db.string_store());

  SherpaResult::Ok((state.get_id(), goto))
}

fn convert_state_to_ir<'db>(
  _j: &mut Journal,
  graph: &GraphHost<'db>,
  state: &GraphState,
  successors: &OrderedSet<&GraphState>,
  entry_name: IString,
  goto_state_id: Option<IString>,
) -> SherpaResult<Vec<(StateId, Box<ParseState>)>> {
  let state_id = state.get_id();
  let db: &ParserDatabase = graph.get_db();
  let s_store = db.string_store();

  let successor_groups = hash_group_btreemap(successors.clone(), |_, s| match s.get_type() {
    StateType::NonTerminalComplete | StateType::NonTerminalShiftLoop | StateType::NonTerminalResolve => SType::GotoSuccessors,
    _ => SType::SymbolSuccessors,
  });

  let base_state = if let Some(successors) = successor_groups.get(&SType::SymbolSuccessors) {
    let mut w = CodeWriter::new(vec![]);

    w.indent();

    add_tok_expr(graph, state, successors, &mut w);

    let mut classes = classify_successors(successors, db);

    let scanner_data = add_match_expr(&mut w, state, graph, &mut classes, goto_state_id);

    Some(Box::new(create_ir_state(graph, w, state, scanner_data)?))
  } else {
    None
  };

  let mut out = vec![];

  if matches!(
    state.get_type(),
    StateType::Complete | StateType::AssignAndFollow(..) | StateType::AssignToken(..) | StateType::Reduce(..)
  ) {
    let mut w = CodeWriter::new(vec![]);
    w.increase_indent();
    w.insert_newline()?;

    match state.get_type() {
      StateType::AssignAndFollow(tok_id) | StateType::AssignToken(tok_id) => {
        let _ = (&mut w) + "set-tok " + db.tok_val(tok_id).to_string();
      }
      StateType::Complete => w.write("pass")?,

      StateType::Reduce(rule_id, completes) => {
        debug_assert!(!state.kernel_items_ref().iter().any(|i| i.is_out_of_scope()));

        w.write(&create_rule_reduction(rule_id, db))?;

        if completes > 0 {
          let _ = (&mut w) + "pop " + completes.to_string();
        }
      }
      _ => unreachable!(),
    }

    if let Some(mut base_state) = base_state {
      if state_id.is_root() {
        base_state.hash_name = (entry_name.to_string(s_store) + "_then").intern(s_store);
      } else {
        base_state.hash_name = (base_state.hash_name.to_string(s_store) + "_then").intern(s_store);
      }

      w.w(" then goto ")?.w(&base_state.hash_name.to_string(s_store))?;

      out.push((state_id.to_post_reduce(), base_state));
    }

    let mut ir_state = create_ir_state(graph, w, state, None)?;

    if state_id.is_root() {
      ir_state.hash_name = entry_name;
      ir_state.root = true;
    }

    out.push((state_id, Box::new(ir_state)));
  } else if state.get_type() == StateType::DifferedReduce {
    let (shifts, reduces): (Vec<_>, Vec<_>) = successors.iter().cloned().partition(|s| s.get_type() == StateType::ShiftPrefix);
    let mut w = CodeWriter::new(vec![]);

    w.w(" push ")?.w(&create_ir_state_name(graph, Some(state), &reduces[0]))?;
    w.w(" then goto ")?.w(&create_ir_state_name(graph, Some(state), &shifts[0]))?;
    let base_state = Box::new(create_ir_state(graph, w, state, None)?);

    out.push((state_id.to_post_reduce(), base_state));
  } else if let Some(mut base_state) = base_state {
    if state_id.is_root() {
      base_state.hash_name = entry_name;
    }

    out.push((state_id, base_state));
  }
  #[cfg(debug_assertions)]
  debug_assert!(
    !out.is_empty()
      || matches!(
        state.get_type(),
        StateType::NonTerminalComplete
          | StateType::NonTermCompleteOOS
          | StateType::ScannerCompleteOOS
          | StateType::Complete
          | StateType::AssignToken(..)
      ),
    "Graph state failed to generate ir states:\n{} \nGraph\n{}",
    state._debug_string_(db),
    graph._debug_string_()
  );

  SherpaResult::Ok(out)
}

fn add_tok_expr(graph: &GraphHost, state: &GraphState, successors: &OrderedSet<&GraphState>, w: &mut CodeWriter<Vec<u8>>) {
  let db = graph.get_db();

  let mut set_token = successors.iter().filter_map(|s| match s.get_type() {
    StateType::AssignToken(tok) | StateType::AssignAndFollow(tok) => Some(tok),
    _ => None,
  });

  if let Some(tok_id) = set_token.next() {
    #[cfg(debug_assertions)]
    debug_assert!(
      set_token.all(|tok| tok == tok_id),
      "[INTERNAL ERROR] Expected a single token assignment, got [ {} ]\n in scanner state: \n{}. \n Successor:{}",
      set_token.map(|tok| db.token(tok).name.to_string(db.string_store())).collect::<Vec<_>>().join(" | "),
      state._debug_string_(db),
      successors.iter().map(|s| { s._debug_string_(db) }).collect::<Vec<_>>().join("\n")
    );

    (w + "set-tok " + db.tok_val(tok_id).to_string()).prime_join(" then ");
  }
}

fn classify_successors<'graph, 'db>(
  successors: &'graph OrderedSet<&'graph GraphState<'db>>,
  _db: &'db ParserDatabase,
) -> Queue<((u32, InputType), OrderedSet<&'graph GraphState<'db>>)> {
  Queue::from_iter(
    hash_group_btreemap(successors.clone(), |_, s| match s.get_symbol().sym() {
      SymbolId::EndOfFile { .. } => (0, InputType::EndOfFile),
      SymbolId::DBToken { .. } | SymbolId::DBNonTerminalToken { .. } => (4, InputType::Token),
      SymbolId::Char { .. } => (1, InputType::Byte),
      SymbolId::Codepoint { .. } => (2, InputType::Codepoint),
      SymbolId::Default => (5, InputType::Default),
      sym if sym.is_class() => (3, InputType::Class),
      _sym => {
        #[cfg(debug_assertions)]
        unreachable!("{_sym:?} {}", s._debug_string_(_db));
        #[cfg(not(debug_assertions))]
        unreachable!()
      }
    })
    .into_iter(),
  )
}

fn add_match_expr<'db>(
  mut w: &mut CodeWriter<Vec<u8>>,
  state: &GraphState,
  graph: &GraphHost<'db>,
  branches: &mut Queue<((u32, InputType), OrderedSet<&GraphState>)>,
  goto_state_id: Option<IString>,
) -> Option<(IString, OrderedSet<PrecedentDBTerm>)> {
  let db = graph.get_db();

  if let Some(((_, input_type), successors)) = branches.pop_front() {
    if matches!(input_type, InputType::Default) {
      let successor = successors.into_iter().next().unwrap();

      let string = build_body(state, successor, graph, goto_state_id).join(" then ");

      if !string.is_empty() {
        let _ = w + string;
      }

      None
    } else {
      let (symbols, skipped) = if input_type == InputType::Token {
        let mut syms = successors.iter().map(|s| s.get_symbol().sym().tok_db_key().unwrap()).collect::<OrderedSet<_>>();

        // If the kernel includes any completed items, include tokens that follow
        // those items.

        for item in state.kernel_items_ref().iter().filter(|i| i.is_complete()) {
          let mode = graph.graph_type;
          let (follow, _) = get_follow_internal(graph, *item).expect("Should be able to build follow sets");

          if graph._goal_nonterm_index_is_(0) && state.id.0 == 351 {
            println!("{}", state._debug_string_(db));
            follow._debug_print_("FOLLOW");
          }

          let iter = follow
            .iter()
            .closure::<Items>(StateId::root())
            .into_iter()
            .filter_map(|i| i.is_term(mode).then_some(i.term_index_at_sym(mode)).flatten());
          syms.extend(iter);
        }

        let skipped = if state.get_type() == StateType::Peek {
          state.get_resolve_items().flatten().filter_map(|i| i.get_skipped()).flatten().collect::<Vec<_>>()
        } else {
          state.kernel_items_ref().iter().filter_map(|i| i.get_skipped()).flatten().collect::<Vec<_>>()
        }
        .into_iter()
        .filter_map(|s| {
          let id = s.tok_db_key().unwrap();
          (!syms.contains(&id)).then_some(id)
        })
        .collect::<OrderedSet<_>>();

        // Build scanner collection
        let mut symbols = OrderedSet::default();

        for state in &successors {
          symbols.insert(PrecedentDBTerm::from(state.get_symbol(), db));
        }

        for sym in &skipped {
          symbols.insert((*sym, 0).into());
        }

        let skipped = if successors.iter().all(|s| matches!(s.get_type(), StateType::Reduce(..))) { None } else { Some(skipped) };

        (Some((ParseState::get_interned_scanner_name(&symbols, graph.get_db().string_store()), symbols)), skipped)
      } else {
        (None, None)
      };

      w = w + "\nmatch: " + input_type.as_str();

      if let Some((name, _)) = &symbols {
        w = w + ":" + name.to_str(db.string_store()).as_str();
      }

      w = (w + " {").indent();

      // Sort successors
      let peeking = successors.iter().any(|s| matches!(s.get_type(), StateType::PeekEndComplete(_) | StateType::Peek));

      for (state_val, s) in successors.iter().map(|s| (s.get_symbol().sym().to_state_val(db), s)).collect::<OrderedMap<_, _>>() {
        w = w + "\n\n( " + state_val.to_string() + " ){ ";
        w = w + build_body(state, s, graph, goto_state_id).join(" then ") + " }";
      }

      // Add skips
      if let Some(skipped) = skipped {
        if !skipped.is_empty() {
          let vals = skipped.iter().map(|v| v.to_val(db).to_string()).collect::<Array<_>>().join(" | ");
          if vals.len() > 0 {
            w = w + "\n( " + vals + " ){ " + peeking.then_some("peek-skip").unwrap_or("skip") + " }";
          }
        }
      }

      if !branches.is_empty() {
        w = (w + "\n\ndefault { ").indent();
        add_match_expr(w, state, graph, branches, goto_state_id);
        w = w + " }";
        w = w.dedent();
      }

      let _ = w.dedent() + "\n}";

      symbols
    }
  } else {
    None
  }
}

fn build_body<'db>(
  state: &GraphState,
  successor: &GraphState,
  graph: &GraphHost<'db>,
  goto_state_id: Option<IString>,
) -> Vec<String> {
  let is_scanner = graph.is_scanner();
  let mut body_string: Vec<String> = Array::new();
  let s_type = successor.get_type();
  let db = graph.get_db();

  if match s_type {
    StateType::Shift | StateType::KernelShift => {
      let scan_expr = successor.get_symbol().sym().is_linefeed().then_some("scan then set-line").unwrap_or("scan");
      body_string.push(is_scanner.then_some(scan_expr).unwrap_or("shift").into());
      true
    }
    StateType::PeekEndComplete(_) => {
      debug_assert!(!is_scanner, "Peek states should not be present in graph");
      body_string.push("reset".into());
      true
    }
    StateType::Peek => {
      debug_assert!(!is_scanner, "Peek states should not be present in graph");
      body_string.push("peek".into());
      true
    }
    StateType::NonTermCompleteOOS => {
      debug_assert!(!is_scanner, "NonTermCompleteOOS states should only exist in normal parse graphs");
      body_string.push("pop".into());
      false
    }
    StateType::ScannerCompleteOOS => {
      debug_assert!(is_scanner, "ScannerCompleteOOS states should only exist in scanner parse graphs");
      body_string.push("pass".into());
      false
    }
    StateType::Reduce(rule_id, completes) => {
      debug_assert!(!successor.kernel_items_ref().iter().any(|i| i.is_out_of_scope()));

      body_string.push(create_rule_reduction(rule_id, db));

      if completes > 0 {
        body_string.push("pop ".to_string() + &completes.to_string());
      }

      false
    }
    StateType::Follow => true,
    StateType::AssignToken(..) | StateType::Complete => {
      body_string.push("pass".into());
      false
    }
    _ => true,
  } {
    // Add goto expressions

    match (&goto_state_id, s_type) {
      // Kernel calls can bypass gotos.
      (_, StateType::KernelCall(..)) | (_, StateType::KernelShift) => {}
      (Some(gt), _) => body_string.push("push ".to_string() + &gt.to_string(db.string_store())),
      _ => {}
    }

    match s_type {
      //Ensure non-terminal calls are immediately called before any other
      // gotos.
      StateType::KernelCall(nterm) | StateType::InternalCall(nterm) => {
        body_string.push("push ".to_string() + &create_ir_state_name(graph, Some(state), successor));
        body_string.push("goto ".to_string() + &db.nonterm_guid_name(nterm).to_string(db.string_store()));
      }
      _ => {
        body_string.push("goto ".to_string() + &create_ir_state_name(graph, Some(state), successor));
      }
    }
  }

  body_string
}

fn create_rule_reduction(rule_id: DBRuleKey, db: &ParserDatabase) -> String {
  let rule = db.rule(rule_id);
  let nterm = db.rule_nonterm(rule_id);
  let nterm: usize = nterm.into();
  let rule_id: usize = rule_id.into();
  let mut w = CodeWriter::new(vec![]);

  let _ = &mut w + "reduce " + rule.symbols.len().to_string();
  let _ = &mut w + " symbols to " + nterm.to_string();
  let _ = &mut w + " with rule " + rule_id.to_string();

  w.to_string()
}

pub(super) fn create_ir_state_name(graph: &GraphHost, origin_state: Option<&GraphState>, target_state: &GraphState) -> String {
  if origin_state.is_some_and(|s| s.get_id() == target_state.get_id()) {
    "%%%%".to_string()
  } else if false {
    graph.get_state_name(target_state.get_id())
  } else if target_state.id.is_goto() {
    "g_".to_string() + &target_state.get_hash(graph.db).to_string()
  } else {
    graph.is_scanner().then_some("s").unwrap_or("p").to_string() + "_" + &target_state.get_hash(graph.db).to_string()
  }
}

pub(super) fn create_ir_state<'db>(
  graph: &GraphHost<'db>,
  w: CodeWriter<Vec<u8>>,
  state: &GraphState,
  scanner: Option<(IString, OrderedSet<PrecedentDBTerm>)>,
) -> SherpaResult<ParseState> {
  let ir_state = ParseState {
    code: w.to_string(),
    hash_name: create_ir_state_name(graph, None, state).intern(graph.get_db().string_store()),
    scanner,
    ..Default::default()
  };

  SherpaResult::Ok(ir_state)
}
