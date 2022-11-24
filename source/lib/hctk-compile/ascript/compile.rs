use std::collections::btree_map;
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::vec;

use hctk_core::grammar::data::ast::ASTNode;
use hctk_core::grammar::data::ast::ASTNodeTraits;
use hctk_core::grammar::data::ast::AST_Add;
use hctk_core::grammar::data::ast::AST_IndexReference;
use hctk_core::grammar::data::ast::AST_NamedReference;
use hctk_core::grammar::data::ast::AST_Struct;
use hctk_core::grammar::data::ast::AST_Vector;
use hctk_core::grammar::data::ast::Ascript as AST_AScript;
use hctk_core::types::*;

use crate::ascript::errors::ErrIncompatibleProductionScalerTypes;
use crate::ascript::errors::ErrPropRedefinition;
use crate::ascript::types::*;

use super::errors::ErrIncompatibleProductionVectorTypes;
use super::errors::ErrUnionOfScalarsAndVectors;
use super::types::AScriptStore;

pub fn compile_ascript_store(ast: &mut AScriptStore) -> Vec<HCError> {
  let mut e = vec![];
  let g = ast.g.clone();

  // Separate all bodies into a list of  of tuple of body id's and
  // Ascript reference nodes.

  let normal_parse_bodies: Vec<(BodyId, Option<&AST_AScript>)> = ast
    .g
    .bodies
    .iter()
    .filter_map(|(id, body)| match g.clone().parse_productions.contains(&body.prod_id) {
      true => {
        for function in &body.reduce_fn_ids {
          if let ReduceFunctionType::Ascript(ascript) = g.reduce_functions.get(function).unwrap() {
            return Some((*id, Some(ascript)));
          }
        }
        Some((*id, None))
      }
      false => None,
    })
    .collect::<Vec<_>>();

  // For reduce function in each body divide into those that resolve
  // into atomic types and those that don't (those that resolve into
  // productions). Add the types of the atomic functions to the
  // production types. Add any structs encountered into a separate
  // table, again adding these atomic struct types to the production
  // types.

  let mut struct_bodies: Vec<(BodyId, &AST_AScript)> = vec![];

  for (body_id, ascript_option_fn) in normal_parse_bodies {
    if let Some(body) = g.clone().bodies.get(&body_id) {
      if let Some(ascript_fn) = &ascript_option_fn {
        match &ascript_fn.ast {
          ASTNode::AST_Struct(box ast_struct) => {
            let (id, mut sub_errors) = compile_struct_type(ast, ast_struct, body);
            struct_bodies.push((body_id, ascript_fn));
            add_production_type(ast, &body, TaggedType {
              type_:        AScriptTypeVal::Struct(id),
              tag:          body_id,
              symbol_index: 0,
            });
            e.append(&mut sub_errors);
          }
          ASTNode::AST_Statements(ast_stmts) => {
            let (sub_types, mut sub_errors) =
              compile_expression_type(ast, ast_stmts.statements.last().unwrap(), body);

            for sub_type in sub_types {
              add_production_type(ast, &body, sub_type);
            }

            e.append(&mut sub_errors);
          }
          _ => {}
        }
      } else {
        match Item::from(body).to_last_sym().get_symbol(&g) {
          SymbolID::Production(id, ..) => add_production_type(ast, &body, TaggedType {
            type_:        AScriptTypeVal::UnresolvedProduction(id),
            tag:          body_id,
            symbol_index: (body.len - 1) as u32,
          }),
          _ => add_production_type(ast, &body, TaggedType {
            type_:        AScriptTypeVal::Token,
            tag:          body_id,
            symbol_index: (body.len - 1) as u32,
          }),
        };
      }
    }
  }

  resolve_production_reduce_types(ast, &mut e);

  if e.have_critical() {
    return e;
  }

  for key in Vec::from_iter(ast.prod_types.keys().cloned()) {
    let _types = ast.prod_types.get(&key).unwrap().to_owned();

    if _types.len() == 1 {
      let (_type, tokens) = (_types.iter().next().unwrap());
      match _type.into() {
        AScriptTypeVal::GenericVec(Some(_types)) => {
          let resolved_vector_type = get_specified_vector_from_generic_vec_values(
            &_types.iter().map(|v| v.into()).collect(),
          );

          if resolved_vector_type.is_undefined() {
            panic!("Need to report invalid Vec type");
            // ///
            // e.push(HCError::grammar_err_multi_location {
            // message: format!(
            // "Invalid combination of types within vector {}",
            // _type.debug_string(Some(&g))
            // ),
            //
            // locations: _types
            // .iter()
            // .flat_map(|type_| {
            // body_id.iter().map(|tok| HCError::grammar_err {
            // inline_message: format!("produces [{}]", type_.debug_string(Some(&g))),
            // loc: Token::empty(),
            // message: String::default(),
            // path: g.id.path.clone(),
            // })
            // })
            // .collect(),
            // })
            //
          } else {
            ast.prod_types.insert(
              key,
              HashMap::from_iter(vec![(
                TaggedType { type_: resolved_vector_type, ..Default::default() },
                tokens.to_owned(),
              )]),
            );
          }
        }
        _ => {}
      }
    }
  }

  // Ensure all non-scanner productions have been added to the ascript data.
  assert_eq!(ast.prod_types.len(), g.parse_productions.len());

  // We now have the base types for all productions. We can now do a
  // thorough analysis of struct types and production return
  // functions.

  // We'll now finish parsing struct data and declaring or resolving
  // type conflicts

  for struct_id in ast.structs.keys().cloned().collect::<Vec<_>>() {
    let bodies = ast.structs.get(&struct_id).unwrap().body_ids.clone();
    for body_id in bodies {
      let body = g.get_body(&body_id).unwrap();
      for function in &body.reduce_fn_ids {
        if let ReduceFunctionType::Ascript(ascript) = g.reduce_functions.get(function).unwrap() {
          if let ASTNode::AST_Struct(box ast_struct) = &ascript.ast {
            e.append(&mut compile_struct_props(ast, &struct_id, ast_struct, &body).1);
          }
        }
      }
    }
  }

  resolve_prop_types(ast);

  e
}

