use super::{item::Item, *};
use crate::{
  grammar::{
    compile::{
      compile_grammars,
      parse::{load_from_path, load_from_string},
      parser::sherpa::{self, ASTNode, Ascript, Reduce},
    },
    create_closure,
    get_closure_cached,
    get_guid_grammar_name,
    get_production_start_items,
    hash_id_value_u64,
  },
  journal::Journal,
};
use std::{
  collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
  fmt::Display,
  path::PathBuf,
  sync::Arc,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]

/// A Globally Unique Id to quickly distinguish instances of [GrammarStore]. This
/// value is derived from the filepath of the grammar's source code.
pub struct GrammarId(pub u64);

impl Display for GrammarId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0.to_string())
  }
}

impl From<&PathBuf> for GrammarId {
  fn from(value: &PathBuf) -> Self {
    GrammarId(hash_id_value_u64(&get_guid_grammar_name(value)))
  }
}

/// Stores the absolute paths and source code of a `*.hcg` file.
#[derive(Debug, Clone)]
pub struct HCGSource {
  /// The absolute path of a hcg file.
  pub absolute_path: PathBuf,
  /// The source code of a hcg file.
  pub source:        String,
}
#[derive(Debug, Clone)]
pub enum ReduceFunctionType {
  Generic(Reduce),
  AscriptOld(Ascript),
  Ascript(crate::grammar::compile::parser::sherpa::Ascript),
  Undefined,
}

impl ReduceFunctionType {
  pub fn new(node: &ASTNode) -> Self {
    match node {
      ASTNode::Reduce(box reduce) => ReduceFunctionType::Generic(reduce.clone()),
      ASTNode::Ascript(box ascript) => ReduceFunctionType::AscriptOld(ascript.clone()),
      _ => ReduceFunctionType::Undefined,
    }
  }
}

/// Identifiers for a Grammar
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct GrammarRef {
  /// A globally unique name to refer to this grammar by. Derived from the
  /// grammar's filepath.
  pub guid_name: String,

  /// The user defined name. This is either the value of the `@NAME` preamble,
  /// or the original file name stem if this preamble is not present.
  pub name: String,

  /// A globally unique identifier for this GrammarStore instance. Derived
  /// from the source path
  pub guid: GrammarId,

  /// The absolute path of the grammar's source file. This may be empty if the source code was passed
  /// in as a string, as with the case of grammars compiled with
  /// [compile_grammar_from_string](sherpa_core::grammar::compile_grammar_from_string)).
  pub path: PathBuf,
}

impl GrammarRef {
  /// TODO
  pub fn new(local_name: String, absolute_path: PathBuf) -> Arc<Self> {
    let guid_name = get_guid_grammar_name(&absolute_path);
    Arc::new(GrammarRef {
      guid: (&absolute_path).into(),
      guid_name,
      name: local_name,
      path: absolute_path,
    })
  }
}

pub type ImportedGrammarReferences = HashMap<String, Arc<GrammarRef>>;

pub type ReduceFunctionTable = BTreeMap<ReduceFunctionId, ReduceFunctionType>;

/// Houses data essential to the compilation and analysis of Hydrocarbon
/// grammar source code.
///
/// # Instantiation
///
/// Use one of the following functions to construct a GrammarStore:
/// - ## [compile_from_path](crate::grammar::compile_from_path)
///     
///     # Examples
///     ```ignore
///     use sherpa::compile_grammar_from_path;
///
///     let number_of_threads = 1;
///     let (grammar_store_option, errors) = compile_from_path(
///         PathBuf::from_str("./my_grammar.hcg"),
///         number_of_threads
///     );
///     ```
/// - ## [compile_from_string](crate::grammar::compile_from_string)

#[derive(Debug, Clone, Default)]
pub struct GrammarStore {
  /// TODO: Docs
  pub id: Arc<GrammarRef>,

  /// Maps [ProductionId] to a list of [RuleIds](RuleId)
  pub production_rules: ProductionBodiesTable,

  /// Maps a [ProductionId] to a [Production].
  pub productions: ProductionTable,

  /// Maps a production's id to it's original name and guid name
  pub production_names: BTreeMap<ProductionId, (String, String)>,

  /// Maps RuleId to rule data.
  pub rules: RuleTable,

