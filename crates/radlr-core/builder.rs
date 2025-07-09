use std::{
  fs::OpenOptions,
  io::Write,
  path::{Path, PathBuf},
};

use crate::{
  compile::states::build_graph::graph::Graphs,
  grammar::{build_compile_db, compile_grammar_from_str, load_grammar, remove_grammar_mut, utils::resolve_grammar_path},
  o_to_r,
  proxy::{Array, DeduplicateIterator, Queue, Set},
  types::{worker_pool::WorkerPool, *},
  GrammarDatabase,
  GrammarIdentities,
  GrammarSoup,
  IString,
  ParseState,
  ParseStatesVec,
  ParserClassification,
  ParserMetrics,
  RadlrError,
  RadlrResult,
};

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

/// Pre-compiled Grammar compilation context
pub struct RadlrGrammar {
  soup: std::sync::Arc<GrammarSoup>,
}

fn blend_soups(soup: std::sync::Arc<GrammarSoup>, other: GrammarSoup) -> RadlrResult<()> {
  let GrammarSoup { grammar_headers, nonterminals, custom_states, .. } = soup.as_ref();

  let mut grammar_headers = grammar_headers.write()?;
  let mut nonterminals = nonterminals.write()?;
  let mut custom_states = custom_states.write()?;

  let other_grammar_headers = std::sync::Arc::into_inner(other.grammar_headers).unwrap().into_inner()?;
  let other_nonterminals = std::sync::Arc::into_inner(other.nonterminals).unwrap().into_inner()?;
  let other_custom_states = std::sync::Arc::into_inner(other.custom_states).unwrap().into_inner()?;

  for (id, header) in other_grammar_headers.into_iter() {
    if grammar_headers.insert(id, header).is_some() {
      let nonterminals_temp = nonterminals.drain(..).collect::<Vec<_>>();
      nonterminals.extend(nonterminals_temp.into_iter().filter(|p| p.g_id != id));

      let custom_states_temp = custom_states.drain().collect::<Vec<_>>();
      custom_states.extend(custom_states_temp.into_iter().filter(|(_, s)| s.g_id != id))
    }
  }

  nonterminals.extend(other_nonterminals.into_iter());
  custom_states.extend(other_custom_states.into_iter());

  Ok(())
}

impl RadlrGrammar {
  pub fn new() -> Self {
    Self { soup: GrammarSoup::new() }
  }

  pub fn path_to_id<T: Into<PathBuf>>(&self, path: T) -> GrammarIdentities {
    GrammarIdentities::from_path(&path.into(), &self.soup.string_store)
  }

  /// Takes a path and makes a best attempt to resolve it to a RadLR grammar
  /// file.
  pub fn resolve_to_grammar_file(path: &Path) -> RadlrResult<PathBuf> {
    let cwd = std::env::current_dir()?;
    let cwd = path.parent().unwrap_or(cwd.as_path());
    let resolved_path = resolve_grammar_path(&path, cwd, &["sg", "radlr"])?;
    let resolved_path = resolved_path.canonicalize()?;
    Ok(resolved_path)
  }

  /// Adds a grammar source to the soup
  pub fn add_source<T: Into<PathBuf>>(&mut self, path: T) -> RadlrResult<&mut Self> {
    let mut errors = Vec::new();
    let id = self.path_to_id(path);

    let RadlrGrammar { soup } = self;

    let mut queue = Queue::from_iter([id]);

    let mut known_imports = Set::from_iter(soup.grammar_headers.read().map_err(|e| RadlrError::from(e))?.iter().map(|i| *i.0));

    while let Some(id) = queue.pop_front() {
      if known_imports.insert(id.guid) {
        match load_grammar(id, soup.string_store.clone()) {
          Ok((new_soup, new_imports)) => {
            blend_soups(
              soup.clone(),
              std::sync::Arc::into_inner(new_soup).expect("There should be only one reference for this"),
            )?;
            queue.append(&mut new_imports.into_iter().collect());
          }
          Err(err) => errors.extend(err.flatten()),
        }
      }
    }

    if !errors.is_empty() {
      Err(RadlrError::Multi(errors))
    } else {
      Ok(self)
    }
  }

