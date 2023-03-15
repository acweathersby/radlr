use super::{
  errors::add_invalid_import_source_error,
  parser::{self as ast, ast::escaped_from, ASTNode, Grammar, Rule},
  string::{CachedString, IString, StringStore},
  types::{
    self,
    ASTToken,
    Array,
    CustomState,
    GrammarHeader,
    GrammarId,
    GrammarIdentity,
    Map,
    Production,
    ProductionId,
    ProductionType,
    SubProduction,
    SymbolId,
    SymbolType,
    TokenSymbol,
  },
  utils::{get_symbol_details, resolve_grammar_path, SymbolData},
};
use crate::{
  ascript::types::{ascript_first_node_id, ascript_last_node_id},
  grammar::{hash_id_value_u64, new::parser::GetASTNodeType},
  Journal,
  SherpaResult,
};
use std::path::PathBuf;

/// Temporary structure to host rule data during
/// construction.
pub(super) struct RuleData<'a> {
  symbols: &'a [(usize, &'a ASTNode)],
  ast_ref: Option<ASTToken>,
}

/// Temporary structure to host production data during
/// construction.
pub(super) struct ProductionData<'a> {
  root_prod_id: ProductionId,
  syms:         &'a mut Map<SymbolId, TokenSymbol>,
  sub_prods:    &'a mut Array<Box<SubProduction>>,
  rules:        &'a mut Array<types::Rule>,
  asts:         &'a mut Array<Box<ast::Ascript>>,
}
impl<'b> ProductionData<'b> {
  /// Create a new [ProductionData] that references a different
  /// `rules` array, but keeps all other references the same
  /// as `self`.
  pub(super) fn set_rules<'d>(
    &'d mut self,
    rules: &'d mut Array<types::Rule>,
  ) -> ProductionData<'d> {
    ProductionData {
      rules,
      root_prod_id: self.root_prod_id,
      syms: &mut self.syms,
      sub_prods: &mut self.sub_prods,
      asts: &mut self.asts,
    }
  }
}

/// Intermediate structure to host grammar data during
/// construction.
pub(super) struct GrammarData {
  pub name:      IString,
  pub source_id: GrammarId,
  pub imports:   Map<IString, GrammarIdentity>,
  pub exports:   Array<ProductionId>,
  pub grammar:   Box<Grammar>,
}

pub(super) fn convert_grammar_data_to_header(
  import_id: GrammarIdentity,
  g_data: GrammarData,
) -> Box<GrammarHeader> {
  let mut identity = import_id;
  identity.name = g_data.name;
  Box::new(GrammarHeader {
    identity,
    pub_prods: g_data.exports,
    imports: g_data.imports.values().map(|v| v.guid).collect(),
  })
}

/// Parse grammar string and create a Grammar AST.
pub(crate) fn parse_grammar(string_data: &str) -> SherpaResult<Box<Grammar>> {
  SherpaResult::Ok(Grammar::from_str(string_data)?)
}