fn resolve_prop_types(ascript: &mut AScriptStore) {
  // Ensure each property entry has a resolved data type.
  for prop_key in ascript.props.keys().cloned().collect::<Vec<_>>() {
    let type_val = ascript.props.get(&prop_key).unwrap().type_val.clone();

    ascript.props.get_mut(&prop_key).unwrap().type_val = TaggedType {
      type_: get_resolved_type(ascript, &type_val.into()).0,
      ..Default::default()
    };
  }
}

fn resolve_production_reduce_types(ast: &mut AScriptStore, e: &mut Vec<HCError>) {
  let mut pending_prods = VecDeque::from_iter(ast.g.parse_productions.iter().cloned().rev());

  while let Some(prod_id) = pending_prods.pop_front() {
    if !ast.prod_types.contains_key(&prod_id) {
      unreachable!("All production should be accounted for");
      continue;
    }

    let mut resubmit = false;
    let mut new_map = HashMap::new();
    let mut vector_types = ast.prod_types.remove(&prod_id).unwrap().into_iter().collect::<Vec<_>>();
    let scalar_types = vector_types.drain_filter(|(a, _)| !a.type_.is_vec()).collect::<Vec<_>>();

    if !scalar_types.is_empty() {
      use AScriptTypeVal::*;
      let (mut prime, mut prime_body_ids) = (TaggedType::default(), BTreeSet::new());

      let mut insert_production_types = |ast: &mut AScriptStore, foreign_prod_id: ProductionId| {
        if foreign_prod_id != prod_id {
          match ast.prod_types.get(&foreign_prod_id) {
            Some(types_) if !types_.is_empty() => {
              new_map.extend(types_.clone());
            }
            Some(_) => {
              panic!(
                "Production [{}] does not produce any types",
                ast.g.get_production_plain_name(&foreign_prod_id)
              )
            }
            _ => {
              panic!(
                "Production [{}] does not exist in production types lookup",
                ast.g.get_production_plain_name(&foreign_prod_id)
              )
            }
          }

          true
        } else {
          false
        }
      };

      for (other, mut body_ids) in scalar_types {
        prime = match (prime.type_.clone(), other.type_.clone()) {
          (Struct(typeA), Struct(typeB)) if typeA != typeB => {
            prime_body_ids.append(&mut body_ids);
            TaggedType {
              type_: GenericStruct(BTreeSet::from_iter(vec![prime, other])),
              ..Default::default()
            }
          }
          (Struct(typeB), GenericStruct(mut btree_set))
          | (GenericStruct(mut btree_set), Struct(typeB)) => {
            btree_set.insert(prime);
            prime_body_ids.append(&mut body_ids);
            TaggedType {
              type_: GenericStruct(BTreeSet::from_iter(btree_set)),
              ..Default::default()
            }
          }
          (type_, UnresolvedProduction(foreign_prod_id)) => {
            resubmit = resubmit.max(insert_production_types(ast, foreign_prod_id));
            prime
          }
          (UnresolvedProduction(foreign_prod_id), _) => {
            resubmit = resubmit.max(insert_production_types(ast, foreign_prod_id));
            other
          }
          (Undefined, _) => {
            prime_body_ids.append(&mut body_ids);
            other
          }
          (a, b) if a.is_same_type(&b) => {
            prime_body_ids.append(&mut body_ids);
            prime
          }
          (a, b) => {
            e.push(ErrIncompatibleProductionScalerTypes::new(
              prod_id,
              ast.g.clone(),
              (a.clone(), prime_body_ids.iter().cloned().collect()),
              (b, body_ids.iter().cloned().collect()),
              ast.get_type_names(),
            ));
            prime
          }
        }
      }

      if !prime.type_.is_undefined() {
        new_map.insert(prime, prime_body_ids);
      }
    }

    if !vector_types.is_empty() {
      // Note: (Invariant) All Vecs are GenericVec at this point.
      use AScriptTypeVal::*;
      let (mut prime, mut prime_body_ids) = (TaggedType::default(), BTreeSet::new());
      let mut vector_types = VecDeque::from_iter(vector_types);

      let mut fun_name = |mut known_types: BTreeSet<TaggedType>,
                          vector_types: &mut VecDeque<(TaggedType, BTreeSet<BodyId>)>|
       -> BTreeSet<TaggedType> {
        vector_types.extend(
          known_types
            .drain_filter(|t| matches!(t.into(), GenericVec(..)))
            .map(|t| (t.into(), BTreeSet::new()))
            .collect::<VecDeque<_>>(),
        );

        for production in known_types.drain_filter(|t| matches!(t.into(), UnresolvedProduction(..)))
        {
          if let UnresolvedProduction(foreign_prod_id) = production.into() {
            if foreign_prod_id != prod_id {
              let other_production_types = ast.prod_types.get(&foreign_prod_id).unwrap();

              new_map.insert(
                TaggedType {
                  type_: AScriptTypeVal::GenericVec(Some(
                    other_production_types.keys().cloned().collect(),
                  )),
                  ..Default::default()
                },
                other_production_types.values().flatten().cloned().collect(),
              );

              resubmit = true;
            }
          }
        }
        known_types
      };

      while let Some((other, mut body_ids)) = vector_types.pop_front() {
        prime = match (prime.type_.clone(), other.type_.clone()) {
          (GenericVec(Some(vecA)), GenericVec(Some(vecB))) => {
            // Check for compatibility, and extract productions from vectors
            let mut known_types = fun_name(vecA, &mut vector_types);

            known_types.extend(fun_name(vecB, &mut vector_types));

            prime_body_ids.append(&mut body_ids);
            TaggedType { type_: GenericVec(Some(known_types)), ..Default::default() }
          }
          (GenericVec(Some(vecA)), GenericVec(None)) => {
            prime_body_ids.append(&mut body_ids);
            TaggedType { type_: GenericVec(Some(vecA)), ..Default::default() }
          }
          (GenericVec(None), GenericVec(Some(vecB))) => {
            prime_body_ids.append(&mut body_ids);
            TaggedType { type_: GenericVec(Some(vecB)), ..Default::default() }
          }
          (GenericVec(None), GenericVec(None)) => {
            prime_body_ids.append(&mut body_ids);
            TaggedType { type_: GenericVec(None), ..Default::default() }
          }
          (Undefined, _) => {
            prime_body_ids.append(&mut body_ids);
            other
          }
          _ => unreachable!("Only GenericVector types Should be defined at this point."),
        }
      }
      if !prime.type_.is_undefined() {
        new_map.insert(prime, prime_body_ids);
      }
    }

    ast.prod_types.insert(prod_id, new_map);

    if ast.g.get_production_plain_name(&prod_id) == "A" {
      println!(
        "Resolving {} \n{:#?}",
        ast.g.get_production_plain_name(&prod_id),
        ast
          .prod_types
          .get(&prod_id)
          .unwrap()
          .iter()
          .map(|(t, _)| { t.type_.debug_string(Some(&ast.g)) })
          .collect::<Vec<_>>()
      );
    }

    if resubmit {
      println!("Repeating {} ", ast.g.get_production_plain_name(&prod_id));
      pending_prods.push_back(prod_id);
    }
  }

  // Do final check for incompatible types
  for prod_id in ast.prod_types.keys().cloned().collect::<Vec<_>>() {
    let mut vector_types = ast.prod_types.get(&prod_id).unwrap().iter().collect::<Vec<_>>();
    let scalar_types = vector_types.drain_filter(|(a, ..)| !a.type_.is_vec()).collect::<Vec<_>>();

    debug_assert!(
      !scalar_types
        .iter()
        .any(|(a, _)| matches!((*a).into(), AScriptTypeVal::UnresolvedProduction(_))),
      "Production [{}] has not been fully resolved \n{:#?}",
      ast.g.get_production_plain_name(&prod_id),
      ast
        .prod_types
        .get(&prod_id)
        .unwrap()
        .iter()
        .map(|(t, _)| { t.debug_string(Some(&ast.g)) })
        .collect::<Vec<_>>()
    );

    if !vector_types.is_empty() && !scalar_types.is_empty() {
      e.push(ErrUnionOfScalarsAndVectors::new(
        ast.g.clone(),
        prod_id,
        scalar_types
          .iter()
          .flat_map(|(type_, bodies)| {
            bodies.iter().map(|b| ((*type_).into(), *b)).collect::<Vec<_>>()
          })
          .collect(),
        vector_types
          .iter()
          .flat_map(|(type_, bodies)| {
            bodies.iter().map(|b| ((*type_).into(), *b)).collect::<Vec<_>>()
          })
          .collect(),
        ast.get_type_names(),
      ));
    }
  }
}