  /// Adds a grammar to the soup from a source string
  pub fn add_source_from_string_with_imports<T: Into<PathBuf>>(&mut self, source: &str, grammar_path: T) -> RadlrResult<()> {
    let mut errors = Vec::new();

    let path: PathBuf = grammar_path.into();

    let id = self.path_to_id(&path);

    let RadlrGrammar { soup } = self;

    let mut known_imports = Set::from_iter(soup.grammar_headers.read().map_err(|e| RadlrError::from(e))?.iter().map(|i| *i.0));

    if !known_imports.contains(&id.guid) {
      match compile_grammar_from_str(source, path.to_owned(), soup.string_store.clone()) {
        Ok((soup, ids)) => {
          known_imports.insert(id.guid);

          let mut queue = Queue::from_iter(ids);

          while let Some(id) = queue.pop_front() {
            if known_imports.insert(id.guid) {
              match load_grammar(id, soup.string_store.clone()) {
                Ok((new_soup, new_imports)) => {
                  blend_soups(
                    soup.clone(),
                    std::sync::Arc::into_inner(new_soup).expect("There should be only one reference for this"),
                  )?;
                  queue.append(&mut new_imports.into_iter().collect());
                }
                Err(err) => errors.extend(err.flatten()),
              }
            }
          }
        }
        Err(err) => errors.extend(err.flatten()),
      }
    }

    if !errors.is_empty() {
      Err(RadlrError::Multi(errors))
    } else {
      Ok(())
    }
  }

  pub fn add_source_from_string<T: Into<PathBuf>>(
    &mut self,
    source: &str,
    grammar_path: T,
    replace: bool,
  ) -> RadlrResult<&mut Self> {
    let mut errors = Vec::new();
    let path: PathBuf = grammar_path.into();
    let id = self.path_to_id(&path);

    let RadlrGrammar { soup } = self;

    let known_imports = Set::from_iter(soup.grammar_headers.read().map_err(|e| RadlrError::from(e))?.iter().map(|i| *i.0));

    if replace || !known_imports.contains(&id.guid) {
      match compile_grammar_from_str(source, path.to_owned(), soup.string_store.clone()) {
        Ok((new_soup, _)) => {
           let Some(reference) = std::sync::Arc::into_inner(new_soup) else {
            return Err(RadlrError::StaticText("Could not take over ownership of soup Arc"));
          };
          
          blend_soups(soup.clone(), reference)?;
        }
        Err(err) => errors.extend(err.flatten()),
      };
    }

    if !errors.is_empty() {
      Err(RadlrError::Multi(errors))
    } else {
      Ok(self)
    }
  }

  pub fn remove_grammar<T: Into<PathBuf>>(self, root_grammar: T) -> RadlrResult<RadlrGrammar> {
    let path: PathBuf = root_grammar.into();
    let id = self.path_to_id(&path);

    let RadlrGrammar { soup } = self;

    let mut soup = o_to_r(std::sync::Arc::into_inner(soup), "could not get exclusive access to soup")?;

    remove_grammar_mut(id.guid, &mut soup)?;

    Ok(Self { soup: std::sync::Arc::new(soup) })
  }

  pub fn build_db<T: Into<PathBuf>>(&self, root_grammar: T, config: ParserConfig) -> RadlrResult<RadlrGrammarDatabase> {
    let RadlrGrammar { soup } = self;
    let path: PathBuf = root_grammar.into();

    let id = GrammarIdentities::from_path(&path, &soup.string_store);

    match build_compile_db(id, soup, &config) {
      Err(errors) => Err(errors),
      Ok(db) => Ok(RadlrGrammarDatabase { db: std::sync::Arc::new(db) }),
    }
  }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

/// A wrapper around a ParserDatabase, providing methods to convert a database
/// into parsers
#[derive(Clone, Debug)]
pub struct RadlrGrammarDatabase {
  db: SharedGrammarDatabase,
}

impl RadlrGrammarDatabase {
  /// Returns a reference to the underlying [ParserDatabase].
  pub fn get_internal(&self) -> &GrammarDatabase {
    &self.db
  }

