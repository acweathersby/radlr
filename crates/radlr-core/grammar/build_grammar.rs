#![allow(for_loops_over_fallibles)]
use super::{
  super::{
    parser::{self, ASTNode, Grammar},
    types::*,
  },
  utils::{get_symbol_details, SymbolData},
};
use crate::{
  grammar::utils::resolve_grammar_path,
  parser::{ast::escaped_from, NonTerminal_Symbol},
  types::error_types::{create_empty_rule_error, create_invalid_import_source_error},
  utils::create_u64_hash,
};
use radlr_rust_runtime::types::{bytecode::MatchInputType, Token};
use std::{collections::HashMap, hash::Hash, path::PathBuf, sync::Arc};

/// Temporary structure to host rule data during
/// construction.
pub struct RuleData<'a> {
  symbols: &'a [(usize, &'a ASTNode)],
  ast_ref: Option<ASTToken>,
}

/// Temporary structure to host non-terminal data during
/// construction.
pub struct NonTermData<'a> {
  root_nterm:       NonTermId,
  root_g_name:      IString,
  root_f_name:      IString,
  symbols:          &'a mut OrderedSet<SymbolId>,
  sub_nonterminals: &'a mut Array<Box<SubNonTerminal>>,
  rules:            &'a mut Array<Rule>,
}

impl<'b> NonTermData<'b> {
  /// Create a new [ProductionData] that references a different
  /// `rules` array, but keeps all other references the same
  /// as `self`.
  pub fn set_rules<'d>(&'d mut self, rules: &'d mut Array<Rule>) -> NonTermData<'d> {
    NonTermData {
      rules,
      root_g_name: self.root_g_name,
      root_f_name: self.root_f_name,
      root_nterm: self.root_nterm,
      symbols: &mut self.symbols,
      sub_nonterminals: &mut self.sub_nonterminals,
    }
  }
}

/// Intermediate structure to host grammar data during
/// construction.
pub struct GrammarData {
  pub id:             GrammarIdentities,
  pub imports:        Map<IString, GrammarIdentities>,
  pub exports:        Array<(IString, (NonTermId, Token))>,
  pub global_skipped: Array<ASTNode>,
  pub grammar:        Box<Grammar>,
}

pub fn convert_grammar_data_to_header(import_id: GrammarIdentities, g_data: GrammarData) -> Box<GrammarHeader> {
  let mut identity = import_id;
  identity.guid_name = g_data.id.guid_name;
  Box::new(GrammarHeader {
    identity,
    pub_nterms: g_data.exports.into_iter().collect(),
    imports: g_data.imports.values().map(|v| v.guid).collect(),
  })
}

/// Parse grammar string and create a Grammar AST.
pub fn parse_grammar(string_data: &str) -> RadlrResult<Box<Grammar>> {
  RadlrResult::Ok(Grammar::from_str(string_data)?)
}

/// Do an initial preparation of the grammar data.
pub fn create_grammar_data(
  grammar: Box<Grammar>,
  grammar_path: &PathBuf,
  string_store: &IStringStore,
) -> RadlrResult<GrammarData> {
  const EXTENSIONS: [&str; 3] = ["sg", "radlr", "hcg"];

  let source_dir = grammar_path.parent().map_or(PathBuf::default(), |p| p.to_owned());

  let mut imports = Map::default();
  let mut exports = Array::default();
  let mut skipped = Array::default();
  let mut name = grammar_path.file_stem().and_then(|d| d.to_str()).unwrap_or("default");

  for preamble in &grammar.preamble {
    match preamble {
      ASTNode::Import(import) => {
        let parser::Import { reference, uri, .. } = import.as_ref();
        let path = PathBuf::from(uri);

        match resolve_grammar_path(&path, &source_dir, &EXTENSIONS) {
          RadlrResult::Ok(path) => {
            imports.insert(reference.intern(string_store), GrammarIdentities::from_path(&path, string_store));
          }
          _ => {
            create_invalid_import_source_error(import, &path, &source_dir);
            imports.insert(reference.intern(string_store), GrammarIdentities::from_path(&path, string_store));
          }
        }
      }
      ASTNode::Export(export) => {
        // Saving exports to a temporary store until we are able extract import
        // information from the preambles, since some exported symbols may
        // be Production_Import_Symbol types.
        exports.push(export.clone());
      }
      ASTNode::Ignore(ignore) => {
        for sym in &ignore.symbols {
          skipped.push(sym.clone())
        }
      }
      ASTNode::Name(name_ast) => {
        name = name_ast.name.as_str();
      }
      node => {
        return Err(
          RadlrError::from_source_token(&node.to_token(), grammar_path.clone(), "Unsupported preamble", "Unsupported preamble", ErrorId(ErrorClass::Grammar, 99, "unsupported-preamble"), RadlrErrorSeverity::Warning)
        )
      }
    }
  }

  let mut g_data = GrammarData {
    id: GrammarIdentities {
      guid:      grammar_path.into(),
      guid_name: grammar_guid_name(name, grammar_path, string_store),
      name:      name.intern(string_store),
      path:      grammar_path.intern(string_store),
    },
    global_skipped: skipped,
    grammar,
    imports,
    exports: Default::default(),
  };

  // Once we have g_data, and more importantly g_data.imports, we can convert
  // our exports into ProductionIds.
  if exports.is_empty() != true {
    for export in exports {
      let nterm = get_nonterminal_id_from_ast_node(&g_data, &export.nonterminal, string_store)?;
      g_data.exports.push((export.reference.intern(string_store), (nterm, export.nonterminal.to_token())));
    }
  } else {
    // Use the fist declared non-terminal as the default entry
    for nterm in &g_data.grammar.rules {
      if nterm.as_TemplateRules().is_some() {
        continue;
      }
      if let Ok(nterm_id) = get_nonterminal_id_from_ast_node(&g_data, &nterm, string_store) {
        g_data.exports.push(("default".intern(string_store), (nterm_id, nterm.to_token())));
        break;
      }
    }
  }

  RadlrResult::Ok(g_data)
}

