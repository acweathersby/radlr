use super::{build_graph::build, build_ir::build_ir};
use crate::{
  journal::{Journal, ReportType},
  types::*,
  writer::code_writer::CodeWriter,
};
type States = OrderedMap<IString, Box<ParseState>>;
type Scanners = OrderedSet<(IString, OrderedSet<PrecedentDBTerm>)>;
use rayon::prelude::*;

pub fn compile_parse_states(mut j: Journal, db: &ParserDatabase, config: ParserConfig) -> SherpaResult<ParseStatesMap> {
  j.set_active_report("State Compile", ReportType::NonTerminalCompile(Default::default()));

  #[cfg(all(debug_assertions, not(feature = "wasm-target")))]
  crate::test::utils::write_debug_file(db, "parse_graph.tmp", "", false)?;

  #[cfg(all(debug_assertions, not(feature = "wasm-target")))]
  crate::test::utils::write_debug_file(db, "scanner_graph.tmp", "", false)?;

  let results = db
    .nonterms()
    .iter()
    .cloned()
    .enumerate()
    .collect::<Vec<_>>()
    .chunks(10)
    .map(|chunks| (j.transfer(), chunks))
    .par_bridge()
    .into_par_iter()
    .map(|(mut local_j, chunks)| {
      let mut states = States::new();
      let mut scanners = Scanners::new();

      for (nterm, nterm_sym) in chunks {
        match create_parse_states_from_prod(&mut local_j, db, *nterm, *nterm_sym, &mut states, &mut scanners, config) {
          SherpaResult::Ok(output) => output,
          SherpaResult::Err(_err) => {}
        }
      }
      (states, scanners)
    })
    .collect::<Vec<_>>();

  let (mut states, scanners) =
    results.into_iter().fold((States::new(), Scanners::new()), |(mut st_to, mut sc_to), (mut st_from, mut sc_from)| {
      st_to.append(&mut st_from);
      sc_to.append(&mut sc_from);
      (st_to, sc_to)
    });

  // Build entry states
  for entry in db.entry_points() {
    let ir = build_entry_ir(entry, db)?;

    for state in ir {
      states.insert(state.hash_name, state);
    }
  }

  // Build Scanners
  for (scanner, group) in scanners {
    let start_items = group
      .iter()
      .flat_map(|s| Items::start_items(db.token(s.tok()).nonterm_id, db).to_origin(Origin::TerminalGoal(s.tok(), s.precedence())))
      .collect::<Array<_>>();

    let graph = build(&mut j, scanner, GraphMode::Scanner, start_items, db, config)?;

    let ir = build_ir(&mut j, &graph, scanner)?;
    // println!("{}", graph.debug_string());
    for state in ir {
      //  println!("{} {}", state.name.to_str(db.string_store()).as_str(),
      // state.code);
      states.insert(state.hash_name, state);
    }
  }

  for (_, state) in &mut states {
    // Warn of failed parses
    match state.build_ast(db) {
      SherpaResult::Err(err) => {
        todo!("Add State compile error to Journal {err}");
      }
      _ => {}
    }
  }

  SherpaResult::Ok(states)
}

fn create_parse_states_from_prod<'db>(
  j: &mut Journal,
  db: &'db ParserDatabase,
  nterm: usize,
  nterm_sym: SymbolId,
  states: &mut States,
  scanners: &mut Scanners,
  config: ParserConfig,
) -> SherpaResult<()> {
  j.set_active_report("Non-terminal Compile", ReportType::NonTerminalCompile(nterm_sym.to_nterm()));

  if let Some(custom_state) = db.custom_state(nterm.into()) {
    let name = db.nonterm_guid_name(nterm.into());
    let state = ParseState {
      hash_name:     name,
      name:          name,
      comment:       "Custom State".into(),
      code:          custom_state.tok.to_string(),
      ast:           Some(Box::new(custom_state.clone())),
      compile_error: None,
      scanner:       None,
      root:          true,
      precedence:    0,
    };
    states.insert(name, Box::new(state));
  } else {
    let start_items = Items::start_items((nterm as u32).into(), db).to_origin(Origin::NonTermGoal(nterm.into()));

    match nterm_sym {
      SymbolId::NonTerminal { .. } => {
        let graph = build(j, db.nonterm_guid_name(nterm.into()), GraphMode::Parser, start_items, db, config)?;

        let ir = build_ir(j, &graph, db.nonterm_guid_name(nterm.into()))?;

        for mut state in ir {
          if let Some(scanner_data) = state.get_scanner() {
            scanners.insert(scanner_data.clone());
          }
          states.insert(state.hash_name, state);
        }
      }
      SymbolId::NonTerminalToken { .. } => {
        let graph = build(j, db.nonterm_guid_name(nterm.into()), GraphMode::Scanner, start_items, db, config)?;

        let ir = build_ir(j, &graph, db.nonterm_guid_name(nterm.into()))?;

        for state in ir {
          states.insert(state.hash_name, state);
        }
      }
      SymbolId::NonTerminalState { .. } => {
        // todo!(load state into the states collection)
      }
      _ => unreachable!(),
    }
  }

  SherpaResult::Ok(())
}

fn build_entry_ir<'db>(
  EntryPoint {
    nonterm_name: nterm_name,
    nonterm_entry_name: nterm_entry_name,
    nonterm_exit_name: nterm_exit_name,
    ..
  }: &EntryPoint,
  db: &'db ParserDatabase,
) -> SherpaResult<Array<Box<ParseState>>> {
  let mut w = CodeWriter::new(Vec::<u8>::with_capacity(512));

  let _ = (&mut w) + "push " + nterm_exit_name.to_string(db.string_store());
  let _ = (&mut w) + " then goto " + nterm_name.to_string(db.string_store());

  let entry_state = ParseState {
    code: w.to_string(),
    hash_name: *nterm_entry_name,
    ..Default::default()
  };

  let mut w = CodeWriter::new(Vec::<u8>::with_capacity(512));

  let _ = (&mut w) + "accept";

  let exit_state = ParseState { code: w.to_string(), hash_name: *nterm_exit_name, ..Default::default() };

  SherpaResult::Ok(Array::from_iter([Box::new(entry_state), Box::new(exit_state)]))
}
