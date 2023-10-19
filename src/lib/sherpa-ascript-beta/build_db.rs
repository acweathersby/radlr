use std::{collections::VecDeque, error::Error, rc::Rc, sync::Arc};

use sherpa_core::{
  parser::{ASTNode, AST_Struct},
  proxy::{OrderedMap, OrderedSet},
  ASTToken,
  CachedString,
  DBNonTermKey,
  DBRuleKey,
  Item,
  ParserDatabase,
  SherpaDatabase,
  SherpaResult,
  SymbolRef,
};

use crate::types::*;

pub fn build_database(db: SherpaDatabase) -> AscriptDatabase {
  let mut adb = AscriptDatabase {
    errors:  Default::default(),
    structs: Default::default(),
    rules:   Default::default(),
    db:      db.into_internal(),
  };

  let db = adb.db.clone();
  let db = &db;

  extract_structs(db, &mut adb);

  let nonterm_types = resolve_nonterm_types(db, &mut adb);

  resolve_expressions(db, &mut adb, nonterm_types);

  adb
}

pub fn extract_structs(db: &ParserDatabase, adb: &mut AscriptDatabase) {
  for (id, db_rule) in db.rules().iter().enumerate().filter(|(_, r)| !r.is_scanner) {
    let rule = &db_rule.rule;

    let rule = match &rule.ast {
      None => AscriptRule::LastSymbol(id, Initializer {
        ty:           AscriptType::Undefined,
        name:         Default::default(),
        output_graph: None,
        ast:          None,
      }),
      Some(ast) => match ast {
        ASTToken::Defined(ast) => match &ast.ast {
          ASTNode::AST_Struct(strct) => match process_struct(adb, strct) {
            Ok(struct_initializer) => AscriptRule::Struct(id, struct_initializer),
            Err(err) => {
              adb.errors.push(err);
              AscriptRule::Invalid(id)
            }
          },
          ASTNode::AST_Statements(stmt) => AscriptRule::Expression(id, Initializer {
            ty:           AscriptType::Undefined,
            name:         Default::default(),
            output_graph: None,
            ast:          Some(ASTNode::AST_Statements(stmt.clone())),
          }),
          _ => unreachable!(),
        },
        ASTToken::ListEntry(_) => AscriptRule::ListInitial(id, Initializer {
          ty:           AscriptType::Undefined,
          name:         Default::default(),
          output_graph: None,
          ast:          None,
        }),
        ASTToken::ListIterate(tok) => AscriptRule::ListContinue(id, Initializer {
          ty:           AscriptType::Undefined,
          name:         Default::default(),
          output_graph: None,
          ast:          None,
        }),
      },
    };

    adb.rules.push(rule);
  }
}

pub fn process_struct(adb: &mut AscriptDatabase, strct: &AST_Struct) -> SherpaResult<StructInitializer> {
  let name = &strct.typ[2..];
  let struct_id = StringId::from(name);
  let mut initializer = StructInitializer { name: struct_id, props: Default::default() };
  let mut seen = OrderedSet::new();

  for prop in &strct.props {
    if let Some(prop) = prop.as_AST_Property() {
      let prop_name: StringId = prop.id.as_str().into();
      let mut ast_prop = AScriptProp {
        is_optional: false,
        name:        prop.id.to_string(),
        tok:         prop.tok.clone(),
        ty:          AscriptType::Undefined,
      };

      seen.insert(prop_name);

      {
        let mut existing_struct = true;
        let ast_struct = adb.structs.entry(struct_id).or_insert_with(|| {
          existing_struct = false;
          AscriptStruct {
            id:         struct_id,
            name:       name.to_string(),
            properties: Default::default(),
          }
        });

        match ast_struct.properties.get_mut(&prop_name) {
          Some(..) => {}
          None => {
            if existing_struct {
              ast_prop.is_optional = true;
            }
            ast_struct.properties.insert(prop_name, ast_prop);
          }
        }
      }

      initializer.props.insert(StringId::from(prop.id.to_string()), Initializer {
        ty:           AscriptType::Undefined,
        name:         prop_name,
        output_graph: None,
        ast:          prop.value.clone(),
      });
    }
  }

  for (name, prop) in adb.structs.get_mut(&struct_id).unwrap().properties.iter_mut() {
    if !seen.contains(name) {
      prop.is_optional |= true;
    }
  }

  Ok(initializer)
}