/// Retrieve the resolved type of the base type. For most ascript types
/// this returns a clone of the `base_type`. For vectors and unresolved
/// productions types, this attempts to replace such types with resolved
/// versions
pub fn get_resolved_type(
  ascript: &AScriptStore,
  base_type: &AScriptTypeVal,
) -> (AScriptTypeVal, bool) {
  match base_type {
    AScriptTypeVal::UnresolvedProduction(production_id) => {
      if let Some(types) = ascript
        .prod_types
        .get(production_id)
        .and_then(|t| Some(t.keys().cloned().collect::<Vec<_>>()))
      {
        if types.len() == 1 {
          ((&types[0]).into(), false)
        } else if types.iter().all(|t| {
          matches!(t.into(), AScriptTypeVal::Struct(..) | AScriptTypeVal::GenericStruct(..))
        }) {
          let nodes = types
            .iter()
            .flat_map(|t| match t.into() {
              AScriptTypeVal::Struct(id) => vec![t.clone()],
              AScriptTypeVal::GenericStruct(ids) => ids.iter().cloned().collect(),
              _ => vec![],
            })
            .collect::<BTreeSet<_>>();

          (AScriptTypeVal::GenericStruct(nodes), false)
        } else {
          (AScriptTypeVal::Any, false)
        }
      } else {
        (AScriptTypeVal::Undefined, false)
      }
    }

    AScriptTypeVal::GenericVec(Some(vector_sub_types)) => {
      let contents = BTreeSet::from_iter(get_resolved_vec_contents(ascript, base_type));
      // Flatten the subtypes into one array and get the resulting type from that
      (get_specified_vector_from_generic_vec_values(&contents), false)
    }

    _ => (base_type.clone(), false),
  }
}