pub fn extract_nonterminals<'a>(
  g_data: &'a GrammarData,
  s_store: &IStringStore,
) -> RadlrResult<(Array<(Box<NonTerminal>, &'a ASTNode)>, Map<IString, Box<NonTerminalTemplate>>, Array<Box<CustomState>>)> {
  let mut nterms = Array::new();
  let mut parse_states = Array::new();
  let mut templates = Map::new();
  for prod_rule in &g_data.grammar.rules {
    match prod_rule {
      ASTNode::TemplateRules(node) => {
        let (guid_name, friendly_name) = nterm_names(&(node.name_sym.name.clone() + "_template"), &g_data.id, s_store);
        templates.insert(
          guid_name,
          Box::new(NonTerminalTemplate {
            guid_name,
            friendly_name,
            rules: node.rules.clone(),
            tok: node.tok.clone(),
            g_id: g_data.id.guid,
            templates: node.template_params.clone(),
          }),
        );
      }
      ASTNode::State(parse_state) => {
        let (guid_name, friendly_name) = nterm_names(parse_state.id.name.as_str(), &g_data.id, s_store);
        parse_states.push(Box::new(CustomState {
          id: NonTermId::from((g_data.id.guid, parse_state.id.name.as_str())),
          guid_name,
          friendly_name,
          g_id: g_data.id.guid,
          tok: parse_state.tok.clone(),
          state: parse_state.clone(),
          symbols: Default::default(),
          nterm_refs: Default::default(),
        }))
      }
      ASTNode::CFRules(box parser::CFRules { name_sym, rules: _, tok })
      | ASTNode::PegRules(box parser::PegRules { name_sym, rules: _, tok }) => {
        let (guid_name, f_name) = nterm_names(name_sym.name.as_str(), &g_data.id, s_store);
        let id = NonTermId::from((g_data.id.guid, name_sym.name.as_str()));
        nterms.push((create_bare_nonterm_struct(id, guid_name, f_name, &g_data.id, tok), prod_rule));
      }
      ASTNode::AppendRules(box parser::AppendRules { name_sym, rules: _, tok }) => {
        match get_nonterminal_symbol(g_data, &name_sym) {
          (Some(name_sym), _) => {
            let (guid_name, f_name) = nterm_names(name_sym.name.as_str(), &g_data.id, s_store);
            let id = NonTermId::from((g_data.id.guid, name_sym.name.as_str()));
            nterms.push((create_bare_nonterm_struct(id, guid_name, f_name, &g_data.id, tok), prod_rule));
          }
          (_, Some(name_sym)) => {
            let import_grammar_name = name_sym.module.to_string().intern(s_store);
            let import_g_id = o_to_r(g_data.imports.get(&import_grammar_name), "could not find grammar E")?; // <- TODO: Change this to a source error
            let (guid_name, f_name) = nterm_names(name_sym.name.as_str(), import_g_id, s_store);
            let id = NonTermId::from((import_g_id.guid, name_sym.name.as_str()));
            nterms.push((create_bare_nonterm_struct(id, guid_name, f_name, import_g_id, tok), prod_rule));
          }
          // Unreachable due to AppendRules guarantee that `name_sym`` is either NonTerm or ImportNonTerm
          _ => unreachable!(),
        };
      }
      node => return Err(
        RadlrError::from_source_token(&node.to_token(), g_data.id.path.to_path(s_store), "Unrecognized rule form", "Unrecognized rule form", ErrorId(ErrorClass::Grammar, 99, "unsupported-rule"), RadlrErrorSeverity::Critical)
      ),
    }
  }

  RadlrResult::Ok((nterms, templates, parse_states))
}

fn create_bare_nonterm_struct(
  id: NonTermId,
  guid_name: IString,
  f_name: IString,
  g_id: &GrammarIdentities,
  tok: &Token,
) -> Box<NonTerminal> {
  Box::new(NonTerminal {
    id,
    guid_name,
    friendly_name: f_name,
    g_id: g_id.guid,
    type_: NonTermType::ContextFree,
    rules: Default::default(),
    sub_nterms: Default::default(),
    symbols: Default::default(),
    tok_nterms: Default::default(),
    tok: tok.clone(),
    asts: Default::default(),
  })
}

pub fn process_parse_state<'a>(
  mut parse_state: Box<CustomState>,
  g_data: &GrammarData,
  s_store: &IStringStore,
) -> RadlrResult<Box<CustomState>> {
  // Extract symbol information from the state.

  fn process_state_nonterm(
    nonterminal: &ASTNode,
    g: &GrammarData,
    s: &IStringStore,
    sr: &mut std::collections::BTreeSet<(Token, IString, NonTermId)>,
  ) -> Result<IString, RadlrError> {
    let nonterm = &nonterminal;
    let (tok, g_id) = match get_nonterminal_symbol(g, nonterm) {
      (Some(nterm), None) => (&nterm.tok, &g.id),
      (None, Some(nterm)) => {
        let ref_name = nterm.module.to_token();
        match g.imports.get(&ref_name) {
          Some(id) => (&nterm.tok, id),
          _ => {
            return Err(RadlrError::Text(nterm.tok.blame(1, 1, "Could not resolve nonterm id from node", None)));
          }
        }
      }
      _ => return Err(
        RadlrError::from_source_token(&nonterminal.to_token(), g.id.path.to_path(s), "Could not resolve non-terminal id", "Could not resolve non-terminal id", ErrorId(ErrorClass::Grammar, 99, "unresolved-nonterm-id"), RadlrErrorSeverity::Critical)
      )
    };
    let name_str = tok.to_string();
    let (guid_name, _) = nterm_names(&name_str, g_id, s);
    sr.insert((tok.clone(), g_id.path, NonTermId::from((g_id.guid, name_str.as_str()))));
    Ok(guid_name)
  }

  fn process_statement(
    stmt: &mut Box<parser::Statement>,
    g: &GrammarData,
    s: &IStringStore,
    sr: &mut OrderedSet<(Token, IString, NonTermId)>,
  ) -> RadlrResult<()> {
    for n in &mut stmt.transitive {
      update_parser_state_node(g, s, n, sr)?;
    }
    for n in &mut stmt.branch {
      update_parser_state_node(g, s, n, sr)?;
    }
    Ok(if let Some(n) = stmt.transitive.as_mut() {
      update_parser_state_node(g, s, n, sr)?;
    })
  }

  fn update_parser_state_node(
    g: &GrammarData,
    s: &IStringStore,
    n: &mut ASTNode,
    sr: &mut OrderedSet<(Token, IString, NonTermId)>,
  ) -> RadlrResult<()> {
    match n {
      ASTNode::Gotos(gotos) => {
        for push in &mut gotos.pushes {
          let name = process_state_nonterm(&mut push.nonterminal, g, s, sr)?.to_string(s);
          push.name = name;
        }

        if let Some(goto) = gotos.goto.as_mut() {
          let name = process_state_nonterm(&goto.nonterminal, g, s, sr)?.to_string(s);
          goto.name = name;
        }

        if let Some(fork) = gotos.fork.as_mut() {
          for init in &mut fork.paths {
            let name = process_state_nonterm(&init.nonterminal, g, s, sr)?.to_string(s);
            init.name = name;
          }
        }
      }
      ASTNode::IntMatch(int_match) => {
        process_statement(&mut int_match.statement, g, s, sr)?;
      }
      ASTNode::NonTermMatch(nterm_match) => {
        process_statement(&mut nterm_match.statement, g, s, sr)?;
      }
      ASTNode::ProductionMatches(ast_match) => {
        for n in &mut ast_match.matches {
          update_parser_state_node(g, s, n, sr)?;
        }
      }
      ASTNode::TerminalMatches(ast_match) => {
        for n in &mut ast_match.matches {
          update_parser_state_node(g, s, n, sr)?;
        }
      }
      ASTNode::Matches(ast_match) => {
        // Convert name to one of our recognized types or warn about an invalid input
        // name.

        let internal_mode = match ast_match.mode.as_str() {
          "BYTE" => MatchInputType::BYTE_SCANLESS_STR,
          mode => return Err(RadlrError::Text("todo(anthony) : Invalid match mode error: ".to_string() + mode)),
        };

        ast_match.mode = internal_mode.into();

        for n in &mut ast_match.matches {
          update_parser_state_node(g, s, n, sr)?;
        }
      }
      ASTNode::Statement(stmt) => {
        process_statement(stmt, g, s, sr)?;
      }
      _ => {}
    }

    Ok(())
  }

  let mut nterm_refs = OrderedSet::new();

  process_statement(&mut parse_state.state.statement, g_data, s_store, &mut nterm_refs)?;

  parse_state.nterm_refs = nterm_refs;

  RadlrResult::Ok(parse_state)
}