/// Do an initial preparation of the grammar data.
pub(super) fn create_grammar_data(
  j: &mut Journal,
  grammar: Box<Grammar>,
  grammar_path: &PathBuf,
  string_store: &StringStore,
) -> SherpaResult<GrammarData> {
  const exts: [&str; 3] = ["sg", "sherpa", "hcg"];

  let source_dir = grammar_path.parent()?.to_owned();
  let mut imports = Map::default();
  let mut exports = Array::default();
  let mut name =
    grammar_path.file_name().and_then(|d| d.to_str()).unwrap_or("default");

  for preamble in &grammar.preamble {
    match preamble {
      ASTNode::Import(import) => {
        let box ast::Import { reference, uri, tok } = import;
        let path = PathBuf::from(uri);

        match resolve_grammar_path(&path, &source_dir, &exts) {
          SherpaResult::Ok(path) => {
            imports.insert(
              reference.intern(string_store),
              GrammarIdentity::from_path(&path, string_store),
            );
          }
          _ => {
            add_invalid_import_source_error(j, import, &path, &source_dir);
          }
        }
      }
      ASTNode::Export(export) => {
        // Saving exports to a temporary store until we are able extract import
        // information from the preambles, since some exported symbols may
        // be Production_Import_Symbol types.
        exports.push(export.clone());
      }
      ASTNode::Ignore(_) => {}
      ASTNode::Name(name_ast) => {
        name = name_ast.name.as_str();
      }
      _ => {
        #[allow(unreachable_code)]
        {
          #[cfg(debug_assertions)]
          unreachable!("Unrecognized node: {:?}", preamble.get_type());
          unreachable!()
        }
      }
    }
  }

  let mut g_data = GrammarData {
    name: name.intern(string_store),
    source_id: grammar_path.into(),
    grammar,
    imports,
    exports: Default::default(),
  };

  // Once we have g_data, and more importantly g_data.imports, we can convert
  // our exports into ProductionIds.
  if exports.is_empty() != true {
    for export in exports {
      let prod_id =
        get_production_id_from_ast_node(&g_data, &export.production)?;
      g_data.exports.push(prod_id);
    }
  } else {
    // Use the fist declared production as the default entry
    let prod = &g_data.grammar.productions[0];
    let prod_id = get_production_id_from_ast_node(&g_data, &prod)?;
    g_data.exports.push(prod_id);
  }

  SherpaResult::Ok(g_data)
}

pub(super) fn extract_productions<'a>(
  j: &mut Journal,
  g_data: &'a GrammarData,
  s_store: &StringStore,
) -> SherpaResult<(
  Array<(Box<Production>, &'a ASTNode)>,
  Array<Box<CustomState>>,
)> {
  let mut productions = Array::new();
  let mut parse_states = Array::new();
  for production in &g_data.grammar.productions {
    match production {
      ASTNode::State(parse_state) => parse_states.push(Box::new(CustomState {
        id:      ProductionId::from((
          g_data.source_id,
          parse_state.id.name.as_str(),
        )),
        g_id:    g_data.source_id,
        tok:     parse_state.tok.clone(),
        state:   parse_state.clone(),
        symbols: Default::default(),
      })),
      ASTNode::PrattProduction(box ast::PrattProduction {
        name_sym,
        rules,
        tok,
      })
      | ASTNode::CFProduction(box ast::CFProduction { name_sym, rules, tok })
      | ASTNode::PegProduction(box ast::PegProduction {
        name_sym,
        rules,
        tok,
      }) => {
        productions.push((
          Box::new(Production {
            id:        ProductionId::from((
              g_data.source_id,
              name_sym.name.as_str(),
            )),
            name:      name_sym.name.intern(s_store),
            g_id:      g_data.source_id,
            type_:     super::types::ProductionType::ContextFree,
            rules:     Default::default(),
            sub_prods: Default::default(),
            symbols:   Default::default(),
            tok_prods: Default::default(),
            tok:       tok.clone(),
            asts:      Default::default(),
          }),
          production,
        ));
      }
      _ => unreachable!(),
    }
  }

  SherpaResult::Ok((productions, parse_states))
}

pub(super) fn process_parse_state<'a>(
  mut parse_state: Box<CustomState>,
  g_data: &GrammarData,
  s_store: &StringStore,
) -> SherpaResult<Box<CustomState>> {
  // Extract symbol information from the state.

  let CustomState { id, g_id, state, symbols, .. } = parse_state.as_mut();
  {
    let ast::Statement { branch, non_branch, transitive } =
      state.statement.as_ref();

    if let Some(branch) = &branch {
      match branch {
        ASTNode::TerminalMatches(term_matches) => {
          for match_ in &term_matches.matches {
            match match_ {
              ASTNode::TermMatch(term_match) => {
                record_symbol(
                  &term_match.sym,
                  0,
                  &mut ProductionData {
                    root_prod_id: *id,
                    syms:         symbols,
                    sub_prods:    &mut Default::default(),
                    rules:        &mut Default::default(),
                    asts:         &mut Default::default(),
                  },
                  g_data,
                  s_store,
                )?;
              }
              _ => {
                // Default
                // Hint
              }
            }
          }
        }
        ASTNode::ProductionMatches(prod_matches) => {}
        ASTNode::Matches(ast_match) => match ast_match.mode.as_str() {
          "TERMINAL" => {}
          _ => {
            /*
             * PRODUCTION
             * TERMINAL
             */
          }
        },
        _ => {
          // ASTNode::Gotos
          // ASTNode::Pass
          // ASTNode::Fail
          // ASTNode::Accept
        }
      }
    }
  }

  SherpaResult::Ok(parse_state)
}