/// Derives the AST types of all parser NonTerminals.
pub fn resolve_nonterm_types(db: &ParserDatabase, adb: &mut AscriptDatabase) -> OrderedMap<DBNonTermKey, AscriptType> {
  let mut pending = adb
    .rules
    .0
    .iter()
    .enumerate()
    .map(|(i, _)| {
      (i, DBRuleKey::from(i), PendingType::Unresolved {
        non_terms:  Default::default(),
        scalars:    Default::default(),
        aggregates: Default::default(),
      })
    })
    .collect::<Vec<_>>();

  let mut nonterms = OrderedMap::new();
  for rule in db.rules() {
    nonterms.entry(rule.nonterm).or_insert((AscriptType::Undefined, 0, 0)).1 += 1
  }

  loop {
    let mut complete = true;

    for (index, rule_id, type_data) in &mut pending {
      if db.rules()[*index].is_scanner {
        continue;
      }

      match type_data {
        PendingType::Unresolved { non_terms: pending_nonterm, scalars, aggregates } => {
          let rule = &db.rules()[*index];
          let nonterm_id = rule.nonterm;

          if pending_nonterm.len() > 0 {
            let mut queue = VecDeque::from_iter(pending_nonterm.iter().cloned());
            while let Some(non_term) = queue.pop_front() {
              match nonterms.get_mut(&non_term) {
                Some((_, 0, _)) => {
                  pending_nonterm.remove(&non_term);
                }
                Some((_, 1.., r)) if non_term == nonterm_id => {
                  // Self referential
                  pending_nonterm.remove(&non_term);
                }
                _ => {}
              }
            }
          }

          let item = Item::from((*rule_id, db));

          let ty = match &mut adb.rules[*index] {
            AscriptRule::LastSymbol(..) => match item.to_complete().decrement() {
              Some(item) => match item.nonterm_index_at_sym(Default::default(), db) {
                Some(nonterm_key) => match nonterms.get_mut(&nonterm_key) {
                  Some((ty, resolved, self_recursive)) => {
                    if *resolved == 0 {
                      *ty
                    } else {
                      *self_recursive += (nonterm_key == nonterm_id) as u32;
                      pending_nonterm.insert(nonterm_key);
                      AscriptType::Undefined
                    }
                  }
                  None => AscriptType::Undefined,
                },
                None => AscriptType::Scalar(AscriptScalarType::Token),
              },
              None => AscriptType::Undefined,
            },
            AscriptRule::ListContinue(..) | AscriptRule::ListInitial(..) => match item.to_complete().decrement() {
              Some(item) => match item.nonterm_index_at_sym(Default::default(), db) {
                Some(nonterm_key) => match nonterms.get_mut(&nonterm_key) {
                  Some((ty, v, self_recursive)) => {
                    if *v == 0 {
                      match ty {
                        AscriptType::Scalar(ty) => AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: *ty }),
                        AscriptType::Aggregate(ty) => match ty {
                          AscriptAggregateType::Map { val_type, .. } => {
                            AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: *val_type })
                          }
                          AscriptAggregateType::Vec { base_type } => {
                            AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: *base_type })
                          }
                        },
                        _ => AscriptType::Undefined,
                      }
                    } else {
                      *self_recursive += (nonterm_key == nonterm_id) as u32;
                      pending_nonterm.insert(nonterm_key);
                      AscriptType::Undefined
                    }
                  }
                  None => AscriptType::Undefined,
                },
                None => AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: AscriptScalarType::Token }),
              },
              None => AscriptType::Undefined,
            },
            AscriptRule::Struct(_, id) => AscriptType::Scalar(AscriptScalarType::Struct(id.name)),
            AscriptRule::Invalid(..) => AscriptType::Undefined,
            AscriptRule::Expression(_, init) => {
              todo!("Resolve stmt type");
            }
          };

          if pending_nonterm.len() > 0 {
            // Keep this alive until all non_terminal requirements are
            // resolved.
            complete = false;
            continue;
          }

          match ty {
            AscriptType::Undefined => {
              *type_data = PendingType::Resolved(AscriptType::Undefined);
              continue;
            }
            a_ty => match nonterms.get_mut(&nonterm_id) {
              Some((nterm_ty, resolved, _)) => {
                *type_data = PendingType::Resolved(*nterm_ty);
                match &mut adb.rules[*index] {
                  AscriptRule::ListContinue(_, init) => init.ty = a_ty,
                  AscriptRule::ListInitial(_, init) => init.ty = a_ty,
                  AscriptRule::Expression(_, init) => init.ty = a_ty,
                  AscriptRule::LastSymbol(_, init) => init.ty = a_ty,
                  _ => {}
                }
                *resolved -= 1;
                match get_resolved_type(a_ty, *nterm_ty) {
                  Ok(ty) => *nterm_ty = ty,
                  Err(err) => todo!("Create Error for failed type \n{err}"),
                }
              }
              None => unreachable!(),
            },
          }
        }
        _ => {}
      }
    }

    if complete {
      break;
    }
  }

  nonterms.into_iter().map(|(k, (v, ..))| (k, v)).collect()
}

