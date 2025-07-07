use crate::utils::{_write_disassembly_to_temp_file_, _write_states_to_temp_file_};
use radlr_bytecode::compile_bytecode;
use radlr_core::*;
use radlr_rust_runtime::types::*;
use std::path::PathBuf;

#[test]
pub fn construct_forking_parser_glr() -> RadlrResult<()> {
  let source = r#"

  IGNORE { c:sp } 


  <> F > E "test"

  <> E > A " !" | B

  <> B > "id" "()" " !"

  <> A > "id" "()"
  
   "#;
  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let input = r#"id () ! test"#;

  let root_path = PathBuf::from("test.sg");

  let mut grammar = RadlrGrammar::new();

  grammar.add_source_from_string(source, &root_path, false)?;

  let config = ParserConfig::default().cst_editor().use_fork_states(true);

  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  let mut parser = pkg.get_parser()?;

  parser.fork_parse(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;

  Ok(())
}
