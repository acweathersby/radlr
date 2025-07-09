#![allow(unused_mut, unused)]
use super::{
  build_grammar::{create_grammar_data, extract_nonterminals, parse_grammar, GrammarData},
  build_grammar_2,
  utils::resolve_grammar_path,
};
use crate::{
  grammar::build_grammar::{convert_grammar_data_to_header, process_nonterminals, process_parse_state},
  types::*,
};
use std::{
  collections::{HashSet, VecDeque},
  path::PathBuf,
  sync::{mpsc::sync_channel, Arc, Mutex, RwLock},
  time::Duration,
};

/// Entrypoint for compiling a single of grammar from a source file.
pub fn compile_grammar_from_str(
  source: &str,
  source_path: PathBuf,
  s_store: IStringStore,
) -> RadlrResult<(Arc<GrammarSoup>, Vec<GrammarIdentities>)> {
  let mut soup = GrammarSoup::from_string_store(s_store);

  let root_id = GrammarIdentities::from_path(&source_path, &soup.string_store);

  let g_data = load_from_str(source, source_path, &soup)?;

  let imports = g_data.imports.values().cloned().collect();

  let grammar_id = compile_grammar_data(g_data, &soup)?;

  Ok((soup, imports))
}

/// Imports a single grammar from a source location and merges it into the soup.
/// Returns a list of grammars that are imported by the source grammar
pub fn load_grammar(
  import_id: GrammarIdentities,
  s_store: IStringStore,
) -> RadlrResult<(Arc<GrammarSoup>, Vec<GrammarIdentities>)> {
  let mut g_c: Arc<GrammarSoup> = GrammarSoup::from_string_store(s_store);
  let GrammarIdentities { guid, guid_name, path, .. } = import_id;

  let grammar_source_path = PathBuf::from(path.to_str(&g_c.string_store).as_str());

  match load_from_path(grammar_source_path, &g_c) {
    RadlrResult::Ok(g_data) => {
      let imports = g_data.imports.values().cloned().collect();

      match compile_grammar_data(g_data, &g_c) {
        Ok(result) => Some(result),
        Err(err) => {
          eprintln!("{err}");
          None
        }
      };

      Ok((g_c, imports))
    }
    RadlrResult::Err(err) => {
      eprintln!("{err}");
      Err(err)
    }
  }
}

fn compile_grammar_data(g_data: GrammarData, g_s: &GrammarSoup) -> RadlrResult<GrammarIdentities> {
  let id = g_data.id;

  let (mut nterms, templates, mut parse_states) = extract_nonterminals(&g_data, &g_s.string_store)?;


  let mut resolved_templates = Map::<NonTermId, Option<Box<NonTerminal>>>::new();
  for nterm in nterms {
    g_s.nonterminals.write().unwrap().push(process_nonterminals(
      nterm,
      &g_data,
      &g_s.string_store,
      &templates,
      &mut resolved_templates,
    )?);
  }

  for (_, resolved_template) in resolved_templates {
    g_s.nonterminals.write().unwrap().push(resolved_template.expect("Should be resolved"));
  }

  for state in parse_states {
    g_s.custom_states.write().unwrap().insert(state.id, process_parse_state(state, &g_data, &g_s.string_store)?);
  }

  {
    g_s.grammar_headers.write().unwrap().insert(g_data.id.guid, convert_grammar_data_to_header(g_data.id, g_data));
  }

  RadlrResult::Ok(id)
}

#[track_caller]
fn load_from_path(source_path: PathBuf, soup: &GrammarSoup) -> RadlrResult<GrammarData> {
  match std::fs::read_to_string(&source_path) {
    Ok(source) => {
      let source = std::fs::read_to_string(&source_path)?;
      load_from_str(source.as_str(), source_path, soup)
    }
    Err(_) => RadlrResult::Err(if source_path.as_os_str().is_empty() {
      ("Source path for grammar is empty").into()
    } else {
      ("Unable to retrieve a grammar source from this path: ".to_string() + source_path.as_os_str().to_str().unwrap()).into()
    }),
  }
}

fn load_from_str(source: &str, source_path: PathBuf, soup: &GrammarSoup) -> RadlrResult<GrammarData> {
  match build_grammar_2::parse_grammar(source) {
    Ok(g) => { /* println!("Parser2 built: {g:#?}") */ }
    Err(g) => println!("Parser2 failed: {g:#?}"),
  }

  let root_grammar = match parse_grammar(&source) {
    RadlrResult::Ok(root_grammar) => root_grammar,
    RadlrResult::Err(err) => {
      eprintln!("{err}");
      return Err(err);
    }
    _ => return RadlrResult::Err(RadlrError::from("Failed Parse")),
  };

  let g_data = create_grammar_data(root_grammar, &source_path, &soup.string_store)?;

  RadlrResult::Ok(g_data)
}