fn resolve_expressions(db: &ParserDatabase, adb: &mut AscriptDatabase, nonterm_types: OrderedMap<DBNonTermKey, AscriptType>) {
  let AscriptDatabase { structs, rules, .. } = adb;

  for (index, ast_rule) in rules.iter_mut().enumerate() {
    let item = Item::from((DBRuleKey::from(index), db));
    match ast_rule {
      AscriptRule::Expression(_, init) => {
        let Initializer { ast, output_graph, .. } = init;

        if let Some(node) = &ast {
          *output_graph = Some(resolve_node(db, node, item, &nonterm_types));
        }
      }
      AscriptRule::ListInitial(_, init) => {
        let Initializer { output_graph, ty, .. } = init;

        let last = match get_item_at_sym_ref(item, db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, db, &nonterm_types),
          None => GraphNode::Undefined(AscriptType::Undefined),
        };

        //*ty = AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: last.get_type().as_scalar().unwrap() });

        let last = GraphNode::Vec(GraphNodeVecInits(vec![last]), *ty);

        *output_graph = Some(last);
      }
      AscriptRule::ListContinue(_, init) => {
        let Initializer { output_graph, ty, .. } = init;
        let first = match get_item_at_sym_ref(item, db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, db, &nonterm_types),
          None => GraphNode::Undefined(AscriptType::Undefined),
        };
        let last = match get_item_at_sym_ref(item, db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, db, &nonterm_types),
          None => GraphNode::Undefined(AscriptType::Undefined),
        };

        //*ty = AscriptType::Aggregate(AscriptAggregateType::Vec { base_type: last.get_type().as_scalar().unwrap() });

        let first = GraphNode::Vec(GraphNodeVecInits(vec![last.clone()]), *ty);
        let join = GraphNode::Add(Rc::new(first), Rc::new(last), *ty);
        *output_graph = Some(join);
      }
      AscriptRule::LastSymbol(_, init) => {
        let Initializer { output_graph, ty, .. } = init;

        let graph_node = match get_item_at_sym_ref(item, db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, db, &nonterm_types),
          None => GraphNode::Undefined(AscriptType::Undefined),
        };

        *ty = *graph_node.get_type();
        *output_graph = Some(graph_node);
      }
      AscriptRule::Struct(_, strct) => {
        let id: StringId = strct.name;
        for (name, init) in strct.props.iter_mut() {
          if let Some(node) = &init.ast {
            let node = resolve_node(db, node, item, &nonterm_types);
            if let Some(strct) = structs.get_mut(&id) {
              if let Some(prop) = strct.properties.get_mut(name) {
                prop.ty = *node.get_type();
                init.ty = *node.get_type();
              }
            }
            init.output_graph = Some(node);
          }
        }
      }
      r => todo!("handle rule {r:?}"),
    }
  }
}

fn get_item_at_sym_ref<'db, F: Fn(Item, &SymbolRef) -> bool>(item: Item, db: &'db ParserDatabase, funct: F) -> Option<Item> {
  let mut i = item;
  while let Some(sym) = i.sym(db) {
    if funct(i, sym) {
      return Some(i);
    }
    i = i.try_increment();
  }
  None
}
fn graph_node_from_item(
  item: Item,
  db: &ParserDatabase,
  nonterm_types: &std::collections::BTreeMap<DBNonTermKey, AscriptType>,
) -> GraphNode {
  if let Some(nonterm_id) = item.nonterm_index_at_sym(Default::default(), db) {
    let ty = nonterm_types.get(&nonterm_id).unwrap();
    GraphNode::Sym(item.sym_index() as usize, *ty)
  } else {
    GraphNode::TokSym(item.sym_index() as usize, AscriptType::Scalar(AscriptScalarType::Token))
  }
}