pub fn process_nonterminals<'a>(
  (mut nterm, nterm_ast): (Box<NonTerminal>, &'a ASTNode),
  g_data: &GrammarData,
  s_store: &IStringStore,
  templates: &Map<IString, Box<NonTerminalTemplate>>,
  resolved: &mut Map<NonTermId, Option<Box<NonTerminal>>>,
) -> RadlrResult<Box<NonTerminal>> {
  let (type_, rules) = match nterm_ast {
    ASTNode::CFRules(nterm) => (NonTermType::ContextFree, &nterm.rules),
    ASTNode::AppendRules(nterm) => (NonTermType::ContextFree, &nterm.rules),
    ASTNode::PegRules(nterm) => (NonTermType::Peg, &nterm.rules),
    _ast => {
      #[cfg(debug_assertions)]
      todo!("Create build for {_ast:?}");
      #[cfg(not(debug_assertions))]
      todo!()
    }
  };

  nterm.type_ = type_;

  for rule in rules {
    process_rule(&mut nterm, rule, g_data, s_store, templates, resolved)?;
  }

  for rule in &nterm.rules {
    if rule.symbols.is_empty() {
      return Err(create_empty_rule_error(rule, s_store));
    }
  }

  RadlrResult::Ok(nterm)
}

fn process_rule(
  nterm: &mut NonTerminal,
  rule: &parser::Rule,
  g_data: &GrammarData,
  s_store: &IStringStore,
  templates: &Map<IString, Box<NonTerminalTemplate>>,
  resolved: &mut Map<NonTermId, Option<Box<NonTerminal>>>,
) -> RadlrResult<()> {
  let ast_syms = rule.symbols.iter().enumerate().collect::<Array<_>>();

  let NonTerminal { id, rules, sub_nterms, symbols, .. } = nterm;
  let mut nterm_data = NonTermData {
    root_nterm: *id,
    symbols,
    sub_nonterminals: sub_nterms,
    rules,
    root_g_name: nterm.guid_name,
    root_f_name: nterm.friendly_name,
  };

  let ast_ref = intern_ast(rule, &mut nterm_data);

  process_rule_symbols(
    &RuleData { symbols: &ast_syms, ast_ref },
    &mut nterm_data,
    g_data,
    s_store,
    rule.tok.clone(),
    templates,
    resolved,
  )?;

  RadlrResult::Ok(())
}

