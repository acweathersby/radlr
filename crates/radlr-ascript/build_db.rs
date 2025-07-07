use std::{
  collections::{BTreeSet, HashSet, VecDeque},
  rc::Rc,
};

use radlr_core::{
  parser::{ASTNode, AST_Struct},
  proxy::{OrderedMap, OrderedSet},
  ASTToken,
  CachedString,
  DBNonTermKey,
  DBRuleKey,
  GrammarDatabase,
  Item,
  RadlrError,
  RadlrGrammarDatabase,
  RadlrResult,
  SymbolRef,
};

use crate::types::*;

pub fn build_database(db: RadlrGrammarDatabase) -> AscriptDatabase {
  let mut adb = AscriptDatabase {
    errors:        Default::default(),
    structs:       Default::default(),
    rules:         Default::default(),
    types:         Default::default(),
    multi_types:   Default::default(),
    multi_type_lu: Default::default(),
    db:            db.into_internal(),
  };

  let db = adb.db.clone();
  let db = &db;

  construct_initializers(db, &mut adb);

  let nonterm_types = resolve_nonterm_types(db, &mut adb);

  if adb.errors.is_empty() {
    match resolve_nonterm_values(&mut adb, &nonterm_types) {
      Ok(()) => {
        fill_struct_rules(&mut adb, &nonterm_types);

        finalize_structs(&mut adb, &nonterm_types);

        resolve_multi_types(&mut adb);

        resolve_struct_definitions(&mut adb);

        if let Err(err) = collect_types(&mut adb) {
          adb.errors.push(RadlrError::Text(err));
        }
      }
      Err(missing_nonterm_definition) => adb.errors.push(RadlrError::Text(format!(
        "Could not resolve Node type for Non-Term [{}]",
        db.nonterm_friendly_name(missing_nonterm_definition).to_string(db.string_store())
      ))),
    }
  }

  adb
}

fn finalize_structs(adb: &mut AscriptDatabase, nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>) {
  for initializer_index in 0..adb.rules.0.len() {
    match &adb.rules.0[initializer_index] {
      AscriptRule::Struct(..) => {
        post_process_struct_initializer(adb, initializer_index, nonterm_types);
      }
      _ => {}
    }
  }

  let AscriptDatabase { rules, structs, db, .. } = adb;

  for rule in &mut rules.0 {
    match rule {
      AscriptRule::Struct(_, init) => {
        if let Some(s) = structs.get(&init.name) {
          for (prop_name, prop) in &s.properties.0 {
            match init.props.0.entry(*prop_name) {
              std::collections::btree_map::Entry::Vacant(init_prop) => {
                init_prop.insert(Initializer {
                  ty:                prop.ty,
                  name:              *prop_name,
                  output_graph:      None,
                  ast:               None,
                  g_id:              db.root_grammar_id,
                  rule_local_string: Default::default(),
                  optional:          prop.is_optional,
                });
              }
              std::collections::btree_map::Entry::Occupied(mut init) => {
                let init = init.get_mut();
                init.optional = prop.is_optional;
                init.ty = prop.ty;
              }
            }
          }
        }
      }
      _ => {}
    }
  }
}

fn fill_struct_rules(adb: &mut AscriptDatabase, nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>) {
  let db = adb.db.clone();

  for initializer_index in 0..adb.rules.0.len() {
    match &adb.rules.0[initializer_index] {
      AscriptRule::Struct(_, strct_initializer) => {
        let rule = &db.rules()[strct_initializer.rule_index];
        let db_nonterm_key = rule.nonterm;
        let rule = &rule.rule;

        match &rule.ast {
          Some(ast) => match ast {
            ASTToken::Defined(ast) => match &ast.ast {
              ASTNode::AST_Struct(strct) => {
                process_struct_initializer(adb, strct, initializer_index, nonterm_types, db_nonterm_key);
              }
              _ => unreachable!(),
            },
            _ => unreachable!(),
          },
          _ => unreachable!(),
        };
      }
      _ => {}
    }
  }
}

fn resolve_struct_definitions(adb: &mut AscriptDatabase) {
  let mut template_required = VecDeque::with_capacity(adb.structs.len());
  let mut dependency_lookup: std::collections::BTreeMap<StringId, BTreeSet<StringId>> = OrderedMap::new();

  for (strct_id, strct) in adb.structs.iter() {
    if strct.has_token || strct.properties.iter().any(|p| p.1.ty.requires_template(&adb)) {
      template_required.push_back(*strct_id);
    }

    for (_, prop) in strct.properties.iter() {
      let mut structs = vec![];

      for dependent_stct_id in prop.ty.get_structs(&mut structs, &adb.multi_types) {
        let entry = dependency_lookup.entry(*dependent_stct_id).or_default();
        entry.insert(*strct_id);
      }
    }
  }

  let AscriptDatabase { rules, structs, db, .. } = adb;

  while let Some(struct_id) = template_required.pop_front() {
    let strct = structs.get_mut(&struct_id).unwrap();

    if !strct.requires_template {
      strct.requires_template = true;
      if let Some(dependencies) = dependency_lookup.get(&struct_id) {
        template_required.extend(dependencies.iter().cloned())
      }
    }
  }

  for (_, structure) in &mut structs.0 {
    let mut properties_to_remove = vec![];

    for (prop_name, prop) in &structure.properties.0 {
      let ty = prop.ty.to_cardinal();

      if ty.is_unknown() {
        properties_to_remove.push(prop_name.clone())
      }
    }

    for prop_name in properties_to_remove {
      structure.properties.0.remove(&prop_name);
    }
  }

  for rule in &mut rules.0 {
    match rule {
      AscriptRule::Struct(_, init) => {
        let name = init.name;

        let structure = structs.0.get(&name).expect(&format!("Could not find struct , {}", name.0.to_string(&db.string_store())));

        let mut properties_to_remove = vec![];

        for (prop_name, _) in &init.props.0 {
          if !structure.properties.contains_key(prop_name) {
            properties_to_remove.push(prop_name.clone());
          }
        }

        for prop_name in properties_to_remove {
          init.props.0.remove(&prop_name);
        }
      }
      _ => {}
    }
  }
}