pub fn get_resolved_vec_contents(
  ast: &AScriptStore,
  base_type: &AScriptTypeVal,
) -> Vec<AScriptTypeVal> {
  use AScriptTypeVal::*;

  match base_type {
    F64Vec => vec![F64(None)],
    F32Vec => vec![F32(None)],
    I64Vec => vec![I64(None)],
    I32Vec => vec![I32(None)],
    I16Vec => vec![I16(None)],
    I8Vec => vec![I8(None)],
    U64Vec => vec![U64(None)],
    U32Vec => vec![U32(None)],
    U16Vec => vec![U16(None)],
    U8Vec => vec![U8(None)],
    GenericStructVec(types) => types.iter().map(|t| t.into()).collect(),
    GenericVec(Some(types)) => {
      types.iter().flat_map(|t| get_resolved_vec_contents(ast, &t.into())).collect()
    }
    TokenVec => vec![Token],
    StringVec => vec![String(None)],
    UnresolvedProduction(_) => get_resolved_vec_contents(ast, &get_resolved_type(ast, base_type).0),
    none_vec_type => {
      vec![none_vec_type.clone()]
    }
  }
}

pub fn add_production_type(ast: &mut AScriptStore, body: &Body, new_return_type: TaggedType) {
  let table = ast.prod_types.entry(body.prod_id).or_insert_with(HashMap::new);

  match table.entry(new_return_type.clone()) {
    Entry::Occupied(mut entry) => {
      entry.get_mut().insert(new_return_type.into());
    }
    Entry::Vacant(entry) => {
      entry.insert(BTreeSet::from_iter(vec![new_return_type.into()]));
    }
  }
}