fn process_rule_symbols(
  rule_data: &RuleData,
  p_data: &mut NonTermData,
  g_data: &GrammarData,
  s_store: &IStringStore,
  tok: Token,
  tmpl: &Map<IString, Box<NonTerminalTemplate>>,
  resolved: &mut Map<NonTermId, Option<Box<NonTerminal>>>,
) -> RadlrResult<()> {
  let mut rules: Array<Rule> = Default::default();

  rules.push(Rule {
    symbols: Default::default(),
    skipped: Default::default(),
    ast:     rule_data.ast_ref.clone(),
    tok:     tok.clone(),
    g_id:    g_data.id,
  });

  for (index, sym) in rule_data.symbols.iter() {
    let original_bodies = 0..rules.len();
    let SymbolData {
      annotation, is_optional, symbol_precedence, token_precedence, sym_atom, ..
    } = get_symbol_details(sym);

    // If the symbol is optional, then we create new set
    // of rule symbols that will not contain the optional
    // symbol.
    is_optional.then(|| rules.append(&mut rules.clone()));

    let (sym, tok) = match sym_atom.unwrap() {
      ASTNode::NotEmptySet(set) => {
        let mut pending_rules = vec![];

        fn get_index_permutations(indexes: Array<usize>) -> Array<Array<usize>> {
          if indexes.len() > 1 {
            let mut out = vec![];
            for (i, candidate) in indexes.iter().enumerate() {
              let mut remainder = indexes.clone();
              remainder.remove(i);
              for mut permutation in get_index_permutations(remainder) {
                permutation.insert(0, *candidate);
                out.push(permutation)
              }
            }
            out
          } else {
            vec![indexes]
          }
        }

        let indices = set.symbols.iter().enumerate().map(|(i, _)| i).collect::<Vec<_>>();

        if indices.len() > 5 && set.unordered {
          todo!("Create error/warning about factorial explosions with `[]` sets greater than 5: (6+)! combinatorial explosion")
        }

        let candidate_symbols = set.symbols.iter().enumerate().map(|(i, s)| (i + index, s)).collect::<Array<_>>();

        for permutation in if set.unordered { get_index_permutations(indices) } else { vec![indices] } {
          let symbols = permutation.iter().map(|i| candidate_symbols[*i]).collect::<Array<_>>();

          process_rule_symbols(
            &RuleData { symbols: &symbols, ast_ref: None },
            &mut p_data.set_rules(&mut pending_rules),
            g_data,
            s_store,
            tok.clone(),
            tmpl,
            resolved,
          )?;
        }

        let mut new_rules = Array::new();
        let mut rule_id = Set::new();

        for pending_rule in pending_rules {
          // Prevent empty rules from being created, unless all permutations are allowed.
          if pending_rule.symbols.is_empty() && !set.allow_empty {
            continue;
          }

          // Prevent duplicate rules from being created.
          if !rule_id.insert(create_u64_hash(&pending_rule.symbols)) {
            continue;
          }

          for rule in &mut rules[original_bodies.clone()] {
            let mut new_rule = rule.clone();
            new_rule.symbols.extend(pending_rule.symbols.iter().cloned());
            new_rules.push(new_rule)
          }
        }

        rules = new_rules;

        // This rule has been replaced with the derived rules, so
        // it no longer needs to be processed, continuing on to
        // the next iteration.
        continue;
      }

      ASTNode::Grouped_Rules(group) => {
        let mut sub_nterm_rules = Default::default();

        for rule in &group.rules {
          let n_rule = &RuleData {
            symbols: &rule.symbols.iter().enumerate().collect::<Array<_>>(),
            ast_ref: intern_ast(rule, p_data),
          };
          process_rule_symbols(
            n_rule,
            &mut p_data.set_rules(&mut sub_nterm_rules),
            g_data,
            s_store,
            rule.tok.clone(),
            tmpl,
            resolved,
          )?
        }

        let nterm = p_data.root_nterm;
        let index = p_data.sub_nonterminals.len();
        let id = NonTermId::from((nterm, index));

        let (guid_name, friendly_name) = sub_nterm_names("group", p_data, s_store);

        p_data.sub_nonterminals.push(Box::new(SubNonTerminal {
          id,
          guid_name,
          friendly_name,
          type_: SubNonTermType::Group,
          g_id: g_data.id.guid,
          rules: sub_nterm_rules,
        }));

        (id.as_sym(), group.tok.clone())
      }

      node @ ASTNode::Template_NonTerminal_Symbol(template_sym) => {
        // Replace the template symbol with the resolved non-terminal
        if let (Some(name_sym), _) = get_nonterminal_symbol(g_data, node) {
          let (guid_name, _) = nterm_names(&(name_sym.name.clone() + "_template"), &g_data.id, s_store);

          if let Some(template) = tmpl.get(&guid_name) {
            let hash = create_u64_hash(&template_sym.template_args);
            let name = name_sym.name.clone() + "_template_" + &hash.to_string();
            let (guid_name, f_name) = nterm_names(name.as_str(), &g_data.id, s_store);
            let id = NonTermId::from((g_data.id.guid, name.as_str()));

            if resolved.get(&id).is_none() {
              // There are two types of params in Template nodes:
              // - TemplateASTType params are used to set the names of ast struct definitions
              //   to keep AScripT happy.
              // - TemplateSym params replace ANY nonterminal symbol in the rule bodies that
              //   match the same name.

              // We first need to check whether the template instance arg signature is
              // compatible with our Template param signature
              let param_sig = template
                .templates
                .iter()
                .map(|t| match t {
                  ASTNode::TemplateASTType(..) => 1,
                  ASTNode::TemplateSym(..) => 2,
                  _ => unreachable!(),
                })
                .collect::<Vec<_>>();

              let arg_sig = template_sym
                .template_args
                .iter()
                .map(|t| match t {
                  ASTNode::AST_STRUCT_TEMPLATE_NAME(..) => 1,
                  _ => 2,
                })
                .collect::<Vec<_>>();

              if param_sig != arg_sig {
                todo!("Create error for incompatible template signatures {param_sig:?} | {arg_sig:?}")
              }

              // We use two different lookup tables to modify our templates rules.

              let (symbol_lookup, ast_struct_lookup) = Map::from_iter(
                template
                  .templates
                  .iter()
                  .map(|d| match d {
                    ASTNode::TemplateASTType(d) => d.val.clone(),
                    ASTNode::TemplateSym(d) => d.val.clone(),
                    _ => unreachable!(),
                  })
                  .zip(template_sym.template_args.iter()),
              )
              .into_iter()
              .partition::<HashMap<_, _>, _>(|(_, a)| !matches!(a, ASTNode::AST_STRUCT_TEMPLATE_NAME(..)));

              resolved.insert(id, None);

              let node = ASTNode::CFRules(Box::new(crate::parser::CFRules::new(
                Box::new(name_sym.clone()),
                template
                  .rules
                  .iter()
                  .map(|r| {
                    let mut rule = r.clone();
                    rule.symbols = rule.symbols.iter().cloned().map(|s| replace_template_symbol(s, &symbol_lookup)).collect();
                    rule.ast = replace_template_ast(rule.ast, &ast_struct_lookup);
                    rule
                  })
                  .collect(),
                template.tok.clone(),
              )));

              let nonterm = (create_bare_nonterm_struct(id, guid_name, f_name, &g_data.id, &tok), &node);

              let nt = process_nonterminals(nonterm, g_data, s_store, tmpl, resolved)?;

              resolved.insert(id, Some(nt));
            }

            (id.as_sym(), template_sym.tok.clone())
          } else {
            unreachable!("Can't resolve template")
          }
        } else {
          unreachable!()
        }
      }

      ASTNode::TokenGroupRules(tk_group) => {
        let hash = create_u64_hash(&tk_group.rules);
        let name = "tk_group_".to_string() + &hash.to_string();
        let (guid_name, f_name) = nterm_names(name.as_str(), &g_data.id, s_store);
        let id = NonTermId::from((g_data.id.guid, name.as_str()));

        if resolved.get(&id).is_none() {
          // Create a new production for this group
          resolved.insert(id, None);

          let node = ASTNode::CFRules(Box::new(crate::parser::CFRules::new(
            Box::new(NonTerminal_Symbol::new(name, tk_group.tok.clone())),
            tk_group.rules.clone(),
            tk_group.tok.clone(),
          )));

          let nonterm = (create_bare_nonterm_struct(id, guid_name, f_name, &g_data.id, &tok), &node);

          let nt = process_nonterminals(nonterm, g_data, s_store, tmpl, resolved)?;

          resolved.insert(id, Some(nt));
        }

        (id.as_tok_sym(), tk_group.tok.clone())
      }

      ASTNode::List_Rules(box parser::List_Rules { symbol, terminal_symbol, tok, .. }) => {
        let mut rule_syms = vec![symbol.clone()];
        let mut rules = Default::default();

        if let Some(terminal_symbol) = terminal_symbol {
          rule_syms.insert(0, terminal_symbol.clone());
        }

        let r_data = &RuleData {
          symbols: &map_symbols_to_indexed(&rule_syms),
          ast_ref: Some(ASTToken::ListEntry(symbol.to_token().get_tok_range())),
        };

        process_rule_symbols(
          r_data,
          &mut p_data.set_rules(&mut rules),
          g_data,
          s_store,
          symbol.to_token().clone(),
          tmpl,
          resolved,
        )?;

        let nterm = p_data.root_nterm;
        let nterm_index = p_data.sub_nonterminals.len();
        let id = NonTermId::from((nterm, nterm_index));
        let sym = id.as_sym();

        let mut secondary_rules = rules.clone();

        // Add the ProductionID as a non-term symbol
        // to the beginning of each rule, making the
        // resulting list non-terminal left recursive.
        for (index, rule) in secondary_rules.iter_mut().enumerate() {
          rule.ast = Some(ASTToken::ListIterate(tok.get_tok_range()));

          rule.tok = tok.clone();

          rule.symbols.insert(0, SymbolRef {
            id: sym,
            annotation: annotation.intern(s_store),
            loc: tok.clone(),
            original_index: 0,
            symbol_precedence: index as u16,
            ..Default::default()
          });

          for (i, SymbolRef { original_index: index, .. }) in rule.symbols.iter_mut().enumerate() {
            *index = i as u32;
          }
        }

        if terminal_symbol.is_some() {
          for rule in &mut rules {
            rule.symbols.remove(0);
            for (i, SymbolRef { original_index: index, .. }) in rule.symbols.iter_mut().enumerate() {
              *index = i as u32;
            }
          }
        }

        rules.append(&mut secondary_rules);

        let (guid_name, friendly_name) = sub_nterm_names("list", p_data, s_store);

        p_data.sub_nonterminals.push(Box::new(SubNonTerminal {
          type_: SubNonTermType::List,
          guid_name,
          friendly_name,
          id,
          g_id: g_data.id.guid,
          rules,
        }));

        (sym, tok.clone())
      }
      sym_atom => (record_symbol(sym, p_data, g_data, s_store)?, sym_atom.to_token()),
    };

    for rule in &mut rules[original_bodies] {
      rule.symbols.push(SymbolRef {
        id: sym,
        annotation: annotation.intern(s_store),
        loc: tok.clone(),
        original_index: *index as u32,
        token_precedence,
        symbol_precedence,
      });
    }
  }

  let skipped_symbols = g_data
    .global_skipped
    .iter()
    .map(|sym_node| {
      let Ok(data) = record_symbol(sym_node, p_data, g_data, s_store) else {
        panic!("Could not record symbol");
      };
      data
    })
    .collect::<Array<_>>();

  for rule in &mut rules {
    rule.skipped = skipped_symbols.clone();
  }

  p_data.rules.append(&mut rules);

  RadlrResult::Ok(())
}