pub fn process_struct_initializer(
  adb: &mut AscriptDatabase,
  ast_struct: &AST_Struct,
  initializer_index: usize,
  nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>,
  db_nt_key: DBNonTermKey,
) {
  let db = adb.db.clone();
  let db = db.as_ref();
  let g_id = adb.db.root_grammar_id;
  let s_store = adb.db.string_store();
  let AscriptDatabase { rules, structs, multi_type_lu, multi_types, .. } = adb;

  let struct_initializer = match &mut rules[initializer_index] {
    AscriptRule::Struct(_, init) => init,
    _ => unreachable!(),
  };

  let rule_index = struct_initializer.rule_index;

  let item = Item::from((DBRuleKey::from(rule_index), db));

  let name = &ast_struct.ty[2..];
  let struct_id = StringId(name.intern(s_store));

  let mut seen: BTreeSet<StringId> = OrderedSet::new();

  let ascript_struct = structs.get_mut(&struct_id).expect("Should have been built in `construct_initializers`");
  let existing_struct = ascript_struct.initialized;
  ascript_struct.initialized = true;

  for prop in &ast_struct.props {
    let (name, prop_name, ast, is_token, tok) = match prop {
      ASTNode::AST_Property(prop) => {
        let prop_name: StringId = prop.id.as_str().into();
        (prop.id.as_str().to_string(), prop_name, prop.value.clone(), false, prop.tok.clone())
      }
      ast @ ASTNode::AST_Token(..) => {
        let tok_id = StringId("tok".intern(s_store));
        ("tok".to_string(), tok_id, Some(ast.clone()), true, Default::default())
      }
      _ => unreachable!(),
    };
    seen.insert(prop_name);

    let (ty, nt_key) = match &ast {
      None => {
        // the property is a named value, so we may be able to resolve this
        // with a symbol reference name lookup.
        match get_item_at_sym_ref(item, &adb.db, |_, sym_ref| sym_ref.annotation == prop_name.0) {
          Some(item) => {
            (graph_type_from_item(item, &adb.db, nonterm_types, None).0, item.nonterm_index_at_sym_parser(db).unwrap_or_default())
          }
          None => match get_item_at_sym_ref(item, &adb.db, |_, sym_ref| match sym_ref.id {
            radlr_core::SymbolId::DBNonTerminal { key } => db.nonterm_friendly_name(key) == prop_name.0,
            _ => false,
          }) {
            Some(item) => (
              graph_type_from_item(item, &adb.db, nonterm_types, None).0,
              item.nonterm_index_at_sym_parser(db).unwrap_or_default(),
            ),
            None => (AscriptType::Undefined, db_nt_key),
          },
        }
      }
      Some(node) => (
        get_graph_type(
          GraphResolveData {
            db,
            item,
            node,
            nonterm_types: &nonterm_types,
            default_nonterm_type: None,
          },
          &mut GraphMutData {
            selected_indices: &mut HashSet::new(),
            multi_indices:    multi_type_lu,
            multi_maps:       multi_types,
          },
          Default::default(),
        )
        .expect("Could not resolve prop type"),
        db_nt_key,
      ),
    };

    struct_initializer.props.insert(prop_name, Initializer {
      ty,
      name: prop_name,
      output_graph: None,
      ast,
      g_id,
      rule_local_string: Default::default(),
      optional: false,
    });

    if is_token {
      ascript_struct.has_token = true;
    }

    match ascript_struct.properties.get_mut(&prop_name) {
      Some(ast_prop) => {
        if ty.is_unknown() {
          ast_prop.is_optional = true;
        } else {
          ast_prop.ty =
            get_resolved_type(&ast_prop.ty, &ty, multi_type_lu, multi_types, nt_key).expect("Could not resolve prop type");
        }
      }
      None => {
        let mut ast_prop = AscriptProp { is_optional: false, name, tok, ty, g_id };
        if existing_struct {
          ast_prop.is_optional = true;
        }
        ascript_struct.properties.insert(prop_name, ast_prop);
      }
    }
  }

  for (name, ast_prop) in structs.get_mut(&struct_id).unwrap().properties.iter_mut() {
    if !seen.contains(name) {
      ast_prop.is_optional |= true;
    }
  }
}

pub fn post_process_struct_initializer(
  adb: &mut AscriptDatabase,
  initializer_index: usize,
  nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>,
) {
  let db = adb.db.clone();
  let db = db.as_ref();
  let AscriptDatabase { rules, structs, multi_type_lu, multi_types, .. } = adb;

  let struct_initializer = match &mut rules[initializer_index] {
    AscriptRule::Struct(_, init) => init,
    _ => unreachable!(),
  };

  let rule_index = struct_initializer.rule_index;

  let item = Item::from((DBRuleKey::from(rule_index), db));
  let struct_name: StringId = struct_initializer.name;

  for (prop_name, init) in struct_initializer.props.iter_mut().rev() {
    let (mut node, rule_tok) = match &init.ast {
      Some(node) => (
        create_graph_node(
          GraphResolveData {
            db,
            item,
            node: init.ast.as_ref().unwrap(),
            nonterm_types: &nonterm_types,
            default_nonterm_type: None,
          },
          &mut GraphMutData {
            selected_indices: &mut HashSet::new(),
            multi_indices:    multi_type_lu,
            multi_maps:       multi_types,
          },
          Default::default(),
        )
        .expect("Could not create graph"),
        node.to_token(),
      ),
      None => {
        // the property is a named value, so we may be able to resolve this
        // with a symbol reference name lookup.

        let node = match get_item_at_sym_ref(item, &adb.db, |_, sym_ref| sym_ref.annotation == prop_name.0) {
          Some(item) => graph_node_from_item(item, &adb.db, &nonterm_types, &mut HashSet::new(), None).expect("Uknown error"),
          None => match get_item_at_sym_ref(item, &adb.db, |_, sym_ref| match sym_ref.id {
            radlr_core::SymbolId::DBNonTerminal { key } => db.nonterm_friendly_name(key) == prop_name.0,
            _ => false,
          }) {
            Some(item) => graph_node_from_item(item, &adb.db, &nonterm_types, &mut HashSet::new(), None).expect("Uknown error"),
            None => GraphNode::Undefined(AscriptType::Undefined),
          },
        };

        (node, Default::default())
      }
    };

    if !node.get_type().is_unknown() {
      if let Some(archetype_struct) = structs.get_mut(&struct_name) {
        if let Some(a_prop) = archetype_struct.properties.get_mut(prop_name) {
          let rule_prop_type = node.get_type();

          match (&a_prop.ty, rule_prop_type, a_prop.ty == *rule_prop_type) {
            (_, _, true) => {}
            (_, AscriptType::Undefined, _) => {
              a_prop.is_optional = true;
            }
            (AscriptType::Undefined, ..) => {
              a_prop.tok = rule_tok;
              a_prop.g_id = init.g_id;
            }
            (AscriptType::Scalar(AscriptScalarType::Multi(..)), rule_ty, ..) if !rule_ty.is_unknown() => {
              if a_prop.ty.is_multi() && node.get_type().get_multi_type_index() != a_prop.ty.get_multi_type_index() {
                node = GraphNode::MultiConvert(Rc::new(node.clone()), a_prop.ty);
              }
            }
            _ => {}
          }

          init.ty = *node.get_type();
        }
      }

      match node {
        GraphNode::Undefined(_) => {}
        node => {
          init.output_graph = Some(node);
        }
      }
    }
  }
}

#[allow(unused)]
fn add_token_nodes(adb: &mut AscriptDatabase) {
  let AscriptDatabase { rules, structs, types, db, .. } = adb;

  let prop_name = StringId("tok".intern(db.string_store()));
  let tok_type = AscriptType::Scalar(AscriptScalarType::Token);
  for (_, s) in &mut structs.0 {
    if !s.has_token {
      s.has_token = true;
      s.properties.insert(prop_name, AscriptProp {
        is_optional: false,
        name:        "tok".to_string(),
        ty:          tok_type,
        tok:         Default::default(),
        g_id:        db.root_grammar_id,
      });
    }
  }

  for rule in &mut rules.0 {
    match rule {
      AscriptRule::Struct(_, init) => match init.props.0.entry(prop_name) {
        std::collections::btree_map::Entry::Vacant(prop) => {
          prop.insert(Initializer {
            ty:                tok_type,
            name:              prop_name,
            output_graph:      Some(GraphNode::TokRule(AscriptType::Scalar(AscriptScalarType::Token))),
            ast:               None,
            g_id:              db.root_grammar_id,
            rule_local_string: Default::default(),
            optional:          false,
          });
        }
        std::collections::btree_map::Entry::Occupied(_) => {}
      },
      _ => {}
    }
  }
}

