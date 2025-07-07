use radlr_ascript::*;
use radlr_core::{proxy::OrderedMap, *};
use radlr_formatter::{Formatter, FormatterContext, FormatterResult, ToValue, Value};
use std::{
  collections::HashMap,
  fs::*,
  io::Write,
  path::{Path, PathBuf},
};

pub fn build_ast_source(
  db: &RadlrGrammarDatabase,
  script: &str,
  ast_path: std::path::PathBuf,
  ast_struct_name: &str,
  extra_config_properties: &[(&str, &str)],
) -> RadlrResult<()> {
  let adb: AscriptDatabase = db.into();
  Ok(if let Some(errors) = adb.get_errors() {
    for error in errors {
      eprintln!("{}", error);
    }
  } else {
    let parser = OpenOptions::new().append(false).truncate(true).write(true).create(true).open(&ast_path)?;

    adb.format(script, parser, 100, ast_struct_name, extra_config_properties)?.flush()?;
  })
}

pub fn build_parser_states(db: &RadlrGrammarDatabase, parser_config: ParserConfig) -> Result<RadlrIRParser, RadlrError> {
  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();
  let states = db.build_states(parser_config, &pool)?;
  let parser = states.build_ir_parser(true, false, &pool)?;
  Ok(parser)
}

pub fn build_parser_source(
  db: &RadlrGrammarDatabase,
  parser_script: &str,
  bytecode: radlr_rust_runtime::types::BytecodeParserDB,
  binary_out_path: std::path::PathBuf,
  parser_out_path: std::path::PathBuf,
) -> Result<(), RadlrError> {
  let parser = OpenOptions::new().append(false).truncate(true).write(true).create(true).open(&parser_out_path)?;

  let s_store = db.get_internal().string_store();

  let mut ctx: FormatterContext = FormatterContext::new("RustForm", s_store.clone());

  let states_lu = bytecode
    .state_to_token_ids_map
    .iter()
    .map(|(id, val)| (id.to_string(), val.clone().into_iter().map(|v| -> Value { v.into() }).collect::<Vec<_>>()))
    .collect::<HashMap<_, _>>();

  let nonterm_name_to_id =
    bytecode.nonterm_name_to_id.iter().map(|(id, val)| (id.clone(), Value::from(*val))).collect::<OrderedMap<_, _>>();

  let token_id_to_str =
    bytecode.token_id_to_str.iter().map(|(id, val)| (id.to_string(), val.into_val(s_store))).collect::<OrderedMap<_, _>>();

  let nonterm_id_to_address = bytecode
    .nonterm_id_to_address
    .iter()
    .map(|(id, val)| (id.to_string(), (*val).into_val(s_store)))
    .collect::<OrderedMap<_, _>>();
  let mut token_maps: std::collections::BTreeMap<Vec<u32>, usize> = OrderedMap::new();

  let state_to_token_ids_map = bytecode
    .state_to_token_ids_map
    .iter()
    .map(|(state, ids)| {
      let index = token_maps.len();
      let id = *token_maps.entry(ids.clone()).or_insert(index);
      ((*state).to_string(), id)
    })
    .collect::<OrderedMap<_, _>>();

  let token_maps = token_maps.iter().map(|(v, k)| (k.to_string(), v.clone())).collect::<OrderedMap<_, _>>();

  let bin_path = path_relative_from_abs(&binary_out_path, &parser_out_path).unwrap();
  let bin_path = bin_path.intern(s_store);

  let binary = bytecode.bytecode.as_slice();

  ctx.set_val("ir_token_lookup", Value::Obj(&states_lu));
  ctx.set_val("binary_path", Value::Str(bin_path));
  ctx.set_val("binary", Value::Obj(&binary));
  ctx.set_val("nonterm_name_to_id", Value::Obj(&nonterm_name_to_id));
  ctx.set_val("token_id_to_str", Value::Obj(&token_id_to_str));
  ctx.set_val("nonterm_id_to_address", Value::Obj(&nonterm_id_to_address));
  ctx.set_val("state_to_token_ids_map", Value::Obj(&state_to_token_ids_map));
  ctx.set_val("token_maps", Value::Obj(&token_maps));
  ctx.set_val("ALLOW_UPPER_ATTRIBUTES", Value::Int(0));

  ctx.max_width = 100;

  let f: Formatter = FormatterResult::from(parser_script).into_result()?;

  f.write_to_output(&mut ctx, parser)?.flush()?;

  Ok(())
}

fn path_relative_from_abs(path: &Path, base: &Path) -> Option<PathBuf> {
  use std::path::Component;

  // Must use absolute paths. Otherwise no action is taken.
  if !(path.is_absolute() & base.is_absolute()) {
    None
  } else {
    let mut itp = path.components().collect::<Vec<_>>();
    let mut itb = base.components().collect::<Vec<_>>();

    // Ignore file components
    let file = if path.extension().is_some() { itp.pop() } else { None };
    if base.extension().is_some() {
      itb.pop();
    }

    let mut itp = itp.into_iter();
    let mut itb = itb.into_iter();

    let mut out_path: Vec<Component> = vec![];
    loop {
      match (itp.next(), itb.next()) {
        (None, None) => break,

        (Some(a), None) => {
          out_path.push(a);
          out_path.extend(itp.by_ref());
          break;
        }

        (None, _) => out_path.push(Component::ParentDir),

        (Some(a), Some(b)) if out_path.is_empty() && a == b => (),

        (Some(a), Some(b)) if b == Component::CurDir => out_path.push(a),

        (Some(_), Some(b)) if b == Component::ParentDir => return None,

        (Some(a), Some(_)) => {
          out_path.push(Component::ParentDir);
          for _ in itb {
            out_path.push(Component::ParentDir);
          }
          out_path.push(a);
          out_path.extend(itp.by_ref());
          break;
        }
      }
    }

    if out_path.is_empty() {
      out_path.push(Component::CurDir);
    }

    if let Some(file) = file {
      out_path.push(file)
    }

    Some(out_path.iter().map(|c| c.as_os_str()).collect())
  }
}

#[test]
fn relative_paths() {
  let path_a = PathBuf::from("/test/test/a/a.r");
  let path_b = PathBuf::from("/test/test/a/b.r");

  let path_c = path_relative_from_abs(&path_a, &path_b).unwrap();

  assert_eq!(path_c.as_os_str().to_str().unwrap(), "./a.r");

  let path_a = PathBuf::from("./test/a/a.r");
  let path_b = PathBuf::from("/test/test/a/b.r");

  assert!(path_relative_from_abs(&path_a, &path_b).is_none());

  let path_a = PathBuf::from("./test/a/a.r");
  let path_b = PathBuf::from("../test/test/a/b.r");

  assert!(path_relative_from_abs(&path_a, &path_b).is_none());

  let path_a = PathBuf::from("/test/a/");
  let path_b = PathBuf::from("/test/a/");

  let path_c = path_relative_from_abs(&path_a, &path_b).unwrap();

  assert_eq!(path_c.as_os_str().to_str().unwrap(), ".");
}