fn replace_template_ast(s: Option<Box<parser::Ascript>>, symbol_lookup: &Map<String, &ASTNode>) -> Option<Box<parser::Ascript>> {
  let root_name = symbol_lookup.get("t_TypeNamePrefix");
  match &s {
    Some(s) => {
      let mut output = s.clone();
      match &mut output.ast {
        ASTNode::AST_Struct(s) => {
          if let Some(new_type) = symbol_lookup.get(&s.ty) {
            if let ASTNode::AST_STRUCT_TEMPLATE_NAME(d) = new_type {
              s.ty = d.typ.clone();
            }
          } else if let Some(ASTNode::AST_STRUCT_TEMPLATE_NAME(root_name)) = root_name {
            let ty = root_name.typ.clone();
            s.ty = ty + &s.ty[2..];
          }
        }
        _ => {}
      }

      Some(output)
    }
    None => None,
  }
}

fn replace_template_symbol(s: ASTNode, symbol_lookup: &Map<String, &ASTNode>) -> ASTNode {
  match s {
    ASTNode::AnnotatedSymbol(mut s) => {
      s.symbol = replace_template_symbol(s.symbol.clone(), symbol_lookup);
      ASTNode::AnnotatedSymbol(s)
    }
    ASTNode::List_Rules(mut s) => {
      s.symbol = replace_template_symbol(s.symbol.clone(), symbol_lookup);
      ASTNode::List_Rules(s)
    }
    ASTNode::Grouped_Rules(mut group) => {
      for rule in &mut group.rules {
        rule.symbols = rule.symbols.iter().map(|s| replace_template_symbol(s.clone(), symbol_lookup)).collect();
      }
      ASTNode::Grouped_Rules(group)
    }
    ASTNode::Template_NonTerminal_Symbol(mut sym) => {
      sym.template_args = sym.template_args.iter().map(|s| replace_template_symbol(s.clone(), symbol_lookup)).collect();
      ASTNode::Template_NonTerminal_Symbol(sym)
    }
    ASTNode::NonTerminal_Symbol(nt) => {
      if let Some(sym) = symbol_lookup.get(&nt.name) {
        (*sym).clone()
      } else {
        ASTNode::NonTerminal_Symbol(nt)
      }
    }
    s => s,
  }
}