pub fn compile_expression_type(
  ast: &mut AScriptStore,
  ast_expression: &ASTNode,
  body: &Body,
) -> (Vec<TaggedType>, Vec<HCError>) {
  use AScriptTypeVal::*;
  let mut errors = vec![];

  let types = match ast_expression {
    ASTNode::AST_Struct(ast_struct) => {
      let (struct_type, mut error) = compile_struct_type(ast, ast_struct, body);

      errors.append(&mut error);

      vec![TaggedType {
        symbol_index: 9999,
        tag:          body.id,
        type_:        Struct(struct_type),
      }]
    }
    ASTNode::AST_Token(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        Token,
    }],
    ASTNode::AST_Add(box AST_Add { left, .. }) => {
      let (sub_types, mut sub_errors) = compile_expression_type(ast, left, body);
      errors.append(&mut sub_errors);
      sub_types
    }
    ASTNode::AST_Vector(box AST_Vector { initializer, .. }) => {
      let mut types = BTreeSet::new();

      for node in initializer {
        let (sub_types, mut sub_errors) = compile_expression_type(ast, node, body);

        for sub_type in sub_types {
          match (&sub_type).into() {
            GenericVec(sub_types) => match sub_types {
              Some(mut sub_type) => {
                types.append(&mut sub_type);
              }
              None => {}
            },
            // We ignore undefined types, since we can represent such types with an empty vector.
            Undefined => {}
            _ => {
              types.insert(sub_type);
            }
          }
        }

        errors.append(&mut sub_errors);
      }
      if types.is_empty() {
        vec![TaggedType {
          symbol_index: body.syms.len() as u32,
          tag:          body.id,
          type_:        GenericVec(None),
        }]
      } else {
        vec![TaggedType {
          symbol_index: body.syms.len() as u32,
          tag:          body.id,
          type_:        GenericVec(Some(types)),
        }]
      }
    }
    ASTNode::AST_STRING(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        String(None),
    }],
    ASTNode::AST_BOOL(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        Bool(None),
    }],
    ASTNode::AST_U8(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        U8(None),
    }],
    ASTNode::AST_U16(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        U16(None),
    }],
    ASTNode::AST_U32(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        U32(None),
    }],
    ASTNode::AST_U64(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        U64(None),
    }],
    ASTNode::AST_I8(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        I8(None),
    }],
    ASTNode::AST_I16(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        I16(None),
    }],
    ASTNode::AST_I32(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        I32(None),
    }],
    ASTNode::AST_I64(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        I64(None),
    }],
    ASTNode::AST_F32(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        F32(None),
    }],
    ASTNode::AST_F64(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        F64(None),
    }],
    ASTNode::AST_NUMBER(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        F64(None),
    }],
    ASTNode::AST_Member(..) => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        Undefined,
    }],
    ASTNode::AST_NamedReference(box AST_NamedReference { value, .. }) => {
      match get_named_body_ref(body, value) {
        Some((_, sym_ref)) => match sym_ref.sym_id {
          SymbolID::Production(id, ..) => match ast.prod_types.get(&id) {
            Some(types) => types
              .keys()
              .map(|t| TaggedType {
                symbol_index: sym_ref.original_index,
                tag:          body.id,
                type_:        UnresolvedProduction(id),
              })
              .collect(),
            None => vec![TaggedType {
              symbol_index: sym_ref.original_index,
              tag:          body.id,
              type_:        UnresolvedProduction(id),
            }],
          },
          _ => vec![TaggedType {
            symbol_index: sym_ref.original_index,
            tag:          body.id,
            type_:        Token,
          }],
        },
        None => vec![TaggedType {
          symbol_index: body.syms.len() as u32,
          tag:          body.id,
          type_:        Undefined,
        }],
      }
    }
    ASTNode::AST_IndexReference(box AST_IndexReference { value, .. }) => {
      match get_indexed_body_ref(body, value) {
        Some((_, sym_ref)) => match sym_ref.sym_id {
          SymbolID::Production(id, ..) => match ast.prod_types.get(&id) {
            Some(types) => types
              .keys()
              .map(|t| TaggedType {
                symbol_index: sym_ref.original_index,
                tag:          body.id,
                type_:        UnresolvedProduction(id),
              })
              .collect(),
            None => vec![TaggedType {
              symbol_index: sym_ref.original_index,
              tag:          body.id,
              type_:        UnresolvedProduction(id),
            }],
          },
          _ => vec![TaggedType {
            symbol_index: sym_ref.original_index,
            tag:          body.id,
            type_:        Token,
          }],
        },
        None => vec![TaggedType {
          symbol_index: body.syms.len() as u32,
          tag:          body.id,
          type_:        Undefined,
        }],
      }
    }
    _ => vec![TaggedType {
      symbol_index: body.syms.len() as u32,
      tag:          body.id,
      type_:        Undefined,
    }],
  };

  (types, errors)
}

