use super::{
  compile::{
    get_indexed_body_ref,
    get_named_body_ref,
    get_production_types,
    get_specified_vector_from_generic_vec_values,
    get_struct_type_from_node,
    production_types_are_structs,
  },
  types::*,
};
use hctk_core::{
  grammar::data::ast::{
    ASTNode,
    AST_IndexReference,
    AST_NamedReference,
    AST_Struct,
    AST_Token,
    AST_Vector,
    AST_BOOL,
    AST_F32,
    AST_F64,
    AST_I16,
    AST_I32,
    AST_I64,
    AST_I8,
    AST_NUMBER,
    AST_STRING,
    AST_U16,
    AST_U32,
    AST_U64,
    AST_U8,
  },
  types::*,
  writer::code_writer::*,
};
use std::{
  collections::{BTreeMap, BTreeSet, VecDeque},
  io::{Result, Write},
  vec,
};

pub fn write<W: Write>(ast: &AScriptStore, w: &mut CodeWriter<W>) -> Result<()> {
  w.indent_spaces(2);

  w.wrtln("use hctk::types::*;")?.newline()?;

  build_astnode_enum(w, ast)?;

  w.newline()?
    .wrtln(&format!("pub type {} = HCObj<{}>;", ast.gen_name(), ast.name))?
    .newline()?
    .wrtln(&format!("impl HCObjTrait for {} {{}}", ast.name))?
    .newline()?;

  build_types_utils(w, ast)?;

  build_structs(ast, w)?;

  build_functions(ast, w)?;

  Ok(())
}

fn build_astnode_enum<W: Write>(w: &mut CodeWriter<W>, ast: &AScriptStore) -> Result<()> {
  w.wrtln(&format!("#[derive(Debug, Clone)]\npub enum {} {{", ast.name))?.indent();
  for _struct in ast.structs.values() {
    w.write_line(&format!("{0}(Box<{0}>),", _struct.type_name))?;
  }
  w.dedent().wrtln("}")?;

  w.wrtln(&format!("impl {} {{", ast.name))?.indent();
  for _struct in ast.structs.values() {
    w.wrtln(&format!("pub fn as_{0}(&self) -> Option<&{0}> {{", _struct.type_name))?
      .indent()
      .wrtln("match self {")?
      .indent()
      .wrtln(&format!("Self::{}(val) => Some(val.as_ref()),", _struct.type_name))?
      .wrtln("_ => None")?
      .dedent()
      .wrtln("}")?
      .dedent()
      .wrtln("}")?;
    w.wrtln(&format!("pub fn as_{0}_mut(&mut self) -> Option<&mut {0}> {{", _struct.type_name))?
      .indent()
      .wrtln("match self {")?
      .indent()
      .wrtln(&format!("Self::{}(val) => Some(val.as_mut()),", _struct.type_name))?
      .wrtln("_ => None")?
      .dedent()
      .wrtln("}")?
      .dedent()
      .wrtln("}")?;
  }
  w.dedent().wrtln("}")?;

  Ok(())
}

fn build_types_utils<W: Write>(w: &mut CodeWriter<W>, ast: &AScriptStore) -> Result<()> {
  w.wrtln(&format!("#[derive(Eq, PartialEq, Clone, Copy, Debug)]\npub enum {}Type {{", ast.name))?
    .indent()
    .write_line("Undefined,")?;
  for name in gen_names {
    w.wrtln(name)?.wrt(",")?;
  }
  for AScriptStruct { type_name, .. } in ast.structs.values() {
    w.wrtln(type_name)?.wrt(",")?;
  }
  w.dedent().wrtln("}")?;
  w.wrtln(&format!("pub trait Get{0} {{ fn get_type(&self) -> {0}; }}", ast.type_name()))?;
  w.wrtln(&format!("impl Get{0} for {1} {{", ast.type_name(), ast.name))?.indent();
  w.wrtln(&format!("fn get_type(&self) -> {} {{", ast.type_name()))?.indent();
  w.wrtln("match self{")?.indent();
  for AScriptStruct { type_name, .. } in ast.structs.values() {
    w.wrtln(&format!("{0}::{2}(..) => {1}::{2}", ast.name, ast.type_name(), type_name))?
      .wrt(",")?;
  }
  w.write_line("_ => ASTNodeType::NONE,")?;
  w.dedent().wrtln("}")?.dedent().wrtln("}")?.dedent().wrtln("}")?;

  w.wrtln(&format!("impl Get{} for {} {{", ast.type_name(), ast.gen_name()))?.indent();
  w.wrtln(&format!("fn get_type(&self) -> {} {{", ast.type_name()))?.indent();
  w.wrtln("match self{")?.indent();
  for name in gen_names {
    if name == "NODE" {
      w.write_line(&format!("{}::NODE(node) => node.get_type(),", ast.gen_name()))?;
    } else if name == "NONE" {
    } else {
      w.write_line(&format!("{0}::{2}(..) => {1}::{2},", ast.gen_name(), ast.type_name(), name))?;
    }
  }
  w.write_line(&format!("_ => {}::NONE,", ast.type_name()))?;
  w.dedent().wrtln("}")?.dedent().wrtln("}")?.dedent().wrtln("}")?;
  Ok(())
}