fn intern_ast(rule: &parser::Rule, _p_data: &mut NonTermData) -> Option<ASTToken> {
  let ast_ref = rule.ast.as_ref().map(|s| ASTToken::Defined(Arc::new(*s.clone())));
  ast_ref
}

fn map_symbols_to_indexed(ast_syms: &Array<ASTNode>) -> Array<(usize, &ASTNode)> {
  ast_syms.iter().enumerate().map(|(i, s)| (i, s)).collect::<Array<_>>()
}

/// Converts a Symbol AST node into a symbol object and
/// records the symbol in the grammar store.
fn record_symbol(
  sym_node: &ASTNode,
  n_data: &mut NonTermData,
  g_data: &GrammarData,
  s_store: &IStringStore,
) -> RadlrResult<SymbolId> {
  let id = match sym_node {
    ASTNode::AnnotatedSymbol(annotated) => record_symbol(&annotated.symbol, n_data, g_data, s_store)?,

    ASTNode::TerminalToken(terminal) => {
      let string = escaped_from((&terminal.val).into())?.join("");
      let val = string.intern(s_store);
      let id = SymbolId::Token { val };

      id
    }

    ASTNode::EOFSymbol(_) => SymbolId::EndOfFile {},

    ASTNode::ClassSymbol(gen) => match gen.val.as_str() {
      "sp" => SymbolId::ClassSpace,
      "tab" => SymbolId::ClassHorizontalTab,
      "nl" => SymbolId::ClassNewLine,
      "id" => SymbolId::ClassIdentifier,
      "num" => SymbolId::ClassNumber,
      "sym" => SymbolId::ClassSymbol,
      "any" => SymbolId::Any,
      _ => {
        return Err(
          RadlrError::from_source_token(&gen.tok, g_data.id.path.to_path(s_store), "Unrecognized character class", "Unrecognized character class", ErrorId(ErrorClass::Grammar, 99, "unrecognized-char-class"), RadlrErrorSeverity::Critical)
        )
      }
    },

    ASTNode::NonTerminal_Symbol(_) | ASTNode::NonTerminal_Import_Symbol(_) => {
      let id = get_nonterminal_id_from_ast_node(g_data, sym_node, s_store)?.as_sym();
      // Bypass the registration of this nonterminal as a symbol.
      return RadlrResult::Ok(id);
    }

    ASTNode::NonTerminal_Terminal_Symbol(token_prod) => {
      let id = get_nonterminal_id_from_ast_node(g_data, &token_prod.nonterminal, s_store)?.as_tok_sym();
      // Bypass the registration of this nonterminal as a symbol.
      return RadlrResult::Ok(id);
    }

    sym_node => {
      return Err(
        RadlrError::from_source_token(&sym_node.to_token(), g_data.id.path.to_path(s_store), "Unrecognized symbol", "Unrecognized symbol", ErrorId(ErrorClass::Grammar, 98, "unrecognized-symbol"), RadlrErrorSeverity::Critical)
      )
    }
  };

  n_data.symbols.insert(id);

  RadlrResult::Ok(id)
}

