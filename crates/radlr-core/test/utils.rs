use crate::{
  types::ParserConfig,
  DBPackage,
  GrammarDatabase,
  GrammarSoup,
  ParseStatesMap,
  RadlrGrammar,
  RadlrGrammarDatabase,
  RadlrIRParser,
  RadlrResult,
  TestPackage,
};
use std::{num::NonZero, path::PathBuf, str::FromStr};

/// Simple single thread compilation of a grammar source string.
/// `test_fn` is called after a successful compilation of parse states.
pub fn build_parse_states_from_source_str<'a, T>(
  source: &str,
  source_path: PathBuf,
  optimize: bool,
  test_fn: &dyn Fn(TestPackage) -> RadlrResult<T>,
) -> RadlrResult<T> {
  #[cfg(not(feature = "wasm-target"))]
  let pool = crate::types::worker_pool::StandardPool::new_with_max_workers().unwrap();

  #[cfg(feature = "wasm-target")]
  let pool = crate::types::worker_pool::SingleThreadPool {};

  if optimize {
    test_fn(
      RadlrGrammar::new()
        .add_source_from_string(source, &source_path, false)?
        .build_db(source_path, Default::default())?
        .build_states(Default::default(), &pool)?
        .build_ir_parser(true, false, &pool)?
        .into(),
    )
  } else {
    test_fn(
      RadlrGrammar::new()
        .add_source_from_string(source, &source_path, false)?
        .build_db(source_path, Default::default())?
        .build_states(Default::default(), &pool)?
        .build_ir_parser(false, false, &pool)?
        .into(),
    )
  }
}

pub fn build_parse_states_from_multi_sources2<'a, T>(
  source_path: PathBuf,
  optimize: bool,
  test_fn: &dyn Fn(TestPackage) -> RadlrResult<T>,
  config: ParserConfig,
) -> RadlrResult<T> {
  let mut grammar = RadlrGrammar::new();

  #[cfg(not(feature = "wasm-target"))]
  let pool = crate::types::worker_pool::StandardPool::new_with_max_workers()?;

  #[cfg(feature = "wasm-target")]
  let pool = crate::types::worker_pool::SingleThreadPool {};

  grammar.add_source(source_path.clone())?;

  if optimize {
    test_fn(grammar.build_db(source_path, config)?.build_states(config, &pool)?.build_ir_parser(true, false, &pool)?.into())
  } else {
    test_fn(grammar.build_db(source_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?.into())
  }
}

pub fn build_parse_states_from_multi_sources<'a, T>(
  sources: &[&str],
  source_path: PathBuf,
  optimize: bool,
  test_fn: &dyn Fn(TestPackage) -> RadlrResult<T>,
  config: ParserConfig,
) -> RadlrResult<T> {
  let mut grammar = RadlrGrammar::new();

  #[cfg(not(feature = "wasm-target"))]
  let pool = crate::types::worker_pool::StandardPool::new_with_max_workers()?;

  #[cfg(feature = "wasm-target")]
  let pool = crate::types::worker_pool::SingleThreadPool {};

  for (index, source) in sources.iter().enumerate() {
    let source_path = source_path.join("ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().nth(index).unwrap().to_string());
    grammar.add_source_from_string(source, source_path, false)?;
  }

  let root_path = source_path.join("A");

  if optimize {
    test_fn(grammar.build_db(root_path, config)?.build_states(config, &pool)?.build_ir_parser(true, false, &pool)?.into())
  } else {
    test_fn(grammar.build_db(root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?.into())
  }
}

/// Compile a parser Data base
pub fn build_parse_db_from_source_str<'a, T>(
  source: &str,
  source_path: PathBuf,
  test_fn: &dyn Fn(DBPackage) -> RadlrResult<T>,
) -> RadlrResult<T> {
  test_fn(
    RadlrGrammar::new().add_source_from_string(source, &source_path, false)?.build_db(&source_path, Default::default())?.into(),
  )
}

/// Writes to a debug file for testing
#[cfg(debug_assertions)]
pub fn write_debug_file<FileName: AsRef<std::path::Path>, Data: AsRef<[u8]>>(
  db: &GrammarDatabase,
  file_name: FileName,
  data: Data,
  append: bool,
) -> RadlrResult<()> {
  use std::{env::*, fs::*, io::Write};

  let file_dir = temp_dir().join("radlr_testing").join(db.name_string());

  let file_path = file_dir.join(file_name);

  create_dir_all(file_dir)?;

  let mut file = OpenOptions::new().append(append).truncate(!append).write(true).create(true).open(file_path)?;
  file.write_all(data.as_ref())?;
  file.flush()?;

  Ok(())
}

#[cfg(debug_assertions)]
impl GrammarDatabase {
  /// Creates a test database that can be used to statically
  /// test components that rely on the database without actually
  /// having to define a grammar. The objects stored in this database are
  /// equivalent to those that would be derived from the following grammar
  /// ```radlr
  /// 
  /// IGNORE { c:sp c:nl }
  ///
  /// <> A > "{" B(+) "}"
  ///
  /// <> B > c:id tk:C?
  ///
  /// <> C > c:num c:num(+)
  /// ```
  pub fn test_lr() -> RadlrResult<RadlrGrammarDatabase> {
    let mut grammar = RadlrGrammar::new();

    let path = PathBuf::from("/test/grammar.sg");

    grammar.add_source_from_string(
      r##"     
     IGNORE { c:sp c:nl }
    
     <> A > "{" B(+) "}"
    
     <> B > c:id tk:C?
    
     <> C > c:num c:num(+)
    "##,
      &path,
      false,
    )?;

    let db = grammar.build_db(&path, Default::default())?;

    Ok(db)
  }
}