/// Collect non-struct type information
fn collect_types(adb: &mut AscriptDatabase) -> Result<(), String> {
  let AscriptDatabase { rules, structs, types, multi_type_lu, .. } = adb;

  for (_, structure) in &structs.0 {
    for (_, prop) in &structure.properties.0 {
      let ty = prop.ty.to_cardinal();

      debug_assert!(!ty.is_unknown());

      add_to_type_list(ty, types, multi_type_lu);
    }
  }
  for rule in &mut rules.0 {
    match rule {
      AscriptRule::Expression(_, init)
      | AscriptRule::ListInitial(_, init)
      | AscriptRule::ListContinue(_, init)
      | AscriptRule::LastSymbol(_, init) => match &init.output_graph {
        Some(node) => {
          let ty = node.get_type().to_cardinal();

          debug_assert!(
            !ty.is_unknown(),
            "This should not be unknown: {}",
            init.ast.as_ref().map(|a| a.to_token()).unwrap_or_default().blame(1, 1, "Produced Undefined value", None)
          );

          add_to_type_list(ty, types, multi_type_lu);
        }
        None => return Err("Graph node could not be resolved".to_string()),
      },

      _ => {}
    }
  }

  Ok(())
}
fn resolve_multi_types(adb: &mut AscriptDatabase) {
  // Flatten multi types
  for i in 0..adb.multi_types.len() {
    let mut accessed_types = adb.multi_types.iter().map(|_| false).collect::<Vec<_>>();
    let mut outgoing_set = BTreeSet::new();
    let mut queue = VecDeque::from_iter(adb.multi_types[i].1.iter().cloned());

    while let Some(ty) = queue.pop_front() {
      if let AscriptScalarType::Multi(index) = ty {
        if !accessed_types[index] {
          accessed_types[index] = true;
          queue.extend(adb.multi_types[index].1.iter().cloned())
        }
      } else {
        outgoing_set.insert(ty);
      }
    }

    adb.multi_types[i].1 = outgoing_set;
    adb.multi_types[i].2 =
      accessed_types.iter().enumerate().filter_map(|(index, used)| (index != i && *used).then_some(index)).collect();
  }

  for i in 0..adb.multi_type_lu.len() {
    for j in 0..adb.multi_type_lu.len() {
      if j == i {
        continue;
      };

      if adb.multi_type_lu[j] != j {
        continue;
      }

      let target = &adb.multi_types[i];
      let other = &adb.multi_types[j];

      if other.0 < target.0 && other.1 == target.1 {
        assert_eq!(other.1.len(), target.1.len());

        let converts = adb.multi_types[i].2.clone();
        adb.multi_types[j].2.extend(converts);

        // Remap all multis that mapped to i to j
        for k in adb.multi_type_lu.iter_mut() {
          *k = if *k == i { j } else { *k }
        }
      }
    }

    for (i, k) in adb.multi_type_lu.iter().enumerate() {
      adb.multi_types[*k].2.remove(&i);
    }
  }
}

/// Extracts composite type information, resolves Any mappings, and inserts type
/// information into `types`. Token and Struct types are not inserted into
/// `types` as they are always compiled into type enums.
fn add_to_type_list(ty: AscriptType, types: &mut AscriptTypes, multi_i: &mut Vec<usize>) -> AscriptType {
  let (ty, insert) = match ty {
    _ty @ AscriptType::Aggregate(agg) => match agg {
      AscriptAggregateType::Map { key_type, val_type } => {
        let key = add_to_type_list(AscriptType::Scalar(key_type).to_cardinal(), types, multi_i);
        let val = add_to_type_list(AscriptType::Scalar(val_type).to_cardinal(), types, multi_i);
        let ty = AscriptType::Aggregate(AscriptAggregateType::Map {
          key_type: key.as_scalar().unwrap(),
          val_type: val.as_scalar().unwrap(),
        });
        (ty, true)
      }
      AscriptAggregateType::Vec { val_type } => {
        let val_type = add_to_type_list(AscriptType::Scalar(val_type).to_cardinal(), types, multi_i);
        let ty = AscriptType::Aggregate(AscriptAggregateType::Vec { val_type: val_type.as_scalar().unwrap() });
        (ty, true)
      }
    },
    AscriptType::Scalar(AscriptScalarType::Struct(..)) | AscriptType::Scalar(AscriptScalarType::Token) => (ty, false),
    AscriptType::Scalar(AscriptScalarType::Multi(index)) => (AscriptType::Scalar(AscriptScalarType::Multi(multi_i[index])), true),
    ty => (ty, true),
  };

  if insert {
    types.0.insert(ty);
  }
  ty
}

pub fn construct_initializers(db: &GrammarDatabase, adb: &mut AscriptDatabase) {
  for (id, db_rule) in db.rules().iter().enumerate().filter(|(_, r)| !r.is_scanner) {
    let rule = &db_rule.rule;
    let g_id = db_rule.rule.g_id;

    let ast_rule = match &rule.ast {
      None => AscriptRule::LastSymbol(id, Initializer {
        g_id,
        rule_local_string: rule.tok.to_string().intern(db.string_store()),
        ..Default::default()
      }),
      Some(ast) => match ast {
        ASTToken::Defined(ast) => match &ast.ast {
          ASTNode::AST_Struct(strct) => {
            let name = &strct.ty[2..];
            let struct_id = StringId(name.intern(db.string_store()));
            let struct_init =
              AscriptRule::Struct(id, StructInitializer { name: struct_id, rule_index: id, ..Default::default() });

            adb.structs.entry(struct_id).or_insert_with(|| AscriptStruct {
              id:                struct_id,
              name:              name.to_string(),
              properties:        Default::default(),
              has_token:         false,
              requires_template: false,
              initialized:       false,
            });

            struct_init
          }

          ASTNode::AST_Statement(stmt) => AscriptRule::Expression(id, Initializer {
            ty: AscriptType::Undefined,
            name: Default::default(),
            output_graph: None,
            ast: Some(stmt.expression.clone()),
            g_id,
            rule_local_string: rule.tok.to_string().intern(db.string_store()),
            optional: false,
          }),
          _ => unreachable!(),
        },
        ASTToken::ListEntry(_) => AscriptRule::ListInitial(id, Initializer {
          g_id,
          rule_local_string: rule.tok.to_string().intern(db.string_store()),
          ..Default::default()
        }),
        ASTToken::ListIterate(_) => AscriptRule::ListContinue(id, Initializer {
          g_id,
          rule_local_string: rule.tok.to_string().intern(db.string_store()),
          ..Default::default()
        }),
      },
    };

    for init in &mut adb.rules.0 {
      match init {
        AscriptRule::Struct(_, s) => {
          if let Some(strct) = adb.structs.get(&s.name) {
            s.has_token = strct.has_token;
          } else {
            panic!("Initializer's structure is not defined!");
          }
        }
        _ => {}
      }
    }

    adb.rules.push(ast_rule);
  }
}