/// Returns `(None, Some(&Production_Import_Symbol))` or
/// `(Some(&Production_Symbol), None)`from a valid non-terminal like node.
///
/// Valid Node:
/// - [ASTNode::Production_Import_Symbol]
/// - [ASTNode::Production_Symbol]
/// - [ASTNode::Production_Terminal_Symbol]
/// - [ASTNode::State]
/// - [ASTNode::PrattProduction]
/// - [ASTNode::PegProduction]
/// - [ASTNode::CFProduction]
fn get_nonterminal_symbol<'a>(
  g_data: &GrammarData,
  node: &'a ASTNode,
) -> (Option<&'a parser::NonTerminal_Symbol>, Option<&'a parser::NonTerminal_Import_Symbol>) {
  match node {
    ASTNode::NonTerminal_Import_Symbol(prod_import) => (None, Some(prod_import.as_ref())),
    ASTNode::NonTerminal_Symbol(prod_sym) => (Some(prod_sym.as_ref()), None),
    ASTNode::NonTerminal_Terminal_Symbol(prod_tok) => get_nonterminal_symbol(g_data, &prod_tok.nonterminal),
    ASTNode::State(box parser::State { id, .. })
    | ASTNode::PegRules(box parser::PegRules { name_sym: id, .. })
    | ASTNode::CFRules(box parser::CFRules { name_sym: id, .. }) => (Some(id.as_ref()), None),
    ASTNode::AppendRules(r) => get_nonterminal_symbol(g_data, &r.name_sym),
    ASTNode::Template_NonTerminal_Symbol(sym) => get_nonterminal_symbol(g_data, &sym.name),
    ASTNode::TemplateRules(sym) => (Some(sym.name_sym.as_ref()), None),
    #[cfg(debug_assertions)]
    _ => (None, None),
  }
}

/// Return the `ProductionId` from a valid non-terminal like node.
///
/// Valid Node:
/// - [ASTNode::Production_Import_Symbol]
/// - [ASTNode::Production_Symbol]
/// - [ASTNode::Production_Terminal_Symbol]
/// - [ASTNode::State]
/// - [ASTNode::PrattProduction]
/// - [ASTNode::PegProduction]
/// - [ASTNode::CFProduction]`
fn get_nonterminal_id_from_ast_node(g_data: &GrammarData, node: &ASTNode, s_store: &IStringStore) -> RadlrResult<NonTermId> {
  match get_nonterminal_symbol(g_data, node) {
    (Some(nterm), None) => Ok(NonTermId::from((g_data.id.guid, nterm.name.as_str()))),
    (None, Some(nterm)) => {
      let ref_name = nterm.module.to_token();

      match g_data.imports.get(&ref_name) {
        Some(GrammarIdentities { guid, .. }) => Ok(NonTermId::from((*guid, nterm.name.as_str()))),
        _ => {
          return Err(
            RadlrError::from_source_token(&nterm.tok, g_data.id.path.to_path(s_store), &format!(
              "in {}",
              nterm.tok.path_ref(&g_data.id.path.to_path(s_store)),
            ), "Could not resolve nonterm id from symbol", ErrorId(ErrorClass::Grammar, 98, "unresolved-nonterm"), RadlrErrorSeverity::Critical)
          )
        }
      }
    }
    _ => {
      return Err(
        RadlrError::from_source_token(&node.to_token(), g_data.id.path.to_path(s_store), "Symbol is not a valid Non-terminal", "Symbol is not a valid Non-terminal", ErrorId(ErrorClass::Grammar, 98, "invalid-nonterm"), RadlrErrorSeverity::Critical)
      )
    }
  }
}