fn resolve_node(
  db: &ParserDatabase,
  node: &ASTNode,
  item: Item,
  nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>,
) -> GraphNode {
  match node {
    ASTNode::AST_Vector(vec) => {
      let mut ty = AscriptType::Undefined;
      let mut initializers = vec![];
      for node in vec.initializer.iter().map(|t| resolve_node(db, t, item, nonterm_types)) {
        ty = get_resolved_type(node.get_type().clone(), ty).unwrap();
        initializers.push(node);
      }
      GraphNode::Vec(GraphNodeVecInits(initializers), ty)
    }
    ASTNode::AST_NamedReference(rf) => match get_item_at_sym_ref(item, db, |_, sym| sym.annotation == rf.value.to_token()) {
      Some(item) => graph_node_from_item(item, db, nonterm_types),
      None => GraphNode::Undefined(AscriptType::Undefined),
    },
    ASTNode::AST_IndexReference(rf) => {
      match get_item_at_sym_ref(item, db, |_, sym| sym.original_index as isize == (rf.value - 1) as isize) {
        Some(item) => graph_node_from_item(item, db, nonterm_types),
        None => GraphNode::Undefined(AscriptType::Undefined),
      }
    }
    ASTNode::AST_Add(add) => {
      let l = resolve_node(db, &add.left, item, nonterm_types);
      let r = resolve_node(db, &add.right, item, nonterm_types);
      match l.get_type() {
        AscriptType::Scalar(l_type) => match l_type {
          _ => todo!("resolve add of {l_type:?} {r:?}"),
        },
        AscriptType::Aggregate(l_type) => match l_type {
          _ => todo!("resolve add of {l_type:?} {r:?}"),
        },
        _ => unreachable!(),
      }
    }
    ASTNode::AST_STRING(str) => {
      if let Some(init) = &str.initializer {
        let gn = resolve_node(db, &init.expression, item, nonterm_types);
        GraphNode::Str(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::String(None)))
      } else {
        GraphNode::Str(None, AscriptType::Scalar(AscriptScalarType::String(None)))
      }
    }
    ASTNode::AST_BOOL(bool) => {
      if let Some(init) = &bool.initializer {
        let gn = resolve_node(db, &init.expression, item, nonterm_types);
        GraphNode::Bool(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::Bool(false)))
      } else {
        GraphNode::Bool(None, AscriptType::Scalar(AscriptScalarType::Bool(bool.value)))
      }
    }
    node => todo!("handle graph resolve of node {node:#?}"),
  }
}

/// Attempts to merge two different types into a single, compatible type.
///
/// Returns an Err if the the combination of types cannot be reasonably merged,
/// such as Vec<u32> and Vec<String>, a Struct and a String, or a Struct and a
/// Struct.
fn get_resolved_type(a: AscriptType, b: AscriptType) -> SherpaResult<AscriptType> {
  use AscriptAggregateType::*;
  use AscriptType::*;
  match b {
    Undefined => Ok(a),
    Aggregate(b_agg) => match a {
      Aggregate(a_agg) => match b_agg {
        Map { key_type: b_key_type, val_type: b_val_type } => match a_agg {
          Map { key_type: a_key_type, val_type: a_val_type } => {
            let key_type = get_resolved_type(Scalar(a_key_type), Scalar(b_key_type))?.as_scalar().unwrap_or_default();
            let val_type = get_resolved_type(Scalar(a_val_type), Scalar(b_val_type))?.as_scalar().unwrap_or_default();
            Ok(Aggregate(Map { key_type, val_type }))
          }
          Vec { base_type: a_base_type } => {
            let base_type = get_resolved_type(Scalar(a_base_type), Scalar(b_val_type))?.as_scalar().unwrap_or_default();
            Ok(Aggregate(Vec { base_type }))
          }
        },
        Vec { base_type: b_base_type } => match a_agg {
          Map { key_type: a_key_type, val_type: a_val_type } => {
            let base_type = get_resolved_type(Scalar(b_base_type), Scalar(a_val_type))?.as_scalar().unwrap_or_default();
            Ok(Aggregate(Vec { base_type }))
          }
          Vec { base_type: a_base_type } => {
            let base_type = get_resolved_type(Scalar(b_base_type), Scalar(a_base_type))?.as_scalar().unwrap_or_default();
            Ok(Aggregate(Vec { base_type }))
          }
        },
      },
      Scalar(_) => match b_agg {
        Vec { base_type } => Ok(Aggregate(Vec {
          base_type: get_resolved_type(a, Scalar(base_type))?.as_scalar().unwrap_or_default(),
        })),
        Map { val_type, .. } => Ok(Aggregate(Vec {
          base_type: get_resolved_type(a, Scalar(val_type))?.as_scalar().unwrap_or_default(),
        })),
      },

      Undefined => Ok(b),
      A_ => todo!("resolve different types {a:?} {b:?}"),
    },
    Scalar(a_scalar) => match a {
      Undefined => Ok(b),
      Scalar(b_scalar) => {
        if std::mem::discriminant(&a_scalar) == std::mem::discriminant(&b_scalar) {
          Ok(a)
        } else {
          todo!("resolve different scalar types {a:?} {b:?}")
        }
      }
      _ => todo!("Resolve types scaler:{a:?} ty:{b:?}"),
    },
    _ => todo!("Resolve types ty:{a:?} ty:{b:?}"),
  }
}