/// Derives the AST types of all parser NonTerminals.
pub fn resolve_nonterm_types(db: &GrammarDatabase, adb: &mut AscriptDatabase) -> OrderedMap<DBNonTermKey, AscriptType> {
  let AscriptDatabase { errors, multi_type_lu, multi_types, .. } = adb;

  let mut resolved_nonterms = OrderedMap::new();

  let mut rule_to_nonterms = OrderedMap::new();
  let nonterms_to_rule = db.get_nonterm_symbol_to_rules();

  // Collect all non_term_dependency_of_a_rule

  for (i, _) in db.rules().iter().enumerate() {
    match &adb.rules.get(i) {
      Some(AscriptRule::Expression(_, node)) => {
        let item = Item::from((DBRuleKey::from(i), db));
        let set = rule_to_nonterms.entry(item.rule_id()).or_insert_with(|| OrderedSet::new());
        get_nonterm_refs(
          GraphResolveData {
            db,
            item,
            node: node.ast.as_ref().unwrap(),
            nonterm_types: &resolved_nonterms,
            default_nonterm_type: None,
          },
          set,
        )
      }
      Some(AscriptRule::LastSymbol(..)) => {
        let item = Item::from((DBRuleKey::from(i), db));
        let item = item.to_penultimate();
        match item.nonterm_index_at_sym(Default::default(), db) {
          Some(nonterm_key) => {
            let set = rule_to_nonterms.entry(item.rule_id()).or_insert_with(|| OrderedSet::new());
            set.insert(nonterm_key);
          }
          None => {
            rule_to_nonterms.entry(item.rule_id()).or_insert_with(|| OrderedSet::new());
          }
        }
      }
      Some(_) => {
        rule_to_nonterms.entry(DBRuleKey::from(i as u32)).or_insert_with(|| OrderedSet::new());
      }

      _ => {}
    }
  }

  // Start with rules that do not rely on non-term-values;
  let mut queue = VecDeque::from_iter(rule_to_nonterms.iter().filter_map(|(r, s)| s.is_empty().then_some(*r)));

  let max_iterations = db.rules().len().pow(2);
  let mut total_iterations = 0;

  while let Some(rule_in_process) = queue.pop_front() {
    total_iterations += 1;

    let nonterm_id = db.db_rule(rule_in_process).nonterm;

    if total_iterations > max_iterations {
      panic!(
        "Could not resolve rule {}",
        db.db_rule(rule_in_process).rule.tok.blame(1, 1, "could not resolve the AST of this rule", None)
      )
    }

    let rule_index: usize = rule_in_process.into();
    let item = Item::from((rule_in_process, db));

    let ty = match &adb.rules.get(rule_index) {
      Some(AscriptRule::Invalid(..)) => AscriptType::Undefined,
      Some(AscriptRule::Struct(_, id)) => AscriptType::Scalar(AscriptScalarType::Struct(
        id.name, false, //adb.structs.get(&id.name).map(|a| a.properties.len() == 0).unwrap_or_default(),
      )),
      Some(AscriptRule::LastSymbol(..)) => match graph_type_from_item(item.to_penultimate(), db, &resolved_nonterms, None) {
        (ty, _) => ty,
      },
      Some(AscriptRule::Expression(_, node)) => match get_graph_type(
        GraphResolveData {
          db,
          item,
          node: node.ast.as_ref().unwrap(),
          nonterm_types: &resolved_nonterms,
          default_nonterm_type: None,
        },
        &mut GraphMutData {
          selected_indices: &mut Default::default(),
          multi_indices:    multi_type_lu,
          multi_maps:       multi_types,
        },
        nonterm_id,
      ) {
        Ok(ty) => ty,
        Err(err) => {
          errors.push(err);
          continue;
        }
      },
      Some(AscriptRule::ListContinue(..)) | Some(AscriptRule::ListInitial(..)) => {
        match item.to_penultimate().nonterm_index_at_sym(Default::default(), db) {
          Some(nonterm_key) => match resolved_nonterms.get(&nonterm_key) {
            Some(ty) => match ty {
              AscriptType::Scalar(ty) => AscriptType::Aggregate(AscriptAggregateType::Vec { val_type: *ty }),
              agg @ AscriptType::Aggregate(..) => *agg,
              _ => AscriptType::Undefined,
            },
            None => AscriptType::Undefined,
          },
          None => AscriptType::Scalar(AscriptScalarType::Token),
        }
      }
      _ => unreachable!(""),
    };

    if ty.is_unknown() {
      queue.extend(rule_to_nonterms.iter().filter_map(|(r, s)| s.contains(&nonterm_id).then_some(*r)));
      queue.push_back(rule_in_process);
      continue;
    }

    let was_empty = resolved_nonterms.get(&nonterm_id).is_none();
    let existing_type = resolved_nonterms.entry(nonterm_id).or_insert(ty.clone());

    match get_resolved_type(existing_type, &ty, multi_type_lu, multi_types, nonterm_id) {
      Ok(resolved_type) => {
        if was_empty || *existing_type != resolved_type {
          queue.extend(rule_to_nonterms.iter().filter_map(|(r, s)| s.contains(&nonterm_id).then_some(*r)));

          *existing_type = resolved_type;

          if let Some(rules_to_update) = nonterms_to_rule.get(&nonterm_id) {
            for new_rule in rules_to_update {
              if *new_rule != rule_in_process {
                queue.push_back(*new_rule)
              }
            }
          }
        }
      }
      Err(err) => errors.push(err),
    }
  }

  resolved_nonterms
}

fn resolve_nonterm_values(
  adb: &mut AscriptDatabase,
  nonterm_types: &OrderedMap<DBNonTermKey, AscriptType>,
) -> Result<(), DBNonTermKey> {
  let AscriptDatabase { rules, multi_type_lu, multi_types, .. } = adb;
  let db = adb.db.clone();
  let db = db.as_ref();

  let mut struct_rules = OrderedMap::<StringId, OrderedSet<(usize, DBNonTermKey)>>::new();

  for (rule_index, ast_rule) in rules.iter_mut().enumerate() {
    let item = Item::from((DBRuleKey::from(rule_index), db));
    let mut selected_indices = HashSet::new();
    let selected_indices = &mut selected_indices;
    let rule_key = ast_rule.get_db_key();
    let nt_key: DBNonTermKey = db.rule_nonterm(rule_key);
    let expected_type = nonterm_types.get(&nt_key).unwrap();

    match ast_rule {
      AscriptRule::Expression(_, init) => {
        let mut node = create_graph_node(
          GraphResolveData {
            db,
            item,
            node: init.ast.as_ref().unwrap(),
            nonterm_types: &nonterm_types,
            default_nonterm_type: None,
          },
          &mut GraphMutData {
            selected_indices: selected_indices,
            multi_indices:    multi_type_lu,
            multi_maps:       multi_types,
          },
          nt_key,
        )?;

        if rule_index == 152 || rule_index == 151 || rule_index == 150 {
          println!("\n\n{rule_index} {node:?} {expected_type:?}");
        }

        let ty = *expected_type;
        let node_type = node.get_type();

        if ty.is_multi() {
          if ty.get_multi_type_index() != node_type.get_multi_type_index() {
            if node_type.is_vector() {
              debug_assert!(ty.is_vector());
              node = GraphNode::MultiConvert(Rc::new(node), ty)
            } else {
              node = GraphNode::MultiConvert(Rc::new(node), ty)
            }
          }
        }

        init.ty = ty;
        init.output_graph = Some(node);
      }
      AscriptRule::ListInitial(_, init) => {
        let Initializer { output_graph, ty, .. } = init;
        let last = match get_item_at_sym_ref(item, db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, db, &nonterm_types, selected_indices, None)?,
          None => GraphNode::Undefined(AscriptType::Undefined),
        };
        match last.get_type() {
          AscriptType::Aggregate(_) => {
            *ty = last.get_type().clone();
            *output_graph = Some(last);
          }
          _ => {
            if expected_type.is_multi() && last.get_type().get_multi_type_index() != expected_type.get_multi_type_index() {
              let last = GraphNode::Vec(
                GraphNodeVecInits(vec![GraphNode::MultiConvert(
                  Rc::new(last.clone()),
                  AscriptType::Scalar(AscriptScalarType::Multi(expected_type.get_multi_type_index().unwrap())),
                )]),
                *expected_type,
              );
              *ty = last.get_type().clone();
              *output_graph = Some(last);
            } else {
              match last.clone() {
                GraphNode::TokSym(..) | GraphNode::Tok(..) | GraphNode::TokRule(..) => {
                  *ty = last.get_type().clone();
                  *output_graph = Some(last);
                }
                last => {
                  let last = GraphNode::Vec(GraphNodeVecInits(vec![last.clone()]), *expected_type);
                  *ty = last.get_type().clone();
                  *output_graph = Some(last);
                }
              }
            }
          }
        };
      }
      AscriptRule::ListContinue(_, init) => {
        let Initializer { output_graph, ty, .. } = init;

        let mut first = match get_item_at_sym_ref(item, &adb.db, |_, _| true /* matches first item */) {
          Some(item) => graph_node_from_item(item, &adb.db, &nonterm_types, selected_indices, None)?,
          None => GraphNode::Undefined(AscriptType::Undefined),
        };
        let mut last = match get_item_at_sym_ref(item, &adb.db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, &adb.db, &nonterm_types, selected_indices, None)?,
          None => GraphNode::Undefined(AscriptType::Undefined),
        };

        dbg!(&ty, &first, &last, item._debug_string_w_db_(db));

        // debug_assert!(matches!(first.get_type(), AscriptType::Aggregate(..)));
        *ty = *expected_type;

        if ty.is_multi() {
          if ty.get_multi_type_index() != last.get_type().get_multi_type_index() {
            last = GraphNode::MultiConvert(Rc::new(last), *expected_type)
          }

          if ty.get_multi_type_index() != first.get_type().get_multi_type_index() {
            first = GraphNode::MultiConvert(Rc::new(first), *expected_type)
          }
        }

        let join = GraphNode::Add(Rc::new(first), Rc::new(last), *ty);

        *output_graph = Some(join);
      }
      AscriptRule::LastSymbol(_, init) => {
        let Initializer { output_graph, ty, .. } = init;

        let graph_node = match get_item_at_sym_ref(item, &adb.db, |item, _| item.is_penultimate()) {
          Some(item) => graph_node_from_item(item, &adb.db, &nonterm_types, selected_indices, None)?,
          None => GraphNode::Undefined(AscriptType::Undefined),
        };

        *ty = *graph_node.get_type();
        *output_graph = Some(graph_node);
      }
      AscriptRule::Struct(_, rule_struct) => {
        let struct_name: StringId = rule_struct.name;
        struct_rules.entry(struct_name).or_default().insert((rule_index, nt_key));
      }
      _r => todo!("handle rule {{_r:?}}"),
    }
  }

  Ok(())
}