  /// Maps [SymbolId] to [Symbol] data. Only stores [Symbols](Symbol) that
  /// represent one of the following:
  /// - [SymbolID::DefinedNumeric]
  /// - [SymbolID::DefinedIdentifier]
  /// - [SymbolID::DefinedSymbol]
  /// - [SymbolID::TokenProduction]
  pub symbols: SymbolsTable,

  /// Maps SymbolId to its original source token string.
  pub symbol_strings: SymbolStringTable,

  /// Store of all production ids encountered in grammar.
  pub production_symbols: BTreeMap<SymbolID, Token>,

  /// Maps a local import name to an absolute file path and its
  /// UUID.
  pub imports: ImportedGrammarReferences,

  /// Closure of all items that can be produced by this grammar.
  pub(crate) closures: HashMap<Item, Vec<Item>>,

  pub(crate) item_ignore_symbols: HashMap<Item, Vec<SymbolID>>,
  /// TODO: Docs
  pub production_ignore_symbols:  HashMap<ProductionId, Vec<SymbolID>>,

  /// A mapping of [ProductionId]s to export names
  ///
  /// These export names are generated from the grammar production:
  ///
  /// ```hgc
  /// <> export_preamble > \@EXPORT sym::production_symbol ( t:AS | t:as ) tk:export_id
  /// ```
  /// where `tk:export_id` is assigned to the second tuple position.
  ///
  /// If no export names are declared in the root grammar, then this will contain
  /// the id of the first production declared in the root grammar, assigned to the
  /// name `default`.
  pub exports: Vec<(ProductionId, String)>,

  /// All items in the grammar that are `B => . A b` for some production `A`.
  pub(crate) lr_items:   BTreeMap<ProductionId, Vec<Item>>,
  /// TODO: Docs
  pub merge_productions: BTreeMap<ProductionId, (String, Vec<Rule>)>,

  /// All productions that are either entry productions or are reachable from the entry productions
  pub parse_productions: BTreeSet<ProductionId>,

  /// Maps bytecode id to rule id for all rules.
  pub bytecode_rule_lookup: BTreeMap<u32, RuleId>,

  /// Maps bytecode id to production id for all productions that are reachable from the entry productions.
  pub bytecode_production_lookup: BTreeMap<u32, ProductionId>,

  /// Maps bytecode id to all tokens that can be generated by scanner states.
  pub bytecode_token_lookup: BTreeMap<u32, SymbolID>,
}

impl GrammarStore {
  fn compile_grammars(
    j: &mut Journal,
    grammars: Vec<(PathBuf, HashMap<String, Arc<GrammarRef>>, Box<sherpa::Grammar>)>,
  ) -> SherpaResult<Arc<GrammarStore>> {
    j.flush_reports();

    if j.have_errors_of_type(SherpaErrorSeverity::Critical) {
      return SherpaResult::None;
    }

    compile_grammars(j, &grammars);

    j.flush_reports();

    if j.have_errors_of_type(SherpaErrorSeverity::Critical) {
      return SherpaResult::None;
    }

    SherpaResult::Ok(j.grammar()?)
  }

  /// Create a GrammarStore from a Grammar file loaded from the filesystem.
  /// This will load any references within the grammar and compile all grammar
  /// definitions into a single GrammarStore.
  ///
  /// Any errors generated during the parsing or compilation of the grammars
  /// will be recorded in the Journal.
  pub fn from_path(j: &mut Journal, path: PathBuf) -> SherpaResult<Arc<GrammarStore>> {
    j.set_active_report(
      "Entry Grammar Parse",
      crate::ReportType::GrammarCompile(Default::default()),
    );
    let grammars = load_from_path(j, path);
    Self::compile_grammars(j, grammars)
  }

  /// Create a GrammarStore from a Grammar defined in a string as if it were
  /// a file located in the folder defined by `base_dir`.
  /// This will load any references within the grammar and compile all grammar
  /// definitions into a single GrammarStore.
  ///
  /// Any errors generated during the parsing or compilation of the grammars
  /// will be recorded in the Journal.
  pub fn from_str_with_base_dir(
    j: &mut Journal,
    string: &str,
    base_dir: &PathBuf,
  ) -> SherpaResult<Arc<GrammarStore>> {
    j.set_active_report(
      "Entry Grammar Parse",
      crate::ReportType::GrammarCompile(Default::default()),
    );
    let grammars = load_from_string(j, string, base_dir.to_owned());
    Self::compile_grammars(j, grammars)
  }