  /// Returns the internal ParserDatabase, consuming the wrapper in the process.
  pub fn into_internal(self) -> SharedGrammarDatabase {
    self.db
  }

  /// Constructs parser and scanner graphs for this variant of the grammar.
  pub fn build_states<Pool: WorkerPool>(&self, config: ParserConfig, pool: &Pool) -> RadlrResult<RadlrParseGraph> {
    let RadlrGrammarDatabase { db } = self;

    match crate::compile::states::build_states::compile_parser_states(db.clone(), config, pool) {
      Ok(graph) => Ok(RadlrParseGraph { graph, db: db.clone(), config }),
      Err(err) => {
        let mut errors = err.flatten();
        if errors.len() > 1 {
          Err(errors.into_iter().dedup::<Array<_>>().into_iter().into_multi())
        } else {
          Err(errors.pop().unwrap())
        }
      }
    }
  }

  pub fn print_terminals(&self) {
    let RadlrGrammarDatabase { db, .. } = self;

    for tok in db.tokens() {
      eprintln!("{: >5}  {: <10}", tok.tok_id.to_val(), tok.name.to_string(db.string_store()))
    }
  }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

/// Comprised of the parser and scanner graphs for the given grammar
pub struct RadlrParseGraph {
  pub(crate) db:     SharedGrammarDatabase,
  pub(crate) config: ParserConfig,
  pub(crate) graph:  std::sync::Arc<Graphs>,
}

impl RadlrParseGraph {
  pub fn build_ir_parser<Pool: WorkerPool>(
    &self,
    optimize: bool,
    optimize_for_debugging: bool,
    pool: &Pool,
  ) -> RadlrResult<RadlrIRParser> {
    match crate::compile::ir::build_ir_concurrent(pool, self.graph.clone(), self.config, &self.db) {
      Ok((classification, ir_states)) => {
        let Self { config, db, .. } = self;

        let (states, report): (Vec<_>, _) = if optimize {
          crate::compile::ir::optimize(db, config, ir_states, optimize_for_debugging, pool)?
        } else {
          crate::compile::ir::sweep(db, config, ir_states, optimize_for_debugging)?
        };

        Ok(RadlrIRParser {
          classification,
          config: *config,
          db: db.clone(),
          states,
          is_optimized: optimize,
          report,
        })
      }

      Err(err) => {
        let mut errors = err.flatten();
        if errors.len() > 1 {
          Err(errors.into_iter().dedup::<Array<_>>().into_iter().into_multi())
        } else {
          Err(errors.pop().unwrap())
        }
      }
    }
  }

  pub fn write_debug_file(&self, output_path: &Path) -> RadlrResult<()> {
    let file_path = output_path.join("parser_states.debug.txt");

    let mut file = OpenOptions::new().truncate(true).write(true).create(true).open(file_path)?;

    for parser_state in self.graph.create_ir_precursors().iter() {
      file.write_all(format!("{:?}", parser_state.node).as_bytes())?;
    }

    file.flush()?;

    Ok(())
  }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

pub struct RadlrIRParser {
  db:             SharedGrammarDatabase,
  config:         ParserConfig,
  states:         ParseStatesVec,
  classification: ParserClassification,
  is_optimized:   bool,
  pub report:     OptimizationReport,
}

impl ParserStore for RadlrIRParser {
  fn report(&self) -> OptimizationReport {
    self.report
  }

  fn get_config(&self) -> &ParserConfig {
    &self.config
  }

  fn get_db(&self) -> &GrammarDatabase {
    &self.db
  }

  fn get_states(&self) -> impl Iterator<Item = (IString, &ParseState)> {
    self.get_states()
  }

  fn get_classification(&self) -> ParserClassification {
    self.classification
  }

  fn get_is_optimized(&self) -> bool {
    self.is_optimized
  }
}

impl RadlrIRParser {
  pub fn get_db(&self) -> &GrammarDatabase {
    &self.db
  }