pub(super) fn process_prod<'a>(
  (mut production, prod_ast): (Box<Production>, &'a ASTNode),
  g_data: &GrammarData,
  s_store: &StringStore,
) -> SherpaResult<Box<Production>> {
  match prod_ast {
    ASTNode::CFProduction(prod) => {
      production.type_ = ProductionType::ContextFree;
      for rule in &prod.rules {
        process_rule(&mut production, rule, g_data, s_store)?;
      }
    }
    _ => unreachable!(),
  }

  SherpaResult::Ok(production)
}

fn process_rule(
  prod: &mut Production,
  rule: &Rule,
  g_data: &GrammarData,
  s_store: &StringStore,
) -> SherpaResult<()> {
  let ast_syms = rule.symbols.iter().enumerate().collect::<Array<_>>();

  let Production { id, rules, sub_prods, symbols, type_, asts, .. } = prod;
  let mut prod_data =
    ProductionData { root_prod_id: *id, syms: symbols, sub_prods, rules, asts };

  let ast_ref = intern_ast(rule, &mut prod_data);

  process_rule_symbols(
    &RuleData { symbols: &ast_syms, ast_ref },
    &mut prod_data,
    g_data,
    s_store,
  )?;

  SherpaResult::Ok(())
}

fn intern_ast(rule: &Rule, p_data: &mut ProductionData) -> Option<ASTToken> {
  let ast_ref = rule.ast.as_ref().map(|s| {
    p_data.asts.push(s.clone());
    ASTToken::Defined(p_data.root_prod_id, p_data.asts.len() - 1)
  });
  ast_ref
}