fn get_item_at_sym_ref<'db, F: Fn(Item, &SymbolRef) -> bool>(item: Item, db: &'db GrammarDatabase, funct: F) -> Option<Item> {
  let mut i = item;
  while let Some(sym) = i.sym(db) {
    if funct(i, sym) {
      return Some(i);
    }
    i = i.try_increment();
  }
  None
}

fn graph_type_from_item(
  item: Item,
  db: &GrammarDatabase,
  nonterm_types: &std::collections::BTreeMap<DBNonTermKey, AscriptType>,
  default_type: Option<(DBNonTermKey, &AscriptType)>,
) -> (AscriptType, Option<DBNonTermKey>) {
  if let Some(nonterm_id) = item.nonterm_index_at_sym(Default::default(), db) {
    match nonterm_types.get(&nonterm_id) {
      Some(ty) => (*ty, Some(nonterm_id)),
      None => match default_type {
        Some((id, ty)) => {
          if id == nonterm_id {
            (*ty, Some(nonterm_id))
          } else {
            (AscriptType::Undefined, Some(nonterm_id))
          }
        }
        None => (AscriptType::Undefined, Some(nonterm_id)),
      },
    }
  } else {
    (AscriptType::Scalar(AscriptScalarType::Token), None)
  }
}

fn graph_node_from_item(
  item: Item,
  db: &GrammarDatabase,
  nonterm_types: &std::collections::BTreeMap<DBNonTermKey, AscriptType>,
  selected_indices: &mut HashSet<usize>,
  default_type: Option<(DBNonTermKey, &AscriptType)>,
) -> Result<GraphNode, DBNonTermKey> {
  let index = item.sym_index() as usize;
  if let Some(nonterm_id) = item.nonterm_index_at_sym(Default::default(), db) {
    match nonterm_types.get(&nonterm_id) {
      Some(ty) => Ok(GraphNode::Sym(index, selected_indices.insert(index), *ty)),
      None => match default_type {
        Some((id, ty)) => {
          if id == nonterm_id {
            Ok(GraphNode::Sym(index, selected_indices.insert(index), *ty))
          } else {
            Err(nonterm_id)
          }
        }
        None => Err(nonterm_id),
      },
    }
  } else {
    Ok(GraphNode::TokSym(index, selected_indices.insert(index), AscriptType::Scalar(AscriptScalarType::Token)))
  }
}

struct GraphMutData<'a> {
  selected_indices: &'a mut HashSet<usize>,
  multi_indices:    &'a mut Vec<usize>,
  multi_maps:       &'a mut Vec<MultiTypeRef>,
}

#[derive(Clone, Copy)]
struct GraphResolveData<'a> {
  item:                 Item,
  db:                   &'a GrammarDatabase,
  node:                 &'a ASTNode,
  nonterm_types:        &'a OrderedMap<DBNonTermKey, AscriptType>,
  default_nonterm_type: Option<(DBNonTermKey, &'a AscriptType)>,
}

impl<'a> GraphResolveData<'a> {
  pub fn to_node(self, node: &'a ASTNode) -> GraphResolveData<'a> {
    Self { node, ..self }
  }
}

fn get_nonterm_refs<'a>(args: GraphResolveData<'a>, nonterms: &mut OrderedSet<DBNonTermKey>) {
  match args.node {
    ASTNode::AST_NamedReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.annotation == rf.value.to_token()) {
        Some(item) => match graph_type_from_item(item, args.db, args.nonterm_types, args.default_nonterm_type) {
          (_, Some(nonterm)) => {
            nonterms.insert(nonterm);
          }
          _ => {}
        },
        None => {}
      }
    }

    ASTNode::AST_IndexReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.original_index as isize == (rf.value - 1) as isize) {
        Some(item) => match graph_type_from_item(item, args.db, args.nonterm_types, args.default_nonterm_type) {
          (_, Some(nonterm)) => {
            nonterms.insert(nonterm);
          }
          _ => {}
        },
        None => {}
      }
    }
    ASTNode::AST_Vector(vec) => {
      for node in vec.initializer.iter() {
        get_nonterm_refs(args.to_node(node), nonterms);
      }
    }

    ASTNode::AST_Map(map) => {
      get_nonterm_refs(args.to_node(&map.key), nonterms);
      get_nonterm_refs(args.to_node(&map.val), nonterms);
    }

    ASTNode::AST_Add(add) => {
      get_nonterm_refs(args.to_node(&add.left), nonterms);
      get_nonterm_refs(args.to_node(&add.right), nonterms);
    }
    ASTNode::AST_Mul(mul) => {
      get_nonterm_refs(args.to_node(&mul.left), nonterms);
      get_nonterm_refs(args.to_node(&mul.right), nonterms);
    }
    ASTNode::AST_Token(..)
    | ASTNode::AST_String(..)
    | ASTNode::AST_Bool(..)
    | ASTNode::AST_U8(..)
    | ASTNode::AST_U16(..)
    | ASTNode::AST_U32(..)
    | ASTNode::AST_U64(..)
    | ASTNode::AST_I8(..)
    | ASTNode::AST_I16(..)
    | ASTNode::AST_I32(..)
    | ASTNode::AST_I64(..)
    | ASTNode::AST_F32(..)
    | ASTNode::AST_F64(..) => {}
    #[cfg(debug_assertions)]
    node => todo!("handle graph type resolve of node {node:#?}"),
    #[cfg(not(debug_assertions))]
    _ => panic!("Unresolved node type"),
  };
}