  /// Compile a GrammarStore from a grammar source `str`.
  ///
  /// # Example
  /// ```rust
  /// use sherpa_core::{Journal, ReportType};
  /// use sherpa_core::compile::GrammarStore;
  ///  
  /// let mut j = Journal::new(None); // Use journal with default config;
  ///
  /// let g = GrammarStore::from_str(&mut j,
  /// r###"
  /// <> A > "hello" "world"
  /// "###
  /// );
  ///
  /// // Print the compilation report.
  /// j.flush_reports();
  /// j.debug_print_reports(ReportType::GrammarCompile(Default::default()));
  /// ```
  pub fn from_str(j: &mut Journal, string: &str) -> SherpaResult<Arc<GrammarStore>> {
    j.set_active_report(
      "Entry Grammar Parse",
      crate::ReportType::GrammarCompile(Default::default()),
    );
    let grammars = load_from_string(j, string, Default::default());
    Self::compile_grammars(j, grammars)
  }

  /// Same as `Self::from_str` except with a `String` type.
  pub fn from_string(j: &mut Journal, string: String) -> SherpaResult<Arc<GrammarStore>> {
    return Self::from_str(j, string.as_str());
  }

  /// Returns a reference to a Symbol given a SymbolId
  pub fn get_symbol(&self, sym_id: &SymbolID) -> Option<&Symbol> {
    match sym_id {
      sym if sym.is_defined() => self.symbols.get(sym),
      sym if sym.is_generic() => Some(*Symbol::generics_lu().get(sym).unwrap()),
      _ => None,
    }
  }

  /// Returns the [Rule] that's mapped to [`rule_id`](RuleId)
  /// within the grammar
  pub fn get_rule(&self, rule_id: &RuleId) -> SherpaResult<&Rule> {
    SherpaResult::Ok(self.rules.get(rule_id)?)
  }

  /// Returns the [Production] that's mapped to [`production_id`](ProductionId)
  /// within the grammar
  pub fn get_production(&self, production_id: &ProductionId) -> SherpaResult<&Production> {
    SherpaResult::Ok(self.productions.get(production_id)?)
  }

  /// Returns a list of [ExportedProductions](ExportedProduction) extracted from
  /// the grammar.
  #[inline]
  pub fn get_exported_productions(&self) -> Vec<ExportedProduction> {
    self
      .exports
      .iter()
      .map(|(id, name)| {
        let production = self.productions.get(id).unwrap();
        ExportedProduction {
          export_name: name,
          guid_name: &production.guid_name,
          production,
          export_id: production.export_id.unwrap(),
        }
      })
      .collect::<Vec<_>>()
  }

  /// Retrieve the non-import and unmangled name of a [Production](Production).
  pub fn get_production_plain_name(&self, prod_id: &ProductionId) -> &str {
    if let Some(prod) = self.productions.get(prod_id) {
      &prod.name
    } else if let Some((name, _)) = self.production_names.get(prod_id) {
      name
    } else {
      ""
    }
  }

  /// Returns GUID entry name for a an entry production, or panics if the
  /// production is not an entry production
  pub fn get_entry_name_from_prod_id(&self, prod_id: &ProductionId) -> SherpaResult<String> {
    if let Some(prod) = self.productions.get(prod_id) {
      if prod.export_id.is_some() {
        SherpaResult::Ok(prod.guid_name.clone() + "_enter")
      } else {
        SherpaResult::Err(SherpaError::SourceError {
          loc:        prod.loc.clone(),
          path:       self.id.path.clone(),
          id:         "invalid-entry-production",
          msg:        format!("Production {} is not an exported production.", prod.name),
          inline_msg: "".into(),
          ps_msg:     "".into(),
          severity:   SherpaErrorSeverity::Critical,
        })
      }
    } else {
      SherpaResult::Err(format!("Could not find entry name for production {:?}", prod_id).into())
    }
  }