/// Compiles a struct type from a production body and
/// ascript struct node.
pub fn compile_struct_type(
  ast: &mut AScriptStore,
  ast_struct: &AST_Struct,
  body: &Body,
) -> (AScriptStructId, Vec<HCError>) {
  let mut errors = vec![];
  let mut types = vec![];
  let mut classes = vec![];
  let mut include_token = false;

  for prop in ast_struct.props.iter() {
    match prop {
      ASTNode::AST_TypeId(id) => types.push(id),
      ASTNode::AST_ClassId(id) => classes.push(id),
      ASTNode::AST_Token(..) => include_token = true,
      // Precompile property to ensure we gather all sub-structs;
      // We don't care about the actual value at this point.
      ASTNode::AST_Property(box prop) => {
        compile_expression_type(ast, &prop.value, body);
      }
      _ => {}
    }
  }

  // Use the last type as the official type name of the struct.
  let type_name =
    if let Some(node) = types.last() { node.value.clone() } else { "unknown".to_string() }[2..]
      .to_string();

  // Validate struct type is singular
  match types.len() {
    2.. => {
      errors.push(HCError::grammar_err_multi_location {
        message:   "Struct Type Redefined".to_string(),
        locations: types
          .iter()
          .enumerate()
          .map(|(i, node)| {
            if i == 0 {
              HCError::grammar_err {
                message: "".to_string(),
                loc: node.Token(),
                inline_message: "First Defined Here".to_string(),
                path: Default::default(),
              }
            } else {
              HCError::grammar_err {
                message: "".to_string(),
                loc: node.Token(),
                inline_message: "Redefined Here".to_string(),
                path: Default::default(),
              }
            }
          })
          .collect::<Vec<_>>(),
      });
    }
    0 => errors.push(HCError::grammar_err {
      message: "Struct defined without a type name".to_string(),
      loc: ast_struct.Token(),
      inline_message: "".to_string(),
      path: Default::default(),
    }),
    _ => {}
  }

  let id = AScriptStructId::new(&type_name);
  match ast.structs.entry(id.clone()) {
    btree_map::Entry::Occupied(mut entry) => {
      let struct_ = entry.get_mut();
      struct_.body_ids.insert(body.id);
      struct_.definition_locations.push(ast_struct.Token());
      struct_.include_token = struct_.include_token.max(include_token);
    }
    btree_map::Entry::Vacant(entry) => {
      entry.insert(AScriptStruct {
        id,
        type_name,
        body_ids: BTreeSet::from_iter(vec![body.id]),
        definition_locations: vec![ast_struct.Token()],
        prop_ids: BTreeSet::new(),
        include_token,
      });
    }
  }

  (id.clone(), errors)
}

/// Completes the compilation of struct type by defining the properties
/// of a struct.
pub fn compile_struct_props(
  ast: &mut AScriptStore,
  id: &AScriptStructId,
  ast_struct: &AST_Struct,
  body: &Body,
) -> (AScriptTypeVal, Vec<HCError>) {
  let mut errors = vec![];

  // Check to see if this struct is already defined. If so, we'll
  // append new properties to it. otherwise we create a new
  // struct entry and add props.

  let mut prop_ids = BTreeSet::new();
  let mut include_token = false;

  for prop in &ast_struct.props {
    match prop {
      ASTNode::AST_Token(..) => include_token = true,
      ASTNode::AST_Property(box prop) => {
        let name = &prop.id;
        let prop_id = AScriptPropId::new(id.clone(), name);

        prop_ids.insert(prop_id.clone());

        for prop_type in compile_expression_type(ast, &prop.value, body).0 {
          match ast.props.get_mut(&prop_id) {
            Some(existing) => {
              use AScriptTypeVal::*;
              match ((&existing.type_val).into(), (&prop_type).into()) {
                (Struct(typeA), Struct(typeB), ..) if typeA != typeB => {
                  existing.type_val = TaggedType {
                    type_: GenericStruct(BTreeSet::from_iter(vec![
                      existing.type_val.clone(),
                      prop_type,
                    ])),
                    ..Default::default()
                  };
                  existing.body_ids.insert(body.id);
                }
                (GenericStruct(mut btree_set), Struct(typeB), ..) => {
                  btree_set.insert(prop_type);
                  existing.type_val =
                    TaggedType { type_: GenericStruct(btree_set), ..Default::default() };
                  existing.body_ids.insert(body.id);
                }
                (Struct(typeA), GenericStruct(mut btree_set), ..) => {
                  btree_set.insert(existing.type_val.clone());
                  existing.type_val =
                    TaggedType { type_: GenericStruct(btree_set), ..Default::default() };
                  existing.body_ids.insert(body.id);
                }
                (GenericStructVec(mut vecA), GenericStructVec(mut vecB), ..) => {
                  vecA.append(&mut vecB);
                  existing.type_val =
                    TaggedType { type_: GenericStructVec(vecA), ..Default::default() };
                  existing.body_ids.insert(body.id);
                }
                (Undefined, _) => {
                  existing.body_ids.insert(body.id);
                  existing.type_val = prop_type.to_owned();
                  existing.location = prop.value.Token();
                  existing.grammar_ref = body.grammar_ref.clone();
                  existing.optional = true;
                }
                (_, Undefined) => {
                  existing.body_ids.insert(body.id);
                  existing.optional = true;
                }
                (a, b) if a.is_same_type(&b) => {
                  existing.body_ids.insert(body.id);
                }
                _ => {
                  errors.push(ErrPropRedefinition::new(
                    ast.structs.get(id).unwrap().type_name.clone(),
                    name.clone(),
                    existing.clone(),
                    AScriptProp {
                      type_val: prop_type.into(),
                      location: prop.value.Token(),
                      grammar_ref: body.grammar_ref.clone(),
                      ..Default::default()
                    },
                  ));
                }
              }
            }
            _ => {
              ast.props.insert(prop_id.clone(), AScriptProp {
                type_val: prop_type.into(),
                body_ids: BTreeSet::from_iter(vec![body.id]),
                location: prop.value.Token(),
                grammar_ref: body.grammar_ref.clone(),
                ..Default::default()
              });
            }
          }
        }
      }
      _ => {}
    }
  }

  match ast.structs.entry(id.clone()) {
    btree_map::Entry::Occupied(mut entry) => {
      let struct_ = entry.get_mut();
      struct_.body_ids.insert(body.id);
      struct_.definition_locations.push(ast_struct.Token());
      struct_.prop_ids.append(&mut prop_ids);
      struct_.include_token = include_token || struct_.include_token;

      for prop_id in &struct_.prop_ids {
        let prop = ast.props.get_mut(&prop_id).unwrap();
        if prop.body_ids.len() != struct_.body_ids.len() {
          prop.optional = true;
        }
      }
    }
    btree_map::Entry::Vacant(entry) => unreachable!("Struct should be defined at this point"),
  }

  (AScriptTypeVal::Struct(id.clone()), errors)
}

