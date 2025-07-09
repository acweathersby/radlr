use crate::utils::{_write_disassembly_to_temp_file_, _write_states_to_temp_file_};
use radlr_bytecode::compile_bytecode;
use radlr_core::*;
use radlr_rust_runtime::{parsers::error_recovery::ErrorRecoveringDatabase, types::*};
use std::path::PathBuf;

#[test]
pub fn construct_error_recovering_parser() -> RadlrResult<()> {
  let source = r#"

  IGNORE { c:sp }

  <> taco > apple ";" $

  <> apple > topic ";"

  <> topic > test "green" "toast"

  <> test > fn "{}" 

  <> fn > ("fn" | "funct" | "function") "(" field(*",") ")" "{" field(*",") "}" 

  <> field > tk:id ":" val

  <> val > c:num

  <> id > c:id(+)
  
   "#;

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let input = r#"fn()(}greentoast;;"#;

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();
  grammar.add_source_from_string(source, &root_path, false)?;

  let config = ParserConfig::default().cst_editor();
  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  pkg.parse_with_recovery(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;

  Ok(())
}

#[test]
pub fn construct_error_recovering_erlang_toy() -> RadlrResult<()> {
  let source = r#"

  IGNORE { c:sp c:nl }
  
  <> fun > tk:name args body $
  
  <> args > "()"
  
  <> body > "->" expr

  <> expr > call

  <> call > "fn " tk:name "(" ")"

  <> name > c:id(+)
  
   "#;

  let input = "test test";

  eprintln!("------------------");

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();
  grammar.add_source_from_string(source, &root_path, false)?;

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let config = ParserConfig::default().cst_editor();
  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  let result =
    pkg.parse_with_recovery(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;

  if let Some(best) = result.first() {
    for (_, sym) in &best.symbols {
      Printer::new(sym, true, &pkg).print();
      eprintln!("\n");
      Printer::new(sym, true, &pkg).print_all();
    }
    eprintln!("\n");
  }

  Ok(())
}

#[test]
pub fn temp_lab_test() -> RadlrResult<()> {
  let source = r#"

  IGNORE { c:sp c:nl }

  <> B > A(+) ";"

  <> A > "Hello" "{" tk:(c:id+)(*) "}" 
  
   "#;

  let input = "Hello{a  Hello  a  Hello } Hello{ hello bbbb } k this is the only way we should have show the make of the master 
  survival and the core of the being is is not the way we should have made this in the retrium };";

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();
  grammar.add_source_from_string(source, &root_path, false)?;

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let config = ParserConfig::default().cst_editor();
  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  //dbg!(&pkg.nonterm_id_to_name, &pkg.token_id_to_str);
  let result =
    pkg.parse_with_recovery(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;
  //dbg!(&result);
  eprintln!("------------------");
  if let Some(best) = result.first() {
    for (_, sym) in &best.symbols {
      for alt in split_alternates(sym) {
        eprintln!("{}\n", Printer::new(alt.first().unwrap(), true, &pkg).to_string());
      }
    }
    eprintln!("\n");
  }

  // assert_eq!(result.trees().first().to_string(), "<name>()->fn <name>()");

  Ok(())
}

#[test]
pub fn temp_lab_tesit2() -> RadlrResult<()> {
  let source = r#"

  IGNORE { c:sp }  

  <> goal > fn | strct
      
  <> fn > tk:(c:id+) "(" ")" ":" "{" "}" :ast  { t_FN }
  
  <> strct > tk:(c:id+) "{" "}" :ast  { t_FN }

"#;

  let input = "test () {";

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();
  grammar.add_source_from_string(source, &root_path, false)?;

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let config = ParserConfig::default().cst_editor();
  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(false, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, true)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  let result =
    pkg.parse_with_recovery(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;

  eprintln!("------------------");
  if let Some(best) = result.first() {
    for (_, sym) in &best.symbols {
      for alt in split_alternates(sym) {
        eprintln!("{}\n", Printer::new(alt.first().unwrap(), true, &pkg).to_string());
      }
    }
    eprintln!("\n");
  }

  // assert_eq!(result.trees().first().to_string(), "<name>()->fn <name>()");

  Ok(())
}

#[test]
pub fn construct_derror_recovering_erlang_toy() -> RadlrResult<()> {
  let source = r#"
  IGNORE { c:sp }

  <> sum > sum "+" num | num 

  <> num > "1" | "2" | "3" | "4" | "5" | "6" | "8" | "9" | "0"
  
   "#;

  let input = "1+2+3+7+4+5+6";

  let root_path = PathBuf::from("test.sg");
  let mut grammar = RadlrGrammar::new();
  grammar.add_source_from_string(source, &root_path, false)?;

  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();

  let mut config = ParserConfig::default().cst_editor();
  config.EXPORT_ALL_NONTERMS = true;
  let parser_data = grammar.build_db(&root_path, config)?.build_states(config, &pool)?.build_ir_parser(true, false, &pool)?;

  _write_states_to_temp_file_(&parser_data)?;

  let pkg = compile_bytecode(&parser_data, false)?;

  _write_disassembly_to_temp_file_(&pkg, parser_data.get_db(), config)?;

  let result =
    pkg.parse_with_recovery(&mut StringInput::from(input), pkg.get_entry_data_from_name("default")?, &Default::default())?;

  println!("\n=============================================");
  if let Some(best) = result.first() {
    for (_, sym) in &best.symbols {
      let str = Printer::new(sym, true, &pkg).to_string();
      println!("OPAH! {str}");
      println!("--\n");
      Printer::new(sym, true, &pkg).print_all();
    }
    println!("\n");
  }

  Ok(())
}