fn build_functions<W: Write>(ast: &AScriptStore, w: &mut CodeWriter<W>) -> Result<()> {
  let g = ast.g.clone();
  let ordered_bodies = g
    .rules
    .iter()
    .filter_map(|(_, b)| {
      if g.parse_productions.contains(&b.prod_id) {
        Some((b.bytecode_id, b))
      } else {
        None
      }
    })
    .collect::<BTreeMap<_, _>>();

  let mut resize_fns = BTreeSet::new();
  let fn_args = format!("args: &mut Vec<{0}>, tok: Token", ast.gen_name());

  // Build reduce functions -------------------------------------
  w.wrtln(&format!("fn noop_fn({}){{}}", fn_args))?.newline()?;

  let mut refs = vec![];

  for (id, body) in &ordered_bodies {
    let prod_id = body.prod_id;
    let prod_data = ast.prod_types.get(&prod_id).unwrap();

    if prod_data.len() != 1 {
      unreachable!(
        "\n\nProduction result not been resolved\n[{}] == {}\n\n\n{}\n\n",
        ast.g.get_production_plain_name(&prod_id),
        body.item().blame_string(&ast.g),
        prod_data
          .iter()
          .map(|(p, _)| { p.debug_string(Some(&ast.g)) })
          .collect::<Vec<_>>()
          .join("\n")
      )
    };

    let mut temp_writer = w.checkpoint();
    let mut noop = 0;
    let fn_name = format!("ast_fn{:0>3}", body.bytecode_id);

    temp_writer
      .wrtln(&format!(
        "/*\n{}\n*/\nfn {}({}){{",
        body.item().blame_string(&ast.g).replace("*/", "* /"),
        fn_name,
        fn_args
      ))?
      .indent();

    if body.reduce_fn_ids.is_empty() {
      if body.len > 1 {
        resize_fns.insert(body.len);
      }
      noop = 1;
    } else {
      let mut ref_index = body.syms.len();

      for function_id in &body.reduce_fn_ids {
        match g.reduce_functions.get(function_id) {
          Some(ReduceFunctionType::Ascript(function)) => match &function.ast {
            ASTNode::AST_Struct(box ast_struct) => {
              if let AScriptTypeVal::Struct(struct_type) = get_struct_type_from_node(ast_struct) {
                let _ref =
                  build_struct_constructor(ast, body, &struct_type, ast_struct, &mut ref_index, 0)?;

                let indices = _ref.get_indices();

                for i in (0..body.syms.len()).rev() {
                  if indices.contains(&i) {
                    temp_writer.wrtln(&format!("let i{} = args.pop().unwrap();", i))?;
                  } else {
                    temp_writer.wrtln("args.pop();")?;
                  }
                }

                temp_writer.write_line(&_ref.to_init_string())?;

                temp_writer.write_line(&format!(
                  "args.push({}::NODE({}::{}(Box::new({}))))",
                  ast.gen_name(),
                  ast.name,
                  ast.structs.get(&struct_type).unwrap().type_name,
                  _ref.get_ref_string()
                ))?;
                break;
              }
            }
            ASTNode::AST_Statements(box statements) => {
              let mut reference = String::new();
              let mut return_type = AScriptTypeVal::Undefined;
              let mut refs = BTreeSet::new();
              let mut statement_writer = temp_writer.checkpoint();

              for (i, statement) in statements.statements.iter().enumerate() {
                match render_expression(ast, statement, body, &mut ref_index, i) {
                  Some(_ref) => {
                    refs.append(&mut _ref.get_indices());
                    return_type = _ref.ast_type.clone();
                    reference = _ref.get_ref_string();
                    statement_writer.write_line(&_ref.to_init_string())?;
                  }
                  None => {}
                }
              }

              for i in (0..body.syms.len()).rev() {
                if refs.contains(&i) {
                  temp_writer.wrtln(&format!("let i{} = args.pop().unwrap();", i))?;
                } else {
                  temp_writer.wrtln("args.pop();")?;
                }
              }

              temp_writer.merge_checkpoint(statement_writer)?;

              let return_type = match return_type {
                AScriptTypeVal::Undefined | AScriptTypeVal::GenericVec(None) => {
                  prod_data.iter().next().unwrap().0.into()
                }
                r => r,
              };

              match return_type {
                AScriptTypeVal::GenericStructVec(..)
                | AScriptTypeVal::TokenVec
                | AScriptTypeVal::StringVec
                | AScriptTypeVal::Bool(..)
                | AScriptTypeVal::F32(..)
                | AScriptTypeVal::F64(..)
                | AScriptTypeVal::U64(..)
                | AScriptTypeVal::U32(..)
                | AScriptTypeVal::U16(..)
                | AScriptTypeVal::U8(..)
                | AScriptTypeVal::I64(..)
                | AScriptTypeVal::I32(..)
                | AScriptTypeVal::I16(..)
                | AScriptTypeVal::I8(..)
                | AScriptTypeVal::U8Vec
                | AScriptTypeVal::U16Vec
                | AScriptTypeVal::U32Vec
                | AScriptTypeVal::U64Vec
                | AScriptTypeVal::I8Vec
                | AScriptTypeVal::I16Vec
                | AScriptTypeVal::I32Vec
                | AScriptTypeVal::I64Vec
                | AScriptTypeVal::F32Vec
                | AScriptTypeVal::F64Vec
                | AScriptTypeVal::String(..)
                | AScriptTypeVal::Token => temp_writer.write_line(&format!(
                  "args.push({}::{}({}))",
                  ast.gen_name(),
                  return_type.hcobj_type_name(None),
                  &reference
                ))?,
                r => {
                  println!("{}", r.debug_string(None));
                  temp_writer.write_line(&reference)?
                }
              };
            }
            _ => {
              noop = 2;
            }
          },
          _ => {
            noop = 3;
          }
        }
      }
    }

    temp_writer.dedent().wrtln("}")?;
    temp_writer.newline()?;

    if noop == 0 {
      w.merge_checkpoint(temp_writer)?;
      refs.push(format!("/* {} */ {}", id, fn_name));
    } else {
      if body.len > 1 {
        refs.push(format!("/* {} {} */ noop_fn_{}", id, noop, body.len));
      } else {
        refs.push(format!("/* {} {} */ noop_fn", id, noop));
      }
    }
  }

  for size in resize_fns {
    let fn_name = format!("noop_fn_{}", size);
    w.wrtln(&format!("fn {}({}){{", fn_name, fn_args))?.indent();
    w.write_line(&format!("args.resize(args.len() - {},{}::NONE);", size, ast.gen_name()))?;
    w.write_line(&format!("args.push({}::TOKEN(tok));", ast.gen_name()))?;
    w.dedent().wrtln("}")?;
    w.newline()?;
  }

  // Reduce Function Array -----------------------

  w.wrt(&format!(
    "pub const REDUCE_FUNCTIONS:[ReduceFunction<{}>; {}] = [",
    ast.name,
    ordered_bodies.len()
  ))?
  .indent()
  .wrt("\n")?;

  w.write(&refs.join(",\n"))?;

  w.dedent().wrtln("];")?.newline()?;

  Ok(())
}