  /// Returns either optimized or unoptimized states depending on if an
  /// optimization pass has been performed.
  pub fn get_states(&self) -> impl Iterator<Item = (IString, &ParseState)> {
    self.states.iter().map(|(a, b)| (*a, b.as_ref()))
  }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

#[test]
pub fn empty_source_path() -> RadlrResult<()> {
  assert!(RadlrGrammar::new().add_source(&std::path::PathBuf::new()).is_err());
  Ok(())
}

pub struct TestPackage {
  pub states:         ParseStatesVec,
  pub db:             SharedGrammarDatabase,
  pub config:         ParserConfig,
  pub report:         OptimizationReport,
  pub classification: ParserClassification,
}

impl ParserStore for TestPackage {
  fn report(&self) -> OptimizationReport {
    self.report
  }

  fn get_config(&self) -> &ParserConfig {
    &self.config
  }

  fn get_db(&self) -> &GrammarDatabase {
    &self.db
  }

  fn get_states(&self) -> impl Iterator<Item = (IString, &ParseState)> {
    self.states.iter().map(|s| (s.0, s.1.as_ref()))
  }

  fn get_is_optimized(&self) -> bool {
    false
  }

  fn get_classification(&self) -> ParserClassification {
    self.classification
  }
}

impl From<RadlrIRParser> for TestPackage {
  fn from(value: RadlrIRParser) -> Self {
    TestPackage {
      config:         value.config,
      states:         value.states.into_iter().collect(),
      db:             value.db,
      report:         value.report,
      classification: value.classification,
    }
  }
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

pub struct DBPackage {
  pub db: SharedGrammarDatabase,
}

impl From<RadlrGrammarDatabase> for DBPackage {
  fn from(value: RadlrGrammarDatabase) -> Self {
    DBPackage { db: value.db.clone() }
  }
}

#[test]
pub fn source_path_to_default_file() -> RadlrResult<()> {
  let grammar_source_path =
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../grammar/json/json.sg").canonicalize().unwrap();
  assert!(RadlrGrammar::new().add_source(&grammar_source_path).is_ok());
  Ok(())
}

#[test]
pub fn build_db() -> RadlrResult<()> {
  let grammar_source_path =
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../grammar/json/json.sg").canonicalize().unwrap();
  assert!(RadlrGrammar::new().add_source(&grammar_source_path)?.build_db(grammar_source_path, Default::default()).is_ok());
  Ok(())
}

#[test]
pub fn build_states() -> RadlrResult<()> {
  let pool = crate::types::worker_pool::SingleThreadPool {};
  let grammar_source_path =
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../grammar/json/json.sg").canonicalize().unwrap();
  assert!(RadlrGrammar::new()
    .add_source(&grammar_source_path)?
    .build_db(grammar_source_path, Default::default())?
    .build_states(Default::default(), &pool)
    .is_ok());
  Ok(())
}

#[test]
pub fn build_with_optimized_states() -> RadlrResult<()> {
  let pool = crate::types::worker_pool::SingleThreadPool {};
  let grammar_source_path =
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../grammar/json/json.sg").canonicalize().unwrap();
  RadlrGrammar::new()
    .add_source(&grammar_source_path)?
    .build_db(grammar_source_path, Default::default())?
    .build_states(Default::default(), &pool)?
    .build_ir_parser(true, false, &pool)?;
  Ok(())
}

// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------
// ----------------------------------------------------------------------------------------

pub trait ParserStore {
  fn get_states(&self) -> impl Iterator<Item = (IString, &ParseState)>;
  fn get_db(&self) -> &GrammarDatabase;
  fn get_classification(&self) -> ParserClassification;
  fn get_is_optimized(&self) -> bool;
  fn get_config(&self) -> &ParserConfig;
  fn report(&self) -> OptimizationReport;

  fn get_metrics(&self) -> ParserMetrics {
    ParserMetrics {
      classification: self.get_classification(),
      num_of_states:  self.get_states().count(),
      optimized:      self.get_is_optimized(),
    }
  }

  /// Prints the ir code of the parser states to `stdout`
  #[inline(always)]
  fn _print_states_(&self) {
    #[cfg(debug_assertions)]
    {
      let states = self.get_states();
      for state in states {
        if let Ok(string) = state.1.print(self.get_db(), true) {
          eprintln!("{}", string);
        }
      }
    }
  }
}