// NAMES ------------------------------------------------------------------------

/// Creates a globally unique name and friendly name for a non-terminal
pub fn nterm_names(base_name: &str, g_data: &GrammarIdentities, s_store: &IStringStore) -> (IString, IString) {
  ((g_data.guid_name.to_string(s_store) + "____" + base_name).intern(s_store), base_name.intern(s_store))
}

/// Creates a globally unique name and friendly name for a sub-non-terminal.
/// `sub_name` should be a string indicating the type of symbol that
/// the sub-non-terminal was derived from.
fn sub_nterm_names(sub_name: &str, p_data: &NonTermData, s_store: &IStringStore) -> (IString, IString) {
  if p_data.sub_nonterminals.is_empty() {
    (
      (p_data.root_g_name.to_string(s_store) + "_" + sub_name).intern(s_store),
      (p_data.root_f_name.to_string(s_store) + "_" + sub_name).intern(s_store),
    )
  } else {
    (
      (p_data.root_g_name.to_string(s_store) + "_" + sub_name + "_" + &p_data.sub_nonterminals.len().to_string()).intern(s_store),
      (p_data.root_f_name.to_string(s_store) + "_" + sub_name + "_" + &p_data.sub_nonterminals.len().to_string()).intern(s_store),
    )
  }
}
/// Creates a globally unique name for a grammar
fn grammar_guid_name(name: &str, grammar_path: &PathBuf, string_store: &IStringStore) -> IString {
  (name.to_string() + "_" + &to_base64_name(grammar_path)).intern(string_store)
}

fn to_base64_name<T: Hash>(val: T) -> String {
  let mut string = Vec::new();

  let val = create_u64_hash(val);

  for i in 0..4 {
    let val = (val >> (i * 6)) & 0x3E;
    let ascii_base = if val < 10 {
      val + 48 // ASCII numbers
    } else if val < 36 {
      val - 10 + 65 // Uppercase ASCII letters
    } else if val < 62 {
      val - 36 + 97 // Lowercase ASCII letters
    } else {
      95 // ASCII underscore
    };
    string.push(ascii_base as u8);
  }

  String::from_utf8(string).unwrap()
}

/// Remove artifacts related to a single grammar from a soup through mutation.
pub fn remove_grammar_mut(id: GrammarId, soup: &mut GrammarSoup) -> RadlrResult<()> {
  let GrammarSoup { grammar_headers, nonterminals, custom_states, .. } = soup;

  {
    let mut grammar_headers = grammar_headers.write()?;

    if grammar_headers.remove(&id).is_none() {
      return RadlrResult::Ok(());
    }
  }

  {
    let mut nonterminals = nonterminals.write()?;

    let nonterminals_temp = nonterminals.drain(..).collect::<Vec<_>>();

    nonterminals.extend(nonterminals_temp.into_iter().filter(|p| p.g_id != id))
  }

  {
    let mut custom_states = custom_states.write()?;

    let custom_states_temp = custom_states.drain().collect::<Vec<_>>();

    custom_states.extend(custom_states_temp.into_iter().filter(|(_, s)| s.g_id != id))
  }

  RadlrResult::Ok(())
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use crate::{parser::Grammar, types::*};

  fn create_test_data(input: &str) -> RadlrResult<(Box<Grammar>, PathBuf, IStringStore)> {
    RadlrResult::Ok((super::parse_grammar(input)?, PathBuf::from("/test.sg"), IStringStore::default()))
  }
  #[test]
  fn extract_nonterminals() -> RadlrResult<()> {
    let (g, path, s_store) = create_test_data(r##"  <> test-sweet-home-alabama > c:id{3} | ("test"{2} "2" :ast $1 ) :ast $1 "##)?;

    let g_data = super::create_grammar_data(g, &path, &s_store)?;

    let (mut nterms, templates, ..) = super::extract_nonterminals(&g_data, &s_store)?;

    assert_eq!(nterms.len(), 1);

    let nterm = super::process_nonterminals(o_to_r(nterms.pop(), "")?, &g_data, &s_store, &templates, &mut Default::default())?;

    assert_eq!(nterm.sub_nterms.len(), 1);
    assert_eq!(nterm.symbols.len(), 3);

    RadlrResult::Ok(())
  }

  #[test]
  fn process_custom_parse_state() -> RadlrResult<()> {
    let (g, path, s_store) = create_test_data(
      r##" 
      
      test-sweet-home-alabama =!> match: TERMINAL {
          ( "test" ) { reduce 2 symbols to data :ast { t_Test } then pass }
          ( "failed" ) { fail }
          ( "accepted in kind" ) { accept }
      }
      
       "##,
    )?;

    let g_data = super::create_grammar_data(g, &path, &s_store)?;

    let (nonterminals, _, mut parse_states) = super::extract_nonterminals(&g_data, &s_store)?;

    assert_eq!(nonterminals.len(), 0);
    assert_eq!(parse_states.len(), 1);

    let parse_state = super::process_parse_state(parse_states.pop().unwrap(), &g_data, &s_store)?;

    assert_eq!(parse_state.symbols.len(), 3);

    RadlrResult::Ok(())
  }
}