fn build_struct_constructor(
  ast: &AScriptStore,
  body: &Rule,
  struct_type: &AScriptStructId,
  ast_struct: &AST_Struct,
  ref_index: &mut usize,
  type_slot: usize,
) -> Result<Ref> {
  let archetype_struct = ast.structs.get(struct_type).unwrap();
  let mut writer = StringBuffer::default();
  let ast_struct_props = ast_struct
    .props
    .iter()
    .filter_map(|p| {
      if let ASTNode::AST_Property(prop) = p {
        Some((prop.id.clone(), prop))
      } else {
        None
      }
    })
    .collect::<BTreeMap<_, _>>();

  writer.wrt(&format!("{0}::new(", archetype_struct.type_name))?.indent().wrt("\n")?;

  let mut predecessors = vec![];

  for (_, val_ref) in archetype_struct.prop_ids.iter().enumerate().map(|(i, prop_id)| {
    let struct_prop_ref = if let Some(ast_prop) = ast_struct_props.get(&prop_id.name) {
      let property = ast.props.get(prop_id).unwrap();
      let ref_ = render_expression(ast, &ast_prop.value, body, ref_index, i + type_slot * 100);
      let (string, ref_) =
        create_type_initializer_value(ref_, &(&property.type_val).into(), property.optional, ast);
      if let Some(ref_) = ref_ {
        predecessors.push(ref_);
      }

      string
    } else {
      get_default_value(prop_id, ast)
    };

    (prop_id.name.clone(), struct_prop_ref)
  }) {
    writer.wrt(&format!("{}, ", val_ref))?;
  }

  if archetype_struct.include_token {
    writer.wrt(&format!("{}, ", "tok"))?;
  }

  writer.dedent().write_line("\n)")?;

  (*ref_index) += 1;

  let mut ref_ = Ref::new(
    *ref_index,
    type_slot,
    String::from_utf8(writer.into_output()).unwrap(),
    AScriptTypeVal::Struct(*struct_type),
  );

  ref_.add_predecessors(predecessors);

  Ok(ref_)
}