fn get_graph_type<'a>(
  args: GraphResolveData<'a>,
  mut_args: &mut GraphMutData,
  nonterm: DBNonTermKey,
) -> RadlrResult<AscriptType> {
  let ty = match args.node {
    ASTNode::AST_NamedReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.annotation == rf.value.to_token()) {
        Some(item) => graph_type_from_item(item, args.db, args.nonterm_types, args.default_nonterm_type).0,
        None => AscriptType::Undefined,
      }
    }

    ASTNode::AST_IndexReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.original_index as isize == (rf.value - 1) as isize) {
        Some(item) => graph_type_from_item(item, args.db, args.nonterm_types, args.default_nonterm_type).0,
        None => AscriptType::Undefined,
      }
    }
    ASTNode::AST_Vector(vec) => {
      let mut ty = AscriptType::Undefined;

      for node in vec.initializer.iter() {
        let node = get_graph_type(args.to_node(node), mut_args, nonterm)?;
        ty = get_resolved_type(&node, &ty, mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();
      }

      let ty = if let Some(ty_) = ty.as_aggregate() {
        match ty_ {
          AscriptAggregateType::Map { val_type, .. } => AscriptType::Scalar(val_type),
          AscriptAggregateType::Vec { val_type } => AscriptType::Scalar(val_type),
        }
      } else {
        ty
      };

      if ty.is_unknown() {
        AscriptType::Undefined
      } else {
        AscriptType::Aggregate(AscriptAggregateType::Vec { val_type: ty.as_scalar().unwrap() })
      }
    }

    ASTNode::AST_Map(map) => {
      let key = get_graph_type(args.to_node(&map.key), mut_args, nonterm)?;
      let val = get_graph_type(args.to_node(&map.val), mut_args, nonterm)?;

      if key.is_unknown() || val.is_unknown() {
        AscriptType::Undefined
      } else {
        let ascript_type = AscriptType::Aggregate(AscriptAggregateType::Map {
          key_type: key.as_scalar().unwrap(),
          val_type: val.as_scalar().unwrap(),
        });

        ascript_type
      }
    }

    ASTNode::AST_Add(add) => {
      let l_ty = get_graph_type(args.to_node(&add.left), mut_args, nonterm)?;
      let r_ty = get_graph_type(args.to_node(&add.right), mut_args, nonterm)?;

      if l_ty.is_unknown() || r_ty.is_unknown() {
        AscriptType::Undefined
      } else if l_ty.is_numeric() && r_ty.is_numeric() {
        let l_scalar = l_ty.as_scalar().unwrap();
        let r_scalar = r_ty.as_scalar().unwrap();

        (l_scalar.precedence() > r_scalar.precedence()).then_some(l_ty).unwrap_or(r_ty)
      } else if l_ty.is_token() || r_ty.is_token() {
        if l_ty.is_string() || r_ty.is_string() {
          AscriptType::Scalar(AscriptScalarType::String(None))
        } else if l_ty.is_token() && r_ty.is_token() {
          AscriptType::Scalar(AscriptScalarType::Token)
        } else {
          AscriptType::Undefined
        }
      } else if l_ty.is_string() || r_ty.is_string() {
        AscriptType::Scalar(AscriptScalarType::String(None))
      } else {
        get_resolved_type(&l_ty, &r_ty, mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap()
      }
    }
    ASTNode::AST_Mul(mul) => {
      let l_ty = get_graph_type(args.to_node(&mul.left), mut_args, nonterm)?;
      let r_ty = get_graph_type(args.to_node(&mul.right), mut_args, nonterm)?;

      if l_ty.is_unknown() || r_ty.is_unknown() {
        AscriptType::Undefined
      } else if l_ty.is_numeric() && r_ty.is_numeric() {
        let l_scalar = l_ty.as_scalar().unwrap();
        let r_scalar = r_ty.as_scalar().unwrap();

        (l_scalar.precedence() > r_scalar.precedence()).then_some(l_ty).unwrap_or(r_ty)
      } else {
        AscriptType::Undefined
      }
    }
    ASTNode::AST_Token(..) => AscriptType::Scalar(AscriptScalarType::Token),
    ASTNode::AST_String(..) => AscriptType::Scalar(AscriptScalarType::String(None)),
    ASTNode::AST_Bool(..) => AscriptType::Scalar(AscriptScalarType::Bool(false)),
    ASTNode::AST_BoolLiteral(val) => AscriptType::Scalar(AscriptScalarType::Bool(val.value)),
    ASTNode::AST_U8(..) => AscriptType::Scalar(AscriptScalarType::U8(None)),
    ASTNode::AST_U16(..) => AscriptType::Scalar(AscriptScalarType::U16(None)),
    ASTNode::AST_U32(..) => AscriptType::Scalar(AscriptScalarType::U32(None)),
    ASTNode::AST_U64(..) => AscriptType::Scalar(AscriptScalarType::U64(None)),
    ASTNode::AST_I8(..) => AscriptType::Scalar(AscriptScalarType::I8(None)),
    ASTNode::AST_I16(..) => AscriptType::Scalar(AscriptScalarType::I16(None)),
    ASTNode::AST_I32(..) => AscriptType::Scalar(AscriptScalarType::I32(None)),
    ASTNode::AST_I64(..) => AscriptType::Scalar(AscriptScalarType::I64(None)),
    ASTNode::AST_F32(..) => AscriptType::Scalar(AscriptScalarType::F32(None)),
    ASTNode::AST_F64(..) => AscriptType::Scalar(AscriptScalarType::F64(None)),

    node => todo!("handle graph type resolve of node {node:#?}"),
  };
  Ok(ty)
}

