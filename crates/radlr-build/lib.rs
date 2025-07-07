#![allow(unused)]
use radlr_core::RadlrGrammar;
pub use radlr_core::RadlrResult;
use std::path::{Path, PathBuf};
use targets::rust::RustConfig;

mod targets;

#[derive(Clone, Copy)]
pub enum TargetLanguage {
  Rust,
  TypeScript,
  JavaScript,
  C,
  Cpp,
  Llvm,
}

#[derive(Clone, Copy)]
pub enum ParserType {
  /// Builds a bytecode parser to be run by the Radlr bytecode interpreter
  /// of target langauge.
  Bytecode,
  /// Build a binary bytecode parser for the target machine. Not all target
  /// languages support this parsing mode.
  Binary,
}

/// Configurations for artifacts built by radlr-build.
#[derive(Clone, Copy)]
pub struct BuildConfig<'a> {
  /// Path of the root grammar file.
  root_grammar_path: &'a Path,

  /// Compile grammar into a parser
  ///
  /// Defaults to true
  pub build_parser: bool,

  /// Create an ast artifact file in the target language based on `:ast`
  /// definitions found in grammar files.
  ///
  /// Defaults to true
  pub build_ast: bool,

  /// Name for the AST type when creating ast file.
  ///
  /// Defaults to `ASTNode`
  pub ast_struct_name: &'a str,

  pub include_debug_symbols: bool,

  /// Directory to place library type artifacts including
  /// binary files and shared/static libraries.
  ///
  /// Defaults to the parent dir of `root_grammar_path` if lib out is not
  /// set.
  pub lib_out: &'a Path,

  /// Directory to place source type artifacts such as
  /// generated parser and AST code files.
  ///
  /// Defaults the parent dir of `root_grammar_path` if lib out is not
  /// set.
  pub source_out: &'a Path,

  /// The type of parser to construct.
  ///
  /// Defaults to Bytecode.
  pub parser_type: ParserType,

  /// Paths to search for `radlr` input files.
  pub include_paths: &'a [&'a Path],

  /// Configurations specific to the Rust language target
  pub rust: RustConfig,

  /// Name prefix to add to output files
  pub name_prefix: Option<&'a str>,
}

impl<'a> BuildConfig<'a> {
  pub fn new(root_grammar: &'a Path) -> Self {
    BuildConfig {
      ast_struct_name:       "ASTNode",
      lib_out:               &root_grammar.parent().unwrap_or(root_grammar),
      source_out:            &root_grammar.parent().unwrap_or(root_grammar),
      build_ast:             true,
      build_parser:          true,
      include_debug_symbols: false,
      root_grammar_path:     root_grammar,
      parser_type:           ParserType::Bytecode,
      include_paths:         &[],
      rust:                  Default::default(),
      name_prefix:           Default::default(),
    }
  }
}

