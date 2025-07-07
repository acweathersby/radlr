use crate::utils::{_write_disassembly_to_temp_file_, _write_states_to_temp_file_};
use radlr_bytecode::compile_bytecode;
use radlr_core::*;
use radlr_rust_runtime::{parsers::cst::EditGraph, types::*};
use std::{path::PathBuf, rc::Rc};

#[test]
pub fn construct_error_recovering_parser() -> RadlrResult<()> {
  let source = r#"
  
  IGNORE { c:sp }
  <> B > A(+) ";"

  <> A > "Hello" "{" tk:(c:id+)(+) "}" 
  
   "#;

  let input = r#"fn(das:2){}{}green toast;;"#;
  //let input = r#"d"#;

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  grammar.add_source_from_string(source, &root_path, false)?;

  let config = ParserConfig::default().cst_editor();

  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  let pkg = Rc::new(pkg);

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  let graph: EditGraph<StringInput, BytecodeParserDB> =
    EditGraph::parse(pkg.default_entrypoint(), input.to_string(), pkg.clone())?;

  eprintln!("\n\n--");
  Printer::new(graph.cst().unwrap().as_ref(), true, pkg.as_ref()).print();
  eprintln!("\n\n");

  let result = graph.patch_insert(&graph.cst().unwrap(), 8, "2,     d").expect("Insert should produce result");

  eprintln!("{:#?}", result);

  eprintln!("\n\n--");
  Printer::new(graph.cst().unwrap().as_ref(), true, pkg.as_ref()).print();
  eprintln!("\n\n");

  graph.patch_insert(&graph.cst().unwrap(), 8, ")} fn(");

  eprintln!("\n\n--");
  Printer::new(graph.cst().unwrap().as_ref(), true, pkg.as_ref()).print();
  eprintln!("\n\n");

  Ok(())
}