  /// Retrieve the globally unique name of a [Production](Production).
  pub fn get_production_guid_name(&self, prod_id: &ProductionId) -> &str {
    if let Some(prod) = self.productions.get(prod_id) {
      &prod.guid_name
    } else if let Some((_, name)) = self.production_names.get(prod_id) {
      name
    } else {
      ""
    }
  }

  /// Attempts to retrieve a rule from the grammar with the matching bytecode_id.
  pub fn get_rule_by_bytecode_id(&self, bytecode_id: u32) -> SherpaResult<&Rule> {
    SherpaResult::Ok(
      self.bytecode_rule_lookup.get(&bytecode_id).and_then(|prod_id| self.rules.get(prod_id))?,
    )
  }

  /// Attempts to retrieve a production from the grammar with the matching bytecode_id.
  pub fn get_production_by_bytecode_id(&self, bytecode_id: u32) -> SherpaResult<&Production> {
    SherpaResult::Ok(
      self
        .bytecode_production_lookup
        .get(&bytecode_id)
        .and_then(|prod_id| self.productions.get(prod_id))?,
    )
  }

  /// Todo: Docs
  pub fn get_symbol_by_bytecode_id(&self, bytecode_id: u32) -> SherpaResult<&Symbol> {
    SherpaResult::Ok(
      self.bytecode_token_lookup.get(&bytecode_id).and_then(|sym_id| self.symbols.get(sym_id))?,
    )
  }

  /// Todo: Docs
  pub fn get_symbol_id_by_bytecode_id(&self, bytecode_id: u32) -> SherpaResult<&SymbolID> {
    SherpaResult::Ok(self.bytecode_token_lookup.get(&bytecode_id)?)
  }

  /// Attempts to retrieve a production from the grammar with the matching name.
  /// If the grammar is an aggregate of multiple grammars which define productions
  /// with the same name, the production that is selected is undetermined.
  pub fn get_production_by_name(&self, name: &str) -> SherpaResult<&Production> {
    for production_id in self.productions.keys() {
      if name == self.get_production_plain_name(production_id) {
        return SherpaResult::Ok(self.productions.get(production_id).unwrap());
      }
    }

    SherpaResult::None
  }

  /// Retrieves first the production_id of the first production
  /// whose plain or guid name matches the query string.
  /// Returns None if no production matches the query.
  pub fn get_production_id_by_name(&self, name: &str) -> Option<ProductionId> {
    for (prod_id, prod) in self.productions.iter() {
      if name == self.get_production_plain_name(prod_id) {
        return Some(prod_id.to_owned());
      }
      if name == prod.guid_name {
        return Some(prod_id.to_owned());
      }
    }

    None
  }

  /// Evaluates whether a production is recursive. Returns
  /// a double of booleans.
  ///
  /// The first boolean value indicates that production is recursive.
  ///
  /// The second boolean value indicates a production has left
  /// recursive, either directly or indirectly.
  pub fn get_production_recursion_type(&self, prod_id: ProductionId) -> RecursionType {
    let mut seen = HashSet::<Item>::new();

    let mut pipeline = VecDeque::from_iter(
      create_closure(&get_production_start_items(&prod_id, self), self).iter().map(|i| (0, *i)),
    );

    let mut recurse_type = RecursionType::NONE;

    while let Some((offset, item)) = pipeline.pop_front() {
      if !item.is_completed() {
        let other_prod_id = item.get_production_id_at_sym(self);

        if prod_id == other_prod_id {
          if offset == 0 {
            if item.get_prod_id(self) == prod_id {
              recurse_type = recurse_type + RecursionType::LEFT_DIRECT;
            } else {
              recurse_type = recurse_type + RecursionType::LEFT_INDIRECT;
            }
          } else {
            recurse_type = recurse_type + RecursionType::RIGHT;
          }
        }

        if seen.insert(item) {
          let new_item = item.increment().unwrap();

          pipeline.push_back((offset + 1, new_item));

          if let SymbolID::Production(..) = new_item.get_symbol(self) {
            for item in get_closure_cached(&new_item, self) {
              pipeline.push_back((offset + 1, *item));
            }
          }
        }
      }
    }
    recurse_type
  }

  //
  pub(crate) fn get_production_start_items_from_name(&self, name: &str) -> Vec<Item> {
    match self.get_production_id_by_name(name) {
      Some(prod_id) => get_production_start_items(&prod_id, self),
      None => vec![],
    }
  }
}