/// Resolves AST expression nodes into AscriptGraph nodes.
///
/// # Invariant:
/// Any type incompatibilities will have been reported before we
/// start building graph nodes. We should be able to assign nodes with having to
/// verify whether there are type conflicts.
fn create_graph_node<'a>(
  args: GraphResolveData<'a>,
  mut_args: &mut GraphMutData,
  nonterm: DBNonTermKey,
) -> Result<GraphNode, DBNonTermKey> {
  match args.node {
    ASTNode::AST_Vector(vec) => {
      let mut ty = AscriptType::Undefined;
      let mut initializers = vec![];

      for node in vec.initializer.iter() {
        let node = create_graph_node(args.to_node(node), mut_args, nonterm)?;
        ty = get_resolved_type(node.get_type(), &ty, mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();

        if !node.get_type().is_unknown() {
          initializers.push(node);
        }
      }

      let vec_type = if let Some(ty_) = ty.as_aggregate() {
        match ty_ {
          AscriptAggregateType::Map { val_type, .. } => AscriptType::Scalar(val_type),
          AscriptAggregateType::Vec { val_type } => AscriptType::Scalar(val_type),
        }
      } else {
        ty
      };

      let val_type = match vec_type.as_scalar() {
        Some(scalar) => scalar,
        None => return Ok(GraphNode::Undefined(AscriptType::Undefined)),
      };

      let ascript_type = AscriptType::Aggregate(AscriptAggregateType::Vec { val_type });
      let undefined_init = initializers.iter().any(|i| i.get_type().is_unknown());

      debug_assert!(!undefined_init);
      debug_assert!(vec_type.as_scalar().is_some(), "Type: {ty:#?}");

      // create base node
      let mut left = None;
      let mut scalar_inits = GraphNodeVecInits(vec![]);

      for initializer in initializers.clone() {
        let multi_mismatch =
          ascript_type.is_multi() && initializer.get_type().get_multi_type_index() != ascript_type.get_multi_type_index();

        if initializer.get_type().is_vector() {
          let initialize = if multi_mismatch {
            Rc::new(GraphNode::MultiConvert(Rc::new(initializer), ascript_type))
          } else {
            Rc::new(initializer)
          };

          if let Some(left_old) = left.take() {
            left = Some(GraphNode::Add(Rc::new(left_old), initialize, ascript_type));
          } else {
            left = Some(GraphNode::Add(Rc::new(GraphNode::Vec(scalar_inits.clone(), ascript_type)), initialize, ascript_type));
          }
        } else if multi_mismatch {
          if let Some(left_old) = left.take() {
            left = Some(GraphNode::Add(
              Rc::new(left_old),
              Rc::new(GraphNode::MultiConvert(Rc::new(initializer), vec_type)),
              ascript_type,
            ));
          } else {
            scalar_inits.push(GraphNode::MultiConvert(Rc::new(initializer), ascript_type))
          }
        }
      }

      match left {
        Some(graph_node) => Ok(graph_node),
        None => Ok(GraphNode::Vec(GraphNodeVecInits(initializers), ascript_type)),
      }
    }

    ASTNode::AST_Map(map) => {
      let key = create_graph_node(args.to_node(&map.key), mut_args, nonterm)?;

      let val = create_graph_node(args.to_node(&map.val), mut_args, nonterm)?;

      let ascript_type = AscriptType::Aggregate(AscriptAggregateType::Map {
        key_type: key.get_type().as_scalar().unwrap(),
        val_type: val.get_type().as_scalar().unwrap(),
      });

      Ok(GraphNode::Map(Rc::new(key), Rc::new(val), ascript_type))
    }

    ASTNode::AST_NamedReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.annotation == rf.value.to_token()) {
        Some(item) => {
          Ok(graph_node_from_item(item, args.db, args.nonterm_types, mut_args.selected_indices, args.default_nonterm_type)?)
        }
        None => Ok(GraphNode::Undefined(AscriptType::Undefined)),
      }
    }

    ASTNode::AST_IndexReference(rf) => {
      match get_item_at_sym_ref(args.item, args.db, |_, sym| sym.original_index as isize == (rf.value - 1) as isize) {
        Some(item) => {
          graph_node_from_item(item, args.db, args.nonterm_types, mut_args.selected_indices, args.default_nonterm_type)
        }
        None => Ok(GraphNode::Undefined(AscriptType::Undefined)),
      }
    }

    ASTNode::AST_Token(tok) => {
      if let Some(range) = &tok.range {
        let node = GraphNode::TokRule(AscriptType::Scalar(AscriptScalarType::Token));

        Ok(GraphNode::Trim(
          Rc::new(node),
          range.start_trim as isize,
          range.end_trim as isize,
          AscriptType::Scalar(AscriptScalarType::Token),
        ))
      } else {
        Ok(GraphNode::TokRule(AscriptType::Scalar(AscriptScalarType::Token)))
      }
    }

    ASTNode::AST_String(str) => {
      if let Some(init) = &str.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Str(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::String(None))))
      } else {
        Ok(GraphNode::Str(None, AscriptType::Scalar(AscriptScalarType::String(None))))
      }
    }

    ASTNode::AST_Add(add) => {
      let l = create_graph_node(args.to_node(&add.left), mut_args, nonterm)?;
      let r = create_graph_node(args.to_node(&add.right), mut_args, nonterm)?;
      let ty = get_resolved_type(l.get_type(), r.get_type(), mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();

      Ok(GraphNode::Add(Rc::new(l), Rc::new(r), ty))
    }

    ASTNode::AST_Sub(sub) => {
      let l = create_graph_node(args.to_node(&sub.left), mut_args, nonterm)?;
      let r = create_graph_node(args.to_node(&sub.right), mut_args, nonterm)?;
      let ty = get_resolved_type(l.get_type(), r.get_type(), mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();
      let (l, r) = convert_binary_op_types(l, ty, r);
      Ok(GraphNode::Sub(Rc::new(l), Rc::new(r), ty))
    }

    ASTNode::AST_Mul(mul) => {
      let l = create_graph_node(args.to_node(&mul.left), mut_args, nonterm)?;
      let r = create_graph_node(args.to_node(&mul.right), mut_args, nonterm)?;
      let ty = get_resolved_type(l.get_type(), r.get_type(), mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();
      let (l, r) = convert_binary_op_types(l, ty, r);
      Ok(GraphNode::Mul(Rc::new(l), Rc::new(r), ty))
    }

    ASTNode::AST_Div(div) => {
      let l = create_graph_node(args.to_node(&div.left), mut_args, nonterm)?;
      let r = create_graph_node(args.to_node(&div.right), mut_args, nonterm)?;
      let ty = get_resolved_type(l.get_type(), r.get_type(), mut_args.multi_indices, mut_args.multi_maps, nonterm).unwrap();
      let (l, r) = convert_binary_op_types(l, ty, r);
      Ok(GraphNode::Div(Rc::new(l), Rc::new(r), ty))
    }

    ASTNode::AST_Pow(..) => {
      todo!("Create Pow Graph node")
    }

    ASTNode::AST_Mod(..) => {
      todo!("Create Mod Graph node")
    }

    ASTNode::AST_BoolLiteral(bool) => {
      let val = bool.value;
      Ok(GraphNode::Bool(None, AscriptType::Scalar(AscriptScalarType::Bool(val))))
    }

    ASTNode::AST_Bool(bool) => {
      if let Some(init) = &bool.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Bool(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::Bool(false))))
      } else {
        Ok(GraphNode::Bool(None, AscriptType::Scalar(AscriptScalarType::Bool(false))))
      }
    }
    ASTNode::AST_NumberLiteral(val) => Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::F64(Some(val.value))))),

    ASTNode::AST_U8(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::U8(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::U8(None))))
      }
    }

    ASTNode::AST_U16(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::U16(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::U16(None))))
      }
    }

    ASTNode::AST_U32(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::U32(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::U32(None))))
      }
    }

    ASTNode::AST_U64(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::U64(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::U64(None))))
      }
    }

    ASTNode::AST_I8(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::I8(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::I8(None))))
      }
    }

    ASTNode::AST_I16(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::I16(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::I16(None))))
      }
    }

    ASTNode::AST_I32(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::I32(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::I32(None))))
      }
    }

    ASTNode::AST_I64(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::I64(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::I64(None))))
      }
    }

    ASTNode::AST_F32(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::F32(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::F32(None))))
      }
    }

    ASTNode::AST_F64(val) => {
      if let Some(init) = &val.initializer {
        let gn = create_graph_node(args.to_node(&init.expression), mut_args, nonterm)?;
        Ok(GraphNode::Num(Some(Rc::new(gn)), AscriptType::Scalar(AscriptScalarType::F64(None))))
      } else {
        Ok(GraphNode::Num(None, AscriptType::Scalar(AscriptScalarType::F64(None))))
      }
    }
    ASTNode::AST_TrimmedReference(val) => {
      let gn = create_graph_node(args.to_node(&val.reference), mut_args, nonterm)?;

      if gn.get_type().is_unknown() {
        Ok(gn)
      } else {
        Ok(GraphNode::Trim(
          Rc::new(gn),
          val.range.start_trim as isize,
          val.range.end_trim as isize,
          AscriptType::Scalar(AscriptScalarType::Token),
        ))
      }
    }

    node => todo!("handle graph resolve of node {node:#?}"),
    #[cfg(not(debug_assertions))]
    _ => panic!("Unresolved node type"),
  }
}

fn convert_binary_op_types(l: GraphNode, ty: AscriptType, r: GraphNode) -> (GraphNode, GraphNode) {
  (
    (l.get_type().to_cardinal() != ty.to_cardinal()).then(|| GraphNode::Num(Some(Rc::new(l.clone())), ty)).unwrap_or(l),
    (r.get_type().to_cardinal() != ty.to_cardinal()).then(|| GraphNode::Num(Some(Rc::new(r.clone())), ty)).unwrap_or(r),
  )
}