pub fn get_production_types(
  ast: &AScriptStore,
  prod_id: &ProductionId,
) -> BTreeSet<AScriptTypeVal> {
  ast.prod_types.get(prod_id).unwrap().keys().map(|t| t.into()).collect::<BTreeSet<_>>()
}

/// Returns a specified vector type from a generic vector
pub fn get_specified_vector_from_generic_vec_values(
  vals: &BTreeSet<AScriptTypeVal>,
) -> AScriptTypeVal {
  if vals.len() > 1 {
    if vals.iter().all(|t| {
      matches!(
        t,
        AScriptTypeVal::Struct(..)
          | AScriptTypeVal::GenericStructVec(..)
          | AScriptTypeVal::GenericStruct(..)
      )
    }) {
      AScriptTypeVal::GenericStructVec(
        vals
          .iter()
          .flat_map(|n| match n {
            AScriptTypeVal::Struct(id) => {
              vec![TaggedType { type_: n.clone(), ..Default::default() }]
            }
            AScriptTypeVal::GenericStruct(struct_ids) => struct_ids.iter().cloned().collect(),
            _ => vec![],
          })
          .collect::<BTreeSet<_>>(),
      )
    } else if vals
      .iter()
      .all(|t| matches!(t, AScriptTypeVal::String(..) | AScriptTypeVal::StringVec))
    {
      AScriptTypeVal::StringVec
    } else if vals.iter().all(|t| matches!(t, AScriptTypeVal::Token | AScriptTypeVal::TokenVec)) {
      AScriptTypeVal::TokenVec
    } else if vals.iter().all(|t| {
      matches!(
        t,
        AScriptTypeVal::U8(..)
          | AScriptTypeVal::U8Vec
          | AScriptTypeVal::U16(..)
          | AScriptTypeVal::U16Vec
          | AScriptTypeVal::U32(..)
          | AScriptTypeVal::U32Vec
          | AScriptTypeVal::U64(..)
          | AScriptTypeVal::U64Vec
          | AScriptTypeVal::I8(..)
          | AScriptTypeVal::I8Vec
          | AScriptTypeVal::I16(..)
          | AScriptTypeVal::I16Vec
          | AScriptTypeVal::I32(..)
          | AScriptTypeVal::I32Vec
          | AScriptTypeVal::I64(..)
          | AScriptTypeVal::I64Vec
          | AScriptTypeVal::F32(..)
          | AScriptTypeVal::F32Vec
          | AScriptTypeVal::F64(..)
          | AScriptTypeVal::F64Vec
      )
    }) {
      match vals
        .iter()
        .map(|v| match v {
          AScriptTypeVal::U8(..) | AScriptTypeVal::U8Vec => 1,
          AScriptTypeVal::I8(..) | AScriptTypeVal::I8Vec => 2,
          AScriptTypeVal::U16(..) | AScriptTypeVal::U16Vec => 3,
          AScriptTypeVal::I16(..) | AScriptTypeVal::I16Vec => 4,
          AScriptTypeVal::U32(..) | AScriptTypeVal::U32Vec => 5,
          AScriptTypeVal::I32(..) | AScriptTypeVal::I32Vec => 6,
          AScriptTypeVal::U64(..) | AScriptTypeVal::U64Vec => 7,
          AScriptTypeVal::I64(..) | AScriptTypeVal::I64Vec => 8,
          AScriptTypeVal::F32(..) | AScriptTypeVal::F32Vec => 9,
          AScriptTypeVal::F64(..) | AScriptTypeVal::F64Vec => 10,
          _ => 0,
        })
        .fold(0, |a, b| a.max(b))
      {
        1 => AScriptTypeVal::U8Vec,
        2 => AScriptTypeVal::I8Vec,
        3 => AScriptTypeVal::U16Vec,
        4 => AScriptTypeVal::I16Vec,
        5 => AScriptTypeVal::U32Vec,
        6 => AScriptTypeVal::I32Vec,
        7 => AScriptTypeVal::U64Vec,
        8 => AScriptTypeVal::I64Vec,
        9 => AScriptTypeVal::F32Vec,
        10 => AScriptTypeVal::F64Vec,
        _ => AScriptTypeVal::Undefined,
      }
    } else {
      AScriptTypeVal::Undefined
    }
  } else {
    match vals.first().unwrap() {
      AScriptTypeVal::Struct(id) => {
        AScriptTypeVal::GenericStructVec(BTreeSet::from_iter(vec![TaggedType {
          type_: AScriptTypeVal::Struct(*id),
          ..Default::default()
        }]))
      }
      AScriptTypeVal::GenericStruct(ids) => {
        AScriptTypeVal::GenericStructVec(ids.iter().cloned().collect())
      }
      AScriptTypeVal::U8(..) => AScriptTypeVal::U8Vec,
      AScriptTypeVal::U16(..) => AScriptTypeVal::U16Vec,
      AScriptTypeVal::U32(..) => AScriptTypeVal::U32Vec,
      AScriptTypeVal::U64(..) => AScriptTypeVal::U64Vec,
      AScriptTypeVal::I8(..) => AScriptTypeVal::I8Vec,
      AScriptTypeVal::I16(..) => AScriptTypeVal::I16Vec,
      AScriptTypeVal::I32(..) => AScriptTypeVal::I32Vec,
      AScriptTypeVal::I64(..) => AScriptTypeVal::I64Vec,
      AScriptTypeVal::F32(..) => AScriptTypeVal::F32Vec,
      AScriptTypeVal::F64(..) => AScriptTypeVal::F64Vec,
      AScriptTypeVal::Token => AScriptTypeVal::TokenVec,
      AScriptTypeVal::String(..) => AScriptTypeVal::StringVec,
      _ => AScriptTypeVal::Undefined,
    }
  }
}