pub fn create_type_initializer_value(
  ref_: Option<Ref>,
  type_val: &AScriptTypeVal,
  optional: bool,
  ast: &AScriptStore,
) -> (String, Option<Ref>) {
  match ref_ {
    Some(ref_) => {
      let mut string = match type_val {
        AScriptTypeVal::GenericStructVec(structs_ids) if structs_ids.len() == 1 => {
          format!(
                "{}.into_iter().map(|v|match v {{ {}::{}(node) => node, _ => panic!(\"could not convert\")}}).collect::<Vec<_>>()",
                ref_.get_ref_string(),
                ast.name,
                ast.structs.get(&structs_ids.first().unwrap().into()).unwrap().type_name
              )
        }
        _ => ref_.get_ref_string(),
      };

      let ref_ = match type_val {
        AScriptTypeVal::Struct(..) => node_to_struct(ref_, ast),
        _ => ref_,
      };

      if optional
        && matches!(type_val, AScriptTypeVal::Struct(..) | AScriptTypeVal::GenericStruct(..))
      {
        string = format!("Some({})", string);
      }

      (string, Some(ref_))
    }
    None => ("Default::default()".to_string(), None),
  }
}

fn build_structs<W: Write>(ast: &AScriptStore, o: &mut CodeWriter<W>) -> Result<()> {
  // Build structs
  for (_, AScriptStruct { include_token, prop_ids: properties, type_name, .. }) in &ast.structs {
    let mut properties = properties
      .iter()
      .map(|p| {
        let AScriptProp { type_val, optional, .. } = ast.props.get(p).unwrap();
        (p.name.clone(), ascript_type_to_string(&type_val.into(), ast), type_val.into(), *optional)
      })
      .collect::<Vec<_>>();

    if *include_token {
      properties.push(("tok".to_string(), "Token".to_string(), AScriptTypeVal::Token, false));
    }

    o.wrtln(&format!("#[derive(Debug, Clone)]\npub struct {} {{", type_name))?.indent();

    for (name, type_string, type_val, optional) in &properties {
      match optional {
        true => match type_val {
          AScriptTypeVal::Struct(..) | AScriptTypeVal::GenericStruct(..) => {
            o.write_line(&format!("pub {}: Option<{}>,", name, type_string))?
          }
          _ => o.write_line(&format!("pub {}: {},", name, type_string))?,
        },
        false => o.write_line(&format!("pub {}: {},", name, type_string))?,
      };
    }

    o.dedent().wrtln("}")?;

    // Create the Nodes Member functions

    o.wrtln(&format!("impl {} {{", type_name))?.indent();

    // NODE::new

    o.wrtln("#[inline]\npub fn new (")?.indent();

    for (name, type_string, type_val, optional) in &properties {
      match optional {
        true => match type_val {
          AScriptTypeVal::Struct(..) | AScriptTypeVal::GenericStruct(..) => {
            o.write_line(&format!("{}:Option<{}>,", name, type_string))?
          }
          _ => o.write_line(&format!("{}:{},", name, type_string))?,
        },
        false => o.write_line(&format!("{}:{},", name, type_string))?,
      };
    }
    o.dedent().wrtln(") -> Self {")?.indent().wrtln("Self{")?.indent();

    for (name, ..) in &properties {
      o.write_line(&format!("{},", name))?;
    }

    o.dedent().wrtln("}")?.dedent().wrtln("}")?;

    // NODE::get_type
    o.write_line(&format!(
      "pub fn get_type(&self) -> {0}Type {{ {0}Type::{1} }}",
      ast.name, type_name
    ))?;

    o.dedent().wrtln("}")?;
  }

  Ok(())
}