fn process_rule_symbols(
  rule_data: &RuleData,
  p_data: &mut ProductionData,
  g_data: &GrammarData,
  s_store: &StringStore,
) -> SherpaResult<()> {
  let mut rules: Array<types::Rule> = Default::default();
  rules.push(types::Rule {
    symbols: Default::default(),
    ast:     rule_data.ast_ref,
  });

  for (index, sym) in rule_data.symbols.iter() {
    let original_bodies = 0..rules.len();
    let SymbolData {
      annotation,
      is_list,
      is_group,
      is_optional,
      is_shift_nothing,
      precedence,
      sym_atom,
      is_eof,
    } = get_symbol_details(sym);

    // If the symbol is optional, then we create new set
    // of rule symbols that will not contain the optional
    // symbol.
    is_optional.then(|| rules.append(&mut rules.clone()));

    let sym = match sym {
      ASTNode::NotEmptySet(set) => {
        let mut pending_rules = vec![];

        fn get_index_permutations(
          indexes: Array<usize>,
        ) -> Array<Array<usize>> {
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

        let indices = set.symbols.iter().enumerate().map(|(i, _)| i).collect();

        let candidate_symbols = set
          .symbols
          .iter()
          .enumerate()
          .map(|(i, s)| (i + index, s))
          .collect::<Array<_>>();

        for permutation in if set.unordered {
          get_index_permutations(indices)
        } else {
          vec![indices]
        } {
          let symbols = permutation
            .iter()
            .map(|i| candidate_symbols[*i])
            .collect::<Array<_>>();

          process_rule_symbols(
            &RuleData { symbols: &symbols, ast_ref: None },
            &mut p_data.set_rules(&mut pending_rules),
            g_data,
            s_store,
          )?;
        }

        let mut new_bodies = Array::new();

        for pending_rule in pending_rules {
          for rule in &mut rules[original_bodies.clone()] {
            let mut new_body = rule.clone();
            new_body.symbols.extend(pending_rule.symbols.iter().cloned());
            new_bodies.push(new_body)
          }
        }

        rules.splice(original_bodies, new_bodies);

        continue;
      }
      ASTNode::GroupProduction(group) => {
        // All bodies are plain without annotations or functions
        if annotation.is_empty()
          && !some_rules_have_ast_definitions(&group.rules)
        {
          // For each rule in the group clone the existing rule lists and
          // process each list independently, inserting the new symbols
          // into the existing bodies. We must make sure the indices are
          // preserved since only the last symbol in each rule can be bound
          // to the index of the group production symbol.

          let mut p = vec![];

          for rule in &group.rules {
            process_rule_symbols(
              &RuleData {
                symbols: &map_symbols_to_unindexed(&rule.symbols),
                ast_ref: None,
              },
              &mut p_data.set_rules(&mut p),
              g_data,
              s_store,
            )?;
          }

          let mut new_rules = vec![];

          for pending_rule in &mut p {
            // The last symbol in each of these new bodies is set
            // with the original symbol id
            pending_rule.symbols.last_mut()?.1 = *index;
            for rule in &mut rules[original_bodies.clone()] {
              let mut new_rule = rule.clone();
              new_rule.symbols.extend(pending_rule.symbols.iter().cloned());
              new_rules.push(new_rule)
            }
          }

          rules.splice(original_bodies, new_rules);

          // We do not to process the existing symbol as it is
          // now replaced with its component rule symbols,
          // so we'll skip the rest of the loop
          continue;
        } else {
          let mut sub_prod_rules = Default::default();

          for rule in &group.rules {
            let n_rule = &RuleData {
              symbols: &rule.symbols.iter().enumerate().collect::<Array<_>>(),
              ast_ref: intern_ast(rule, p_data),
            };
            process_rule_symbols(
              n_rule,
              &mut p_data.set_rules(&mut sub_prod_rules),
              g_data,
              s_store,
            )?
          }

          let prod_id = p_data.root_prod_id;
          let index = p_data.sub_prods.len();
          let id = ProductionId::from((prod_id, index));

          p_data.sub_prods.push(Box::new(SubProduction {
            id,
            type_: types::SubProductionType::Group,
            g_id: g_data.source_id,
            rules: sub_prod_rules,
          }));

          SymbolId::NonTerminal { id, precedence: 0 }
        }
      }
      ASTNode::List_Production(box ast::List_Production {
        symbol,
        terminal_symbol,
        tok,
        ..
      }) => {
        use super::parser::*;

        let mut rule_syms = vec![symbol.clone()];
        let mut rules = Default::default();
        let prod_id = p_data.root_prod_id;
        let index = p_data.sub_prods.len();
        let id = ProductionId::from((prod_id, index));
        let sym = SymbolId::NonTerminal { id, precedence: 0 };

        if let Some(terminal_symbol) = terminal_symbol {
          rule_syms.insert(0, terminal_symbol.clone());
        }

        let r_data = &RuleData {
          symbols: &map_symbols_to_unindexed(&rule_syms),
          ast_ref: Some(ASTToken::ListIterate(tok.get_tok_range())),
        };

        process_rule_symbols(
          r_data,
          &mut p_data.set_rules(&mut rules),
          g_data,
          s_store,
        )?;

        /// Add the ProductionID as a nonterm symbol
        /// to the beginning of each rule, making the
        /// resulting list production left recursive.
        for rule in &mut rules {
          rule.symbols.insert(0, (sym, 9999))
        }

        let r_data = &RuleData {
          symbols: &vec![(9999, symbol)],
          ast_ref: Some(ASTToken::ListEntry(tok.get_tok_range())),
        };

        process_rule_symbols(
          r_data,
          &mut p_data.set_rules(&mut rules),
          g_data,
          s_store,
        )?;

        p_data.sub_prods.push(Box::new(SubProduction {
          type_: types::SubProductionType::List,
          id,
          g_id: g_data.source_id,
          rules,
        }));

        sym
      }
      _ => record_symbol(sym, precedence as u16, p_data, g_data, s_store)?,
    };

    for rule in &mut rules[original_bodies] {
      rule.symbols.push((sym, *index));
    }
  }

  p_data.rules.append(&mut rules);

  SherpaResult::Ok(())
}
fn map_symbols_to_unindexed(
  ast_syms: &Array<ASTNode>,
) -> Array<(usize, &ASTNode)> {
  ast_syms.iter().map(|s| (9999, s)).collect::<Array<_>>()
}
fn some_rules_have_ast_definitions(rules: &[Box<Rule>]) -> bool {
  rules.iter().any(|rule| rule.ast.is_some())
}

/// Converts a Symbol AST node into a symbol object and
/// inventories the symbol into the grammar store.
fn record_symbol(
  sym_node: &ASTNode,
  precedence: u16,
  p_data: &mut ProductionData,
  g_data: &GrammarData,
  s_store: &StringStore,
) -> SherpaResult<SymbolId> {
  let id = match sym_node {
    ASTNode::AnnotatedSymbol(box annotated) => {
      let p =
        annotated.precedence.as_ref().map(|p| p.val as u16).unwrap_or_default();
      record_symbol(&annotated.symbol, p, p_data, g_data, s_store)?
    }
    ASTNode::TerminalToken(box terminal) => {
      let string = escaped_from((&terminal.val).into())?.join("");
      let symbol =
        TokenSymbol { type_: SymbolType::Token, val: string.intern(s_store) };

      let id = SymbolId::Token { id: hash_id_value_u64(symbol), precedence };

      p_data.syms.entry(id).or_insert(symbol);

      id
    }
    ASTNode::Production_Symbol(_) | ASTNode::Production_Import_Symbol(_) => {
      let id = get_production_id_from_ast_node(g_data, sym_node)?;
      SymbolId::NonTerminal { id, precedence }
    }
    ASTNode::EOFSymbol(_) => SymbolId::EndOfFile { precedence },
    ASTNode::ClassSymbol(gen) => match gen.val.as_str() {
      "sp" => SymbolId::GenericSpace { precedence },
      "tab" => SymbolId::GenericHorizontalTab { precedence },
      "nl" => SymbolId::GenericNewLine { precedence },
      "id" => SymbolId::GenericIdentifier { precedence },
      "num" => SymbolId::GenericNumber { precedence },
      "sym" => SymbolId::GenericSymbol { precedence },

      _ => {
        #[allow(unreachable_code)]
        {
          #[cfg(debug_assertions)]
          unreachable!("unsupported generic {}", gen.val);
          unreachable!()
        }
      }
    },
    ASTNode::Production_Terminal_Symbol(token_prod) => {
      let id = get_production_id_from_ast_node(g_data, &token_prod.production)?;
      SymbolId::NonTerminalToken { id, precedence }
    }
    _ => {
      #[allow(unreachable_code)]
      {
        #[cfg(debug_assertions)]
        unreachable!("Unrecognized node: {:?}", sym_node.get_type());
        unreachable!()
      }
    }
  };

  SherpaResult::Ok(id)
}

/// Returns `(None, Some(&Production_Import_Symbol))` or
/// `(Some(&Production_Symbol), None)`from a valid production like node.
///
/// Valid Node:
/// - [ASTNode::Production_Import_Symbol]
/// - [ASTNode::Production_Symbol]
/// - [ASTNode::Production_Terminal_Symbol]
/// - [ASTNode::State]
/// - [ASTNode::PrattProduction]
/// - [ASTNode::PegProduction]
/// - [ASTNode::CFProduction]
fn get_production_symbol<'a>(
  g_data: &GrammarData,
  node: &'a ASTNode,
) -> (
  Option<&'a ast::Production_Symbol>,
  Option<&'a ast::Production_Import_Symbol>,
) {
  match node {
    ASTNode::Production_Import_Symbol(prod_import) => {
      (None, Some(prod_import.as_ref()))
    }
    ASTNode::Production_Symbol(prod_sym) => (Some(prod_sym.as_ref()), None),
    ASTNode::Production_Terminal_Symbol(prod_tok) => {
      get_production_symbol(g_data, &prod_tok.production)
    }
    ASTNode::State(box ast::State { id, .. })
    | ASTNode::PrattProduction(box ast::PrattProduction {
      name_sym: id, ..
    })
    | ASTNode::PegProduction(box ast::PegProduction { name_sym: id, .. })
    | ASTNode::CFProduction(box ast::CFProduction { name_sym: id, .. }) => {
      (Some(id.as_ref()), None)
    }
    _ => unreachable!(),
  }
}
/// Return the `ProductionId` from a valid production like node.
///
/// Valid Node:
/// - [ASTNode::Production_Import_Symbol]
/// - [ASTNode::Production_Symbol]
/// - [ASTNode::Production_Terminal_Symbol]
/// - [ASTNode::State]
/// - [ASTNode::PrattProduction]
/// - [ASTNode::PegProduction]
/// - [ASTNode::CFProduction]
fn get_production_id_from_ast_node(
  g_data: &GrammarData,
  node: &ASTNode,
) -> Option<ProductionId> {
  match get_production_symbol(g_data, node) {
    (Some(prod), None) => {
      Some(ProductionId::from((g_data.source_id, prod.name.as_str())))
    }
    (None, Some(prod)) => {
      let ref_name = prod.module.proxy();

      match g_data.imports.get(&ref_name) {
        Some(GrammarIdentity { guid, .. }) => {
          Some(ProductionId::from((*guid, prod.name.as_str())))
        }
        _ => None,
      }
    }
    _ => {
      #[allow(unreachable_code)]
      {
        #[cfg(debug_assertions)]
        unreachable!("Unrecognized node: {:?}", node.get_type());
        unreachable!()
      }
    }
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use crate::{
    grammar::new::{parser::Grammar, string::StringStore, types::GrammarId},
    Journal,
    SherpaResult,
  };

  fn create_test_data(
    input: &str,
  ) -> SherpaResult<(Journal, Box<Grammar>, PathBuf, StringStore)> {
    let mut j = Journal::new(None);
    j.set_active_report(
      "test",
      crate::ReportType::GrammarCompile(Default::default()),
    );

    let dir = PathBuf::from("/test.sg");

    SherpaResult::Ok((
      j,
      super::parse_grammar(input)?,
      PathBuf::from("/test.sg"),
      StringStore::default(),
    ))
  }
  #[test]
  fn extract_productions() -> SherpaResult<()> {
    let (mut j, g, path, s_store) = create_test_data(
      r##"  <> test-sweet-home-alabama > c:id{3} | ("test"{2} "2" :ast $1 ) :ast $1 "##,
    )?;

    let g_data = super::create_grammar_data(&mut j, g, &path, &s_store)?;

    let (mut prods, ..) =
      super::extract_productions(&mut j, &g_data, &s_store)?;

    assert_eq!(prods.len(), 1);

    let prod = super::process_prod(prods.pop()?, &g_data, &s_store)?;

    assert_eq!(prod.sub_prods.len(), 1);
    assert_eq!(prod.symbols.len(), 2);

    SherpaResult::Ok(())
  }

  #[test]
  fn process_custom_parse_state() -> SherpaResult<()> {
    let (mut j, g, path, s_store) = create_test_data(
      r##" 
      
      test-sweet-home-alabama =!> match: TERMINAL {
          ( "test" ) { reduce 2 symbols to data :ast { t_Test } then pass }
          ( "failed" ) { fail }
          ( "accepted in kind" ) { accept }
      }
      
       "##,
    )?;

    let g_data = super::create_grammar_data(&mut j, g, &path, &s_store)?;

    let (mut productions, mut parse_states) =
      super::extract_productions(&mut j, &g_data, &s_store)?;

    assert_eq!(productions.len(), 0);
    assert_eq!(parse_states.len(), 1);

    let parse_state =
      super::process_parse_state(parse_states.pop()?, &g_data, &s_store)?;

    dbg!(&parse_state, &s_store);

    assert_eq!(parse_state.symbols.len(), 3);

    SherpaResult::Ok(())
  }
}