pub fn get_named_body_ref<'a>(body: &'a Body, val: &str) -> Option<(usize, &'a BodySymbol)> {
  if val == "first" {
    Some((0, body.syms.first().unwrap()))
  } else if val == "last" {
    Some((body.syms.len() - 1, body.syms.last().unwrap()))
  } else {
    body.syms.iter().enumerate().filter(|(_, s)| s.annotation == *val).last()
  }
}

pub fn get_indexed_body_ref<'a>(body: &'a Body, i: &f64) -> Option<(usize, &'a BodySymbol)> {
  body.syms.iter().enumerate().filter(|(_, s)| s.original_index == (*i - 1.0) as u32).last()
}

pub fn get_struct_type_from_node(ast_struct: &AST_Struct) -> AScriptTypeVal {
  let types = ast_struct
    .props
    .iter()
    .filter_map(|node| match node {
      ASTNode::AST_TypeId(id) => Some(id),
      _ => None,
    })
    .collect::<Vec<_>>();

  // Use the last type as the official type name of the struct.
  if let Some(node) = types.last() {
    AScriptTypeVal::Struct(AScriptStructId::new(&node.value.clone()[2..]))
  } else {
    AScriptTypeVal::Undefined
  }
}

pub fn production_types_are_structs(production_types: &BTreeSet<AScriptTypeVal>) -> bool {
  production_types.iter().all(|t| matches!(t.clone(), AScriptTypeVal::Struct(..)))
}