/// Attempts to merge two different types into a single, compatible type.
///
/// Returns an Err if the the combination of types cannot be reasonably merged,
/// such as Vec<u32> and Vec<String>, a Struct and a String, or a Struct and a
/// Struct.
fn get_resolved_type(
  a: &AscriptType,
  b: &AscriptType,
  multi_i: &mut Vec<usize>,
  multi_m: &mut Vec<MultiTypeRef>,
  target_nonterm: DBNonTermKey,
) -> RadlrResult<AscriptType> {
  use AscriptAggregateType::*;
  use AscriptType::*;
  match b {
    Undefined => Ok(*a),
    Aggregate(b_agg) => match a {
      Aggregate(a_agg) => match b_agg {
        Map { key_type: b_key_type, val_type: b_val_type } => match a_agg {
          Map { key_type: a_key_type, val_type: a_val_type } => {
            let key_type = get_resolved_type(&Scalar(*a_key_type), &Scalar(*b_key_type), multi_i, multi_m, target_nonterm)?
              .as_scalar()
              .unwrap_or_default();
            let val_type = get_resolved_type(&Scalar(*a_val_type), &Scalar(*b_val_type), multi_i, multi_m, target_nonterm)?
              .as_scalar()
              .unwrap_or_default();
            Ok(Aggregate(Map { key_type, val_type }))
          }
          Vec { val_type: a_base_type } => {
            let base_type = get_resolved_type(&Scalar(*a_base_type), &Scalar(*b_val_type), multi_i, multi_m, target_nonterm)?
              .as_scalar()
              .unwrap_or_default();
            Ok(Aggregate(Vec { val_type: base_type }))
          }
        },
        Vec { val_type: b_base_type } => match a_agg {
          Map { val_type: a_val_type, .. } => {
            let base_type = get_resolved_type(&Scalar(*b_base_type), &Scalar(*a_val_type), multi_i, multi_m, target_nonterm)?
              .as_scalar()
              .unwrap_or_default();
            Ok(Aggregate(Vec { val_type: base_type }))
          }
          Vec { val_type: a_base_type } => {
            let base_type = get_resolved_type(&Scalar(*b_base_type), &Scalar(*a_base_type), multi_i, multi_m, target_nonterm)?
              .as_scalar()
              .unwrap_or_default();
            Ok(Aggregate(Vec { val_type: base_type }))
          }
        },
      },
      Scalar(_) => match b_agg {
        Vec { val_type: base_type } => Ok(Aggregate(Vec {
          val_type: get_resolved_type(a, &Scalar(*base_type), multi_i, multi_m, target_nonterm)?.as_scalar().unwrap_or_default(),
        })),
        Map { .. } => todo!("Issue invalid joining of scalar and map"),
      },

      Undefined => Ok(*b),
      _ => todo!("resolve different types {a:?} {b:?}"),
    },
    Scalar(b_scalar) => match a {
      Undefined => Ok(*b),

      Scalar(a_scalar) => {
        use BaseType::*;

        let a_base_type = BaseType::from(*a_scalar);
        let b_base_type = BaseType::from(*b_scalar);

        let types = if a_base_type < b_base_type {
          ((a_base_type, a_scalar, a), (b_base_type, b_scalar, b))
        } else {
          ((b_base_type, b_scalar, b), (a_base_type, a_scalar, a))
        };

        match types {
          (
            (_, _, AscriptType::Scalar(AscriptScalarType::Multi(multi_a))),
            (_, _, AscriptType::Scalar(AscriptScalarType::Multi(multi_b))),
          ) => {
            let a_index = multi_i[*multi_a];
            let b_index = multi_i[*multi_b];

            if a_index != b_index {
              let nt_a = multi_m[a_index].0;
              let nt_b = multi_m[b_index].0;

              if nt_a == target_nonterm || nt_b == target_nonterm {
                let (used_index, other) = if nt_a == target_nonterm { (a_index, b_index) } else { (b_index, a_index) };

                let other_index = AscriptScalarType::Multi(other);

                multi_m[used_index].1.insert(other_index);

                return Ok(AscriptType::Scalar(AscriptScalarType::Multi(used_index)));
              } else {
                create_multi(
                  [AscriptScalarType::Multi(a_index), AscriptScalarType::Multi(b_index)].into_iter(),
                  multi_i,
                  multi_m,
                  target_nonterm,
                )
              }
            } else {
              Ok(*a)
            }
          }
          ((_, _, AscriptType::Scalar(AscriptScalarType::Multi(multi))), (_, _, AscriptType::Scalar(scalar)))
          | ((_, _, AscriptType::Scalar(scalar)), (_, _, AscriptType::Scalar(AscriptScalarType::Multi(multi)))) => {
            let a_index = multi_i[*multi];

            if multi_m[a_index].0 == target_nonterm {
              multi_m[a_index].1.insert(*scalar);
              Ok(AscriptType::Scalar(AscriptScalarType::Multi(*multi)))
            } else {
              create_multi([*scalar, AscriptScalarType::Multi(*multi)].into_iter(), multi_i, multi_m, target_nonterm)
            }
          }

          ((_, _, a), (_, _, b)) if a == b => Ok(*a),
          ((a_base, a_scl, a), (b_base, b_scl, b)) if a_base == b_base && a_base != BaseType::Other => {
            if a_scl.byte_size() > b_scl.byte_size() {
              Ok(*a)
            } else {
              Ok(*b)
            }
          }
          ((Bool, a_scl, _), (Int, b_scl, _)) | ((Uint, a_scl, _), (Int, b_scl, _)) => {
            match (a_scl.byte_size() + 1).max(b_scl.byte_size()) {
              1..=2 => Ok(Scalar(AscriptScalarType::I16(None))),
              3..=4 => Ok(Scalar(AscriptScalarType::I32(None))),
              _ => Ok(Scalar(AscriptScalarType::I64(None))),
            }
          }
          ((Bool, a_scl, _), (Float, b_scl, _))
          | ((Uint, a_scl, _), (Float, b_scl, _))
          | ((Int, a_scl, _), (Float, b_scl, _)) => match (a_scl.byte_size() + 1).max(b_scl.byte_size()) {
            1..=4 => Ok(Scalar(AscriptScalarType::F32(None))),
            _ => Ok(Scalar(AscriptScalarType::F64(None))),
          },
          _ => create_multi([*a_scalar, *b_scalar].iter().cloned(), multi_i, multi_m, target_nonterm),
        }
      }
      Aggregate(a_gg) => match b_scalar {
        AscriptScalarType::Struct(..) => Err(RadlrError::StaticText("Incompatible Types")),
        _ => todo!("Resolve types agg_a:{a_gg:?} ty:{b_scalar:?}"),
      },
      _ => todo!("Resolve types scaler:{a:?} ty:{b:?}"),
    },
    _ => todo!("Resolve types ty:{a:?} ty:{b:?}"),
  }
}

fn create_multi<T: Iterator<Item = AscriptScalarType>>(
  scalars: T,
  multi_i: &mut Vec<usize>,
  multi_m: &mut Vec<MultiTypeRef>,
  ref_name: DBNonTermKey,
) -> Result<AscriptType, RadlrError> {
  if let Some((index, multi_m)) = multi_m.iter_mut().enumerate().find(|(_, i)| i.0 == ref_name) {
    multi_m.1.extend(scalars);
    return Ok(AscriptType::Scalar(AscriptScalarType::Multi(index)));
  } else {
    assert!(ref_name != DBNonTermKey::default());
    let types = BTreeSet::from_iter(scalars);

    let index = multi_i.len();

    multi_i.push(multi_m.len());
    multi_m.push((ref_name, types, Default::default()));

    return Ok(AscriptType::Scalar(AscriptScalarType::Multi(index)));
  }
}