/// Build a Radlr parser from a grammar source string
pub fn source_string_build<'b>(
  build_config: BuildConfig<'b>,
  parser_config: radlr_core::ParserConfig,
  source: &str,
  target: TargetLanguage,
) -> RadlrResult<()> {
  let mut local_build_config = build_config;

  let BuildConfig { lib_out, source_out, root_grammar_path, .. } = &mut local_build_config;

  std::fs::create_dir_all(*lib_out)?;
  std::fs::create_dir_all(*source_out)?;

  let canonical_libout = lib_out.canonicalize()?;
  let canonical_source_out = source_out.canonicalize()?;

  (*lib_out) = &canonical_libout;
  (*source_out) = &canonical_source_out;

  let db = RadlrGrammar::new()
    .add_source_from_string(&source, &root_grammar_path, false)?
    .build_db(&root_grammar_path, parser_config)?;

  match target {
    TargetLanguage::Rust => {
      targets::rust::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::TypeScript => {
      targets::typescript::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::JavaScript => {
      targets::javascript::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::Cpp => {
      todo!("Build Cpp: Not yet supported")
    }
    TargetLanguage::C => {
      todo!("Build C: Not yet supported")
    }
    TargetLanguage::Llvm => {
      todo!("Build Llvm: Not yet supported")
    }
  }

  Ok(())
}

/// Build a Radlr parser from a grammar file
pub fn fs_build<'b>(
  build_config: BuildConfig<'b>,
  parser_config: radlr_core::ParserConfig,
  target: TargetLanguage,
) -> RadlrResult<()> {
  let mut local_build_config = build_config;

  let BuildConfig { lib_out, source_out, root_grammar_path, .. } = &mut local_build_config;

  std::fs::create_dir_all(*lib_out)?;
  std::fs::create_dir_all(*source_out)?;

  let canonical_libout = lib_out.canonicalize()?;
  let canonical_source_out = source_out.canonicalize()?;

  (*lib_out) = &canonical_libout;
  (*source_out) = &canonical_source_out;

  let resolved_root_path = RadlrGrammar::resolve_to_grammar_file(root_grammar_path)?;

  let db = RadlrGrammar::new().add_source(&resolved_root_path)?.build_db(resolved_root_path, parser_config)?;

  match target {
    TargetLanguage::Rust => {
      targets::rust::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::TypeScript => {
      targets::typescript::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::JavaScript => {
      targets::javascript::build(&db, local_build_config, parser_config)?;
    }
    TargetLanguage::Cpp => {
      todo!("Build Cpp: Not yet supported")
    }
    TargetLanguage::C => {
      todo!("Build C: Not yet supported")
    }
    TargetLanguage::Llvm => {
      todo!("Build Llvm: Not yet supported")
    }
  }

  Ok(())
}

#[test]
fn builds_basic_grammar() -> RadlrResult<()> {
  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;
  let path = root.join("test_grammar.radlr");
  let output = root.join("build");

  let mut build_config = BuildConfig::new(&path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  fs_build(build_config, Default::default(), TargetLanguage::Rust)?;

  Ok(())
}

#[test]
fn builds_basic_grammar_javascript() -> RadlrResult<()> {
  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;

  let path = root.join("test_grammar.radlr");
  let output = root.join("build");

  let mut build_config = BuildConfig::new(&path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  fs_build(build_config, Default::default(), TargetLanguage::JavaScript)?;

  Ok(())
}

#[test]
fn builds_basic_grammar_javascript_a() -> RadlrResult<()> {
  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;

  println!("{:?}", root.join("../../grammars/syntax_highlight/syntax_highlight.radlr").canonicalize()?);

  let path = root.join("../../grammars/syntax_highlight/syntax_highlight.radlr").canonicalize()?;
  let output = root.join("build");

  let mut build_config = BuildConfig::new(&path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  fs_build(build_config, Default::default(), TargetLanguage::JavaScript)?;

  Ok(())
}

#[test]
fn builds_radlr_lang() -> RadlrResult<()> {
  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;

  println!("{:?}", root.join("../../grammars/radlr/3.0.0-pre-bootstrap/radlr.radlr").canonicalize()?);

  let path = root.join("../../grammars/radlr/3.0.0-pre-bootstrap/radlr.radlr").canonicalize()?;
  let output = root.join("build");

  let mut build_config = BuildConfig::new(&path);
  build_config.source_out = &output;
  build_config.lib_out = &output;
  build_config.build_parser = false;

  fs_build(build_config, Default::default(), TargetLanguage::Rust)?;

  Ok(())
}

#[test]
fn builds_rum_lang() -> RadlrResult<()> {
  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;

  let path = root.join("../../../lib_rum_common/crates/language/grammar/low_level.radlr").canonicalize()?;
  let output = root.join("build");

  let mut build_config = BuildConfig::new(&path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  fs_build(build_config, Default::default(), TargetLanguage::Rust)?;

  Ok(())
}

#[test]
fn build_test() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > ( [ ( R | B )(*)^s T?^eoi ]  :ast [$s, $eoi] )^s 

  <> T > "tt" :ast { t_T }

  <> R > "test" :ast { t_R }

  <> B >  "nest" :ast { t_B }
    
   "##;

  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;
  let output = root.join("build");
  let default_path: PathBuf = "grammar.radlr".into();

  let mut build_config = BuildConfig::new(&default_path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  source_string_build(build_config, Default::default(), source, TargetLanguage::Rust)?;

  Ok(())
}

#[test]
fn builds_values() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > C 

  <> C > B | "R" :ast  { t_R }

  <> B > "X" :ast {t_X} | "Y" :ast {t_Y} | "Z" :ast {t_Z} | W

  <> W > "W" :ast { t_W }
  

   "##;

  let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).canonicalize()?;
  let output = root.join("build");
  let default_path: PathBuf = "grammar.radlr".into();

  let mut build_config = BuildConfig::new(&default_path);
  build_config.source_out = &output;
  build_config.lib_out = &output;

  source_string_build(build_config, Default::default(), source, TargetLanguage::Rust)?;

  Ok(())
}