pub fn render_expression(
  ast: &AScriptStore,
  ast_expression: &ASTNode,
  body: &Rule,
  ref_index: &mut usize,
  type_slot: usize,
) -> Option<Ref> {
  let (b, s) = (body, ast);

  match ast_expression {
    ASTNode::AST_Struct(ast_struct) => {
      if let AScriptTypeVal::Struct(struct_type) = get_struct_type_from_node(ast_struct) {
        if let Ok(_ref) =
          build_struct_constructor(ast, body, &struct_type, ast_struct, ref_index, type_slot)
        {
          Some(_ref)
        } else {
          None
        }
      } else {
        None
      }
    }
    ASTNode::AST_Token(box AST_Token {}) => Some(Ref::token(bump_ref_index(ref_index), type_slot)),
    ASTNode::AST_Add(..) => Some(Ref::token(bump_ref_index(ref_index), type_slot)),
    ASTNode::AST_Vector(box AST_Vector { initializer, .. }) => {
      let mut results = initializer
        .iter()
        .filter_map(|n| render_expression(ast, n, body, ref_index, type_slot))
        .collect::<VecDeque<_>>();

      if results.is_empty() {
        Some(Ref::new(
          bump_ref_index(ref_index),
          type_slot,
          "vec![];".to_string(),
          AScriptTypeVal::GenericVec(None),
        ))
      } else {
        let types = results.iter().map(|t| t.ast_type.clone()).collect::<BTreeSet<_>>();

        let mut vector_ref = if results[0].ast_type.is_vec() {
          results.pop_front().unwrap()
        } else {
          Ref::new(
            bump_ref_index(ref_index),
            type_slot,
            "vec![]".to_string(),
            get_specified_vector_from_generic_vec_values(&types),
          )
        };

        for mut _ref in results {
          if _ref.ast_type.is_vec() {
            _ref.make_mutable();
            let val_ref = _ref.get_ref_string();
            vector_ref
              .add_post_init_expression(format!("%%.append(&mut {})", val_ref))
              .make_mutable()
          } else {
            let val_ref = _ref.get_ref_string();
            vector_ref.add_post_init_expression(format!("%%.push({})", val_ref)).make_mutable()
          };

          vector_ref.add_predecessor(_ref);
        }

        Some(vector_ref)
      }
    }
    ASTNode::AST_STRING(box AST_STRING { value, .. }) => match value {
      ASTNode::AST_NUMBER(box AST_NUMBER { value, .. }) => Some(Ref::new(
        bump_ref_index(ref_index),
        type_slot,
        format!("\"{}\".to_string()", value),
        AScriptTypeVal::String(Some(value.to_string())),
      )),
      _ => {
        let ref_ = render_expression(s, value, b, ref_index, type_slot)?;
        match ref_.ast_type {
          AScriptTypeVal::TokenVec => {
            // Merge the last and first token together
            // get the string value from the resulting span of the union
            Some(ref_.from(
              "(%%.first().unwrap() + %%.last().unwrap()).to_string()".to_string(),
              AScriptTypeVal::String(None),
            ))
          }
          AScriptTypeVal::String(..) => Some(ref_),
          _ => Some(ref_.from("%%.to_string()".to_string(), AScriptTypeVal::String(None))),
        }
      }
    },
    ASTNode::AST_BOOL(box AST_BOOL { value, initializer, .. }) => match initializer {
      ASTNode::NONE => Some(Ref::new(
        bump_ref_index(ref_index),
        type_slot,
        format!("{}", value),
        AScriptTypeVal::Bool(Some(*value)),
      )),
      ast => match render_expression(s, ast, body, ref_index, type_slot) {
        Some(_) => Some(Ref::new(
          bump_ref_index(ref_index),
          type_slot,
          "true".to_string(),
          AScriptTypeVal::Bool(Some(true)),
        )),
        None => Some(Ref::new(
          bump_ref_index(ref_index),
          type_slot,
          "false".to_string(),
          AScriptTypeVal::Bool(Some(false)),
        )),
      },
    },
    ASTNode::AST_U64(box AST_U64 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValU64>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_U32(box AST_U32 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValU32>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_U16(box AST_U16 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValU16>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_U8(box AST_U8 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValU8>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_I64(box AST_I64 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValI64>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_I32(box AST_I32 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValI32>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_I16(box AST_I16 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValI16>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_I8(box AST_I8 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValI8>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_F32(box AST_F32 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValF32>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_F64(box AST_F64 { initializer, .. }) => {
      convert_numeric::<AScriptTypeValF64>(initializer, b, s, ref_index, type_slot)
    }
    ASTNode::AST_NUMBER(..) => None,
    ASTNode::AST_Member(..) => None,
    ASTNode::AST_NamedReference(box AST_NamedReference { value, .. }) => {
      match get_named_body_ref(body, value) {
        Some((index, sym_id)) => render_body_symbol(sym_id, ast, index, type_slot),
        None => None,
      }
    }
    ASTNode::AST_IndexReference(box AST_IndexReference { value, .. }) => {
      match get_indexed_body_ref(body, value) {
        Some((index, sym_id)) => render_body_symbol(sym_id, ast, index, type_slot),
        None => None,
      }
    }
    _ => None,
  }
}

fn node_to_struct(ref_: Ref, ast: &AScriptStore) -> Ref {
  use AScriptTypeVal::*;
  match ref_.ast_type.clone() {
    Struct(struct_type) => {
      let struct_name = ast.structs.get(&struct_type).unwrap().type_name.clone();
      ref_.from(
        format!(
          "if let {}::{}(obj) = %%
      {{ obj }}
      else {{panic!(\"invalid node\")}}",
          ast.name, struct_name
        ),
        Struct(struct_type),
      )
    }
    GenericStruct(struct_types) if struct_types.len() == 1 => {
      let struct_type = struct_types.first().unwrap();
      let struct_name = ast.structs.get(&(struct_type.into())).unwrap().type_name.clone();
      ref_.from(
        format!(
          "if let {}::{}(obj) = %%
      {{ obj }}
      else {{panic!(\"invalid node\")}}",
          ast.name, struct_name
        ),
        Struct(struct_type.into()),
      )
    }
    _ => ref_,
  }
}

fn render_body_symbol(
  sym: &RuleSymbol,
  ast: &AScriptStore,
  i: usize,
  type_slot: usize,
) -> Option<Ref> {
  use AScriptTypeVal::*;
  let mut ref_ = match &sym.sym_id {
    SymbolID::Production(prod_id, ..) => {
      let types = get_production_types(ast, prod_id);
      if types.len() == 1 {
        let _type = types.first().unwrap().clone();

        if let Some(init_string) = match _type {
          F64(..) => Some(format!("i{0}.to_f64()", i)),
          F32(..) => Some(format!("i{0}.to_f32()", i)),
          U64(..) => Some(format!("i{0}.to_u64()", i)),
          I64(..) => Some(format!("i{0}.to_i64()", i)),
          U32(..) => Some(format!("i{0}.to_u32()", i)),
          I32(..) => Some(format!("i{0}.to_i32()", i)),
          U16(..) => Some(format!("i{0}.to_u16()", i)),
          I16(..) => Some(format!("i{0}.to_i16()", i)),
          U8(..) => Some(format!("i{0}.to_u8()", i)),
          I8(..) => Some(format!("i{0}.to_i8()", i)),
          String(..) => Some(format!("i{0}.to_string()", i)),
          Token => Some(format!("i{0}.to_token()", i)),
          GenericStructVec(..) => Some(format!("i{0}.into_nodes()", i)),
          StringVec => Some(format!("i{0}.into_strings()", i)),
          TokenVec => Some(format!("i{0}.into_tokens()", i)),
          F64Vec => Some(format!("i{0}.into_f64_vec()", i)),
          F32Vec => Some(format!("i{0}.into_f32_vec()", i)),
          U64Vec => Some(format!("i{0}.into_u64_vec()", i)),
          I64Vec => Some(format!("i{0}.into_i64_vec()", i)),
          U32Vec => Some(format!("i{0}.into_u32_vec()", i)),
          I32Vec => Some(format!("i{0}.into_i32_vec()", i)),
          U16Vec => Some(format!("i{0}.into_u16_vec()", i)),
          I16Vec => Some(format!("i{0}.into_i16_vec()", i)),
          U8Vec => Some(format!("i{0}.into_u8_vec()", i)),
          I8Vec => Some(format!("i{0}.into_i8_vec()", i)),
          GenericStruct(..) => Some(format!("i{0}.clone().into_node().unwrap()", i)),
          Struct(..) => Some(format!("i{0}.clone().into_node().unwrap()", i)),
          _ => None,
        } {
          Ref::new(i, type_slot, init_string, _type)
        } else {
          Ref::new(i, type_slot, "".to_string(), match _type.to_owned() {
            GenericVec(types) => get_specified_vector_from_generic_vec_values(
              &types.unwrap().iter().map(|t| t.into()).collect(),
            ),
            _type => _type,
          })
        }
      } else if production_types_are_structs(&types) {
        Ref::new(
          i,
          type_slot,
          format!("i{0}.clone().into_node().unwrap()", i),
          GenericStruct(extract_struct_types(&types)),
        )
      } else {
        Ref::new(i, type_slot, format!("i{0}.to_token()", i), Token)
      }
    }
    _ => Ref::new(i, type_slot, format!("i{0}.to_token()", i), Token),
  };

  ref_.add_body_index(i);

  Some(ref_)
}

fn extract_struct_types(types: &BTreeSet<AScriptTypeVal>) -> BTreeSet<TaggedType> {
  types
    .iter()
    .filter_map(|t| match t {
      AScriptTypeVal::Struct(id) => Some(TaggedType { type_: t.clone(), ..Default::default() }),
      _ => None,
    })
    .collect::<BTreeSet<_>>()
}

fn convert_numeric<T: AScriptNumericType>(
  init: &ASTNode,
  body: &Rule,
  ast: &AScriptStore,
  ref_index: &mut usize,
  type_slot: usize,
) -> Option<Ref> {
  let rust_type = T::prim_type_name();
  let tok_conversion_fn = T::to_fn_name();

  match init {
    ASTNode::AST_NUMBER(box AST_NUMBER { value, .. }) => Some(Ref::new(
      bump_ref_index(ref_index),
      type_slot,
      format!("{}{}", T::string_from_f64(*value), rust_type,),
      T::from_f64(*value),
    )),
    _ => {
      let ref_ = render_expression(ast, init, body, ref_index, type_slot)?;

      match ref_.ast_type {
        AScriptTypeVal::F64(..)
        | AScriptTypeVal::F32(..)
        | AScriptTypeVal::Bool(..)
        | AScriptTypeVal::I8(..)
        | AScriptTypeVal::I16(..)
        | AScriptTypeVal::I32(..)
        | AScriptTypeVal::I64(..)
        | AScriptTypeVal::U8(..)
        | AScriptTypeVal::U16(..)
        | AScriptTypeVal::U32(..)
        | AScriptTypeVal::U64(..) => {
          Some(ref_.from(format!("%% as {};", rust_type), T::from_f64(0.0)))
        }
        _ => Some(ref_.from(format!("%%.{}();", tok_conversion_fn), T::from_f64(0.0))),
      }
    }
  }
}

fn bump_ref_index(ref_index: &mut usize) -> usize {
  *ref_index += 1;
  *ref_index
}

pub fn ascript_type_to_string(ast_type: &AScriptTypeVal, ast: &AScriptStore) -> String {
  use AScriptTypeVal::*;
  match ast_type {
    GenericVec(types) => format!("Vec<{:?}>", types),
    Struct(id) => format!("Box<{}>", ast.structs.get(id).unwrap().type_name),
    String(..) => "String".to_string(),
    Bool(..) => "bool".to_string(),
    F64(..) => "f64".to_string(),
    F32(..) => "f32".to_string(),
    I64(..) => "i64".to_string(),
    I32(..) => "i32".to_string(),
    I16(..) => "i16".to_string(),
    I8(..) => "i8".to_string(),
    U64(..) => "u64".to_string(),
    U32(..) => "u32".to_string(),
    U16(..) => "u16".to_string(),
    U8(..) => "u8".to_string(),
    Undefined => "Undefined".to_string(),
    Token => "Token".to_string(),
    UnresolvedProduction(prod_id) => {
      let production_types = get_production_types(ast, prod_id);
      if production_types.len() > 1 {
        if production_types_are_structs(&production_types) {
          ast.name.clone()
        } else {
          "HCObj::None".to_string()
        }
      } else {
        ascript_type_to_string(&production_types.first().unwrap(), ast)
      }
    }
    F64Vec => "Vec<f64>".to_string(),
    F32Vec => "Vec<f32>".to_string(),
    I64Vec => "Vec<i64>".to_string(),
    I32Vec => "Vec<i32>".to_string(),
    I16Vec => "Vec<i16>".to_string(),
    I8Vec => "Vec<i8>".to_string(),
    U64Vec => "Vec<u64>".to_string(),
    U32Vec => "Vec<u32>".to_string(),
    U16Vec => "Vec<u16>".to_string(),
    U8Vec => "Vec<u8>".to_string(),
    TokenVec => "Vec<Token>".to_string(),
    GenericStruct(struct_ids) => {
      if struct_ids.len() > 1 {
        ast.name.clone()
      } else {
        format!("Box<{}>", ast.structs.get(&struct_ids.first().unwrap().into()).unwrap().type_name)
      }
    }
    GenericStructVec(struct_ids) => {
      if struct_ids.len() > 1 {
        format!("Vec<{}>", ast.name)
      } else {
        format!(
          "Vec<Box<{}>>",
          ast.structs.get(&struct_ids.first().unwrap().into()).unwrap().type_name
        )
      }
    }
    _ => {
      panic!("Could not resolve compiled ascript type {:?}", ast_type)
    }
  }
}

fn get_default_value_(ast_type: &AScriptTypeVal, ast: &AScriptStore) -> String {
  use AScriptTypeVal::*;
  match ast_type {
    GenericVec(..) => "vec![]".to_string(),
    Struct(..) => "None".to_string(),
    GenericStruct(..) => "None".to_string(),
    String(..) => "String::new()".to_string(),
    Bool(..) => "false".to_string(),
    F64(..) => "0f64".to_string(),
    F32(..) => "0f32".to_string(),
    I64(..) => "0i64".to_string(),
    I32(..) => "0i32".to_string(),
    I16(..) => "0i16".to_string(),
    I8(..) => "0i8".to_string(),
    U64(..) => "0u64".to_string(),
    U32(..) => "0u32".to_string(),
    U16(..) => "0u16".to_string(),
    U8(..) => "0u8".to_string(),
    Undefined => "None".to_string(),
    Token => "Token::new()".to_string(),
    UnresolvedProduction(prod_id) => {
      let production_types = get_production_types(ast, prod_id);
      if production_types.len() > 1 {
        if production_types_are_structs(&production_types) {
          ast.name.clone()
        } else {
          "HCObj::None".to_string()
        }
      } else {
        get_default_value_(&production_types.first().unwrap(), ast)
      }
    }
    F64Vec | F32Vec | U64Vec | I64Vec | U32Vec | I32Vec | U16Vec | I16Vec | U8Vec | I8Vec
    | GenericStructVec(..) | StringVec | TokenVec => "Vec::new()".to_string(),
    _ => {
      panic!("Could not resolve compiled ascript type {:?}", ast_type)
    }
  }
}

fn get_default_value(prop_id: &AScriptPropId, ast: &AScriptStore) -> String {
  if let Some(prop) = ast.props.get(prop_id) {
    get_default_value_(&(&prop.type_val).into(), ast)
  } else {
    "None".to_string()
  }
}

#[derive(Clone)]
pub struct Ref {
  init_index: usize,
  type_slot: usize,
  body_indices: BTreeSet<usize>,
  init_expression: String,
  ast_type: AScriptTypeVal,
  predecessors: Option<Vec<Box<Ref>>>,
  post_init_statements: Option<Vec<String>>,
  is_mutable: bool,
}

impl Ref {
  pub fn new(
    init_index: usize,
    type_slot: usize,
    init_expression: String,
    ast_type: AScriptTypeVal,
  ) -> Self {
    Ref {
      init_index,
      type_slot,
      body_indices: BTreeSet::from_iter(vec![init_index]),
      init_expression,
      ast_type,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub fn token(init_index: usize, type_slot: usize) -> Self {
    Ref {
      init_index,
      type_slot,
      body_indices: BTreeSet::new(),
      init_expression: "tok".to_string(),
      ast_type: AScriptTypeVal::Token,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub fn get_type(&self) -> AScriptTypeVal {
    self.ast_type.clone()
  }

  pub fn from(self, init_expression: String, ast_type: AScriptTypeVal) -> Self {
    Ref {
      init_index: self.init_index,
      type_slot: self.type_slot,
      init_expression,
      ast_type,
      body_indices: BTreeSet::new(),
      predecessors: Some(vec![Box::new(self)]),
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub fn make_mutable(&mut self) -> &mut Self {
    self.is_mutable = true;
    self
  }

  pub fn add_body_index(&mut self, index: usize) {
    self.body_indices.insert(index);
  }

  pub fn get_ref_string(&self) -> String {
    format!("ref_{}_{}", self.init_index, self.type_slot)
  }

  pub fn get_indices(&self) -> BTreeSet<usize> {
    let mut indices = self.body_indices.clone();

    if let Some(predecessors) = &self.predecessors {
      for predecessor in predecessors {
        indices.append(&mut predecessor.get_indices())
      }
    }

    indices
  }

  pub fn add_post_init_expression(&mut self, string: String) -> &mut Self {
    self.post_init_statements.get_or_insert(vec![]).push(string);

    self
  }

  pub fn to_init_string(&self) -> String {
    let mut string = String::new();

    if let Some(predecessors) = &self.predecessors {
      for predecessor in predecessors {
        string = string + &predecessor.to_init_string()
      }
    }

    let ref_string = self.get_ref_string();

    string += &format!(
      "\nlet{}{} = {};",
      if self.is_mutable { " mut " } else { " " },
      ref_string,
      self.init_expression.replace("%%", &ref_string)
    );

    if let Some(statements) = &self.post_init_statements {
      string = string + "\n" + &statements.join(";\n").replace("%%", &ref_string) + ";";
    }

    string
  }

  pub fn add_predecessor(&mut self, predecessor: Ref) -> &mut Self {
    self.predecessors.get_or_insert(vec![]).push(Box::new(predecessor));

    self
  }

  pub fn add_predecessors(&mut self, predecessors: Vec<Ref>) -> &mut Self {
    let prev = self.predecessors.get_or_insert(vec![]);

    for predecessor in predecessors {
      prev.push(Box::new(predecessor))
    }

    self
  }
}
