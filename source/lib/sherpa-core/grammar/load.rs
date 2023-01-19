//! Functions for Grammar file path resolution and loading,
//! and initial parse functions

use super::{
  data::ast::{ASTNode, Grammar, Import},
  multitask::WorkVerifier,
};
use crate::{journal::Journal, types::*};
use std::{
  collections::{HashSet, VecDeque},
  fs::read,
  num::NonZeroUsize,
  path::{Path, PathBuf},
  sync::Mutex,
  thread::{self},
};

// TODO: Replace with the new grammar parser
use super::parse::compile_grammar_ast;

const allowed_extensions: [&str; 3] = ["hc", "hcg", "grammar"];

pub(crate) fn get_usable_thread_count(requested_count: usize) -> usize {
  NonZeroUsize::min(
    NonZeroUsize::new(usize::max(1, requested_count)).unwrap(),
    std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap()),
  )
  .get()
}

/// Loads all grammars that are indirectly or directly referenced from a single filepath.
/// Returns a vector grammars in no particular order except the first grammar belongs to
/// the file path
pub(crate) fn load_all(
  j: &mut Journal,
  absolute_path: &PathBuf,
  number_of_threads: usize,
) -> (Vec<(PathBuf, ImportedGrammarReferences, Box<Grammar>)>, Vec<SherpaError>) {
  let pending_grammar_paths =
    Mutex::new(VecDeque::<PathBuf>::from_iter(vec![absolute_path.clone()]));
  let claimed_grammar_paths = Mutex::new(HashSet::<PathBuf>::new());
  let work_verifier = Mutex::new(WorkVerifier::new(1));

  let results = thread::scope(|s| {
    [0..get_usable_thread_count(number_of_threads)]
      .into_iter()
      .map(|_| {
        let mut j = j.transfer();
        let claimed_grammar_paths = &claimed_grammar_paths;
        let work_verifier = &work_verifier;
        let pending_grammar_paths = &pending_grammar_paths;
        s.spawn(move || {
          let mut grammars = vec![];
          let mut errors = vec![];

          loop {
            match {
              {
                let val = pending_grammar_paths
                  .lock()
                  .unwrap()
                  .pop_front()
                  .and_then(|path| {
                    claimed_grammar_paths.lock().as_mut().map_or(None, |d| {
                      let mut work_verifier = work_verifier.lock().unwrap();
                      if d.insert(path.clone()) {
                        work_verifier.start_one_unit_of_work();
                        Some(path)
                      } else {
                        work_verifier.skip_one_unit_of_work();
                        None
                      }
                    })
                  })
                  .clone();
                val
              }
            } {
              Some(path) => match load_grammar(&mut j, &path) {
                SherpaResult::Ok((grammar, imports)) => {
                  let mut imports_refs: ImportedGrammarReferences = Default::default();

                  for box Import { uri, reference, tok } in imports {
                    let base_path = PathBuf::from(uri);
                    match resolve_grammar_path(
                      &base_path,
                      &path.parent().unwrap_or(Path::new("")).to_path_buf(),
                      &allowed_extensions,
                    ) {
                      SherpaResult::Ok(path) => {
                        imports_refs.insert(
                          reference.to_string(),
                          GrammarRef::new(reference.to_string(), path.clone()),
                        );
                        pending_grammar_paths.lock().unwrap().push_back(path);
                        work_verifier.lock().unwrap().add_units_of_work(1);
                      }
                      SherpaResult::MultipleErrors(mut new_errors) => {
                        errors.append(&mut new_errors)
                      }
                      SherpaResult::Err(err) => errors.push(SherpaError::SourceError {
                        loc:        tok,
                        path:       path.clone(),
                        id:         "nonexistent-import-source",
                        msg:        format!(
                          "Could not load \n\t{}{}\n",
                          base_path.to_str().unwrap(),
                          err
                        ),
                        inline_msg: "source not found".to_string(),
                        severity:   SherpaErrorSeverity::Critical,
                        ps_msg:     Default::default(),
                      }),
                      SherpaResult::None => errors.push(SherpaError::SourceError {
                        loc:        tok,
                        path:       path.clone(),
                        id:         "nonexistent-import-source",
                        msg:        format!("Could not load {}", base_path.to_str().unwrap()),
                        inline_msg: "source not found".to_string(),
                        severity:   SherpaErrorSeverity::Critical,
                        ps_msg:     Default::default(),
                      }),
                    }
                  }

                  grammars.push((path, imports_refs, grammar));
                  {
                    work_verifier.lock().unwrap().complete_one_unit_of_work();
                  }
                }
                SherpaResult::Err(err) => errors.push(err),
                _ => {}
              },
              None => {
                if work_verifier.lock().unwrap().is_complete() {
                  break;
                }
              }
            }
          }
          (grammars, errors)
        })
      })
      .map(|s| s.join().unwrap())
      .collect::<Vec<_>>()
  });

  let mut grammars = vec![];
  let mut errors = vec![];

  for (mut g, mut e) in results {
    grammars.append(&mut g);
    errors.append(&mut e);
  }

  (grammars, errors)
}

/// Loads and parses a grammar file, returning the parsed grammar node and a vector of Import nodes.
pub(crate) fn load_grammar(
  _j: &mut Journal,
  absolute_path: &PathBuf,
) -> SherpaResult<(Box<Grammar>, Vec<Box<Import>>)> {
  match read(absolute_path) {
    Ok(buffer) => match compile_grammar_ast(buffer) {
      Ok(grammar) => {
        let import_paths = grammar
          .preamble
          .iter()
          .filter_map(|a| match a {
            ASTNode::Import(import) => Some(import.clone()),
            _ => None,
          })
          .collect();
        SherpaResult::Ok((grammar, import_paths))
      }
      Err(err) => SherpaResult::Err(err),
    },
    Err(err) => SherpaResult::Err(err.into()),
  }
}

/// Resolves and verifies a grammar file path acquired from an `@IMPORT` statement exists.
///
/// If the file path does not have an extension, attempts are made to assert
/// the existence of the file path when appended with one of the following extension types
/// appended to it: `.hc`, `.hcg` `.grammar`.
///
/// Additionally, if the given file path is relative, then it is appended to the parent dir
/// path of the current grammar, whose path is provided by the `cgd`, current grammar dir,
/// argument.
pub(crate) fn resolve_grammar_path(
  path: &PathBuf,
  cgd: &PathBuf,
  extension: &[&str],
) -> SherpaResult<PathBuf> {
  SherpaResult::Ok(
    match (
      path.is_file(),
      path.extension().is_some(),
      // Ensure path is is an absolute path
      match path.is_absolute() {
        true => (path.to_owned(), false),
        false => (cgd.join(&path), cgd.join(&path).is_file()),
      },
    ) {
      // Path is relative to the given cgd
      (false, _, (path, true)) => path.canonicalize()?,
      // Attempt to verify the file path with different extensions. First valid
      // path wins.
      (false, false, (path, _)) => extension
        .iter()
        .filter_map(|ext| {
          let mut path = path.clone();
          path.set_extension(ext);
          path.canonicalize().ok()
        })
        .next()
        .ok_or(format!("Tried to load file with these extension {:?}", extension))?,

      // Default
      _ => path.canonicalize()?,
    },
  )
}
