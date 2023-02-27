use super::{compile::get_struct_type_from_node, types::*};
use crate::{
  grammar::compile::parser::sherpa::{
    ASTNode,
    ASTNodeType,
    AST_NamedReference,
    AST_Struct,
    GetASTNodeType,
  },
  types::*,
  writer::code_writer::*,
};
use std::{
  collections::{hash_map, BTreeMap, BTreeSet, HashMap},
  fmt::Display,
  io::Write,
  mem::Discriminant,
  vec,
};

#[derive(Clone)]
pub(crate) struct AscriptTypeHandler<'a> {
  pub name:    &'a dyn Fn(&AScriptStore, &AScriptTypeVal, bool) -> String,
  pub default: &'a dyn Fn(&AScriptStore, &AScriptTypeVal, bool) -> String,
}

#[derive(Clone)]
pub(crate) struct ASTExprHandler<'a> {
  /// ```no_compile
  /// pub(crate) fn render_expression(
  ///      utils: &AscriptWriterUtils,
  ///      ast: &ASTNode,
  ///      rule: &Rule,
  ///      ref_index: &mut usize,
  ///      type_slot: usize,
  ///) -> Option<Ref>
  /// ```
  pub expr: &'a dyn Fn(&AscriptWriterUtils, &ASTNode, &Rule, &mut usize, usize) -> Option<Ref>,
}

pub(crate) type PropHandlerFn =
  dyn Fn(&AscriptWriterUtils, Option<Ref>, &AScriptTypeVal, bool) -> (String, Option<Ref>);

pub(crate) struct AscriptPropHandler<'a> {
  /// ```no_compile
  /// pub(crate) fn render_expression(
  ///      utils: &AscriptWriterUtils,
  ///      ast: &ASTNode,
  ///      rule: &Rule,
  ///      ref_index: &mut usize,
  ///      type_slot: usize,
  ///) -> Option<Ref>
  /// ```
  pub expr: &'a PropHandlerFn,
}

type AssignmentWriter<'a> =
  dyn Fn(&AscriptWriterUtils, &AScriptTypeVal, String, String, bool) -> String;

type SlotAssign<'a> = dyn Fn(&AscriptWriterUtils, &AScriptTypeVal, String) -> String;

/// Called when a Node struct needs to be constructed.
type StructConstructorExpr = dyn Fn(
  &AscriptWriterUtils,
  &mut CodeWriter<Vec<u8>>,
  String,
  Vec<(String, String, AScriptTypeVal)>,
  bool,
) -> SherpaResult<()>;

/// Token Concatenation
///
/// The expressions used to concetenate two parse tokens. The
/// resulting token should represent a span of characters from
/// the beginning of the first token to the end of the last token.
///
/// Example Function:
/// ```no_compile
/// fn token_concat(first:String, last:String) -> String {
///     format!("{first} + {last}")
/// }
/// ```
type TokenConcat = dyn Fn(String, String) -> String;

type GetName = dyn Fn(usize) -> String;

/// Slot Extraction:
///
/// Expression to extract and assign either the node value or token from a
/// parse slot, or both.
///
type SlotExtract = dyn Fn(Option<String>, Option<String>, usize) -> String;

/// Slot Extraction:
///
/// Expression to extract and assign either the node value or token from a
/// parse slot, or both.
///
type CreateToken = dyn Fn(String, TokenCreationType) -> String;

pub(crate) enum TokenCreationType {
  String,
  Token,
}

pub(crate) struct StructProp<'a> {
  pub name:        String,
  pub type_string: String,
  pub type_:       &'a TaggedType,
  pub optional:    bool,
}
pub(crate) struct StructData<'a> {
  pub name:      String,
  pub props:     Vec<StructProp<'a>>,
  pub tokenized: bool,
}

#[derive(Default)]
pub(crate) struct Handlers<'a> {
  type_handlers: HashMap<Discriminant<AScriptTypeVal>, AscriptTypeHandler<'a>>,
  prop_handlers: HashMap<Discriminant<AScriptTypeVal>, AscriptPropHandler<'a>>,
  expr_handlers: HashMap<ASTNodeType, ASTExprHandler<'a>>,
}

pub(crate) struct AscriptWriterUtils<'a> {
  /// Internal use. Assign to `Default::default()`
  pub handlers: Handlers<'a>,
  /// General Assignment
  pub assignment_writer: &'a AssignmentWriter<'a>,
  pub slot_assign: &'a SlotAssign<'a>,
  pub token_concat: &'a TokenConcat,
  /// Slot Extraction:
  ///
  /// Expression to extract and assign either the node value or token from a
  /// parse slot, or both.
  ///
  pub slot_extract: &'a SlotExtract,
  pub create_token: &'a CreateToken,
  /// Name used for token variables
  pub get_token_name: &'a GetName,
  pub get_slot_obj_name: &'a GetName,
  pub struct_construction: &'a StructConstructorExpr,

  pub store: &'a AScriptStore,
}

impl<'a> AscriptWriterUtils<'a> {
  #[track_caller]
  pub fn add_type_handler(&mut self, type_: AScriptTypeVal, handler: AscriptTypeHandler<'a>) {
    match self.handlers.type_handlers.entry(type_.get_discriminant()) {
      hash_map::Entry::Occupied(_) => {
        panic!("Type handler already registered for [{}]", type_.debug_string(None))
      }
      hash_map::Entry::Vacant(e) => {
        e.insert(handler);
      }
    }
  }

  #[track_caller]
  pub fn add_ast_handler(&mut self, type_: ASTNodeType, handler: ASTExprHandler<'a>) {
    match self.handlers.expr_handlers.entry(type_) {
      hash_map::Entry::Occupied(_) => {
        panic!("Type handler already registered for [{:?}]", type_)
      }
      hash_map::Entry::Vacant(e) => {
        e.insert(handler);
      }
    }
  }

  #[track_caller]
  pub fn add_prop_handler(&mut self, type_: AScriptTypeVal, handler: AscriptPropHandler<'a>) {
    match self.handlers.prop_handlers.entry(type_.get_discriminant()) {
      hash_map::Entry::Occupied(_) => {
        panic!("Prop handler already registered for [{:?}]", type_)
      }
      hash_map::Entry::Vacant(e) => {
        e.insert(handler);
      }
    }
  }

  /// Increase the value of the monotonic reference index counter.
  pub fn bump_ref_index(&self, ref_index: &mut usize) -> usize {
    *ref_index += 1;
    *ref_index
  }

  pub fn ascript_type_to_string(&self, type_: &AScriptTypeVal, optional: bool) -> String {
    let discriminant = type_.get_discriminant();
    if let Some(type_handler) = self.handlers.type_handlers.get(&discriminant) {
      (*type_handler.name)(self.store, type_, optional)
    } else {
      format!("[UNHANDLED {}]", type_.debug_string(None))
    }
  }

  pub fn ascript_type_to_default_string(&self, type_: &AScriptTypeVal, optional: bool) -> String {
    let discriminant = type_.get_discriminant();
    if let Some(type_handler) = self.handlers.type_handlers.get(&discriminant) {
      (*type_handler.default)(self.store, type_, optional)
    } else {
      format!("[UNHANDLED {}]", type_.debug_string(None))
    }
  }

  pub fn ast_expr_to_ref(
    &self,
    ast: &ASTNode,
    rule: &Rule,
    ref_index: &mut usize,
    type_slot: usize,
  ) -> Option<Ref>
  where
    Self: Sized,
  {
    if let Some(expr_handler) = self.handlers.expr_handlers.get(&ast.get_type()) {
      (*expr_handler.expr)(self, ast, rule, ref_index, type_slot)
    } else {
      panic!("{}", SherpaError::SourceError {
        loc:        ast.to_token(),
        path:       rule.grammar_ref.path.clone(),
        id:         "ascript-writer-utils-unhandled-ast-node",
        msg:        format!("An unhandled ast node has been encountered"),
        inline_msg: format!("Node type [{:?}] lacks an ASTExprHandler", ast.get_type()),
        ps_msg:     "Add an ASTExprHandler for this type using AscriptWriterUtils::add_ast_handler"
          .into(),
        severity:   SherpaErrorSeverity::Warning,
      })
    }
  }

  pub fn build_struct_constructor(
    &self,
    rule: &Rule,
    struct_type: &AScriptStructId,
    ast_struct: &AST_Struct,
    ref_index: &mut usize,
    type_slot: usize,
  ) -> SherpaResult<Ref> {
    let store = self.store;
    let archetype_struct = store.structs.get(struct_type).unwrap();
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

    let mut predecessors = vec![];

    let prop_assignments = archetype_struct
      .prop_ids
      .iter()
      .enumerate()
      .map(|(i, prop_id)| {
        let prop_val = store.props.get(prop_id);
        let struct_prop_val = match prop_val {
          Some(prop) => {
            if let Some(ast_prop) = ast_struct_props.get(&prop_id.name) {
              let property = store.props.get(prop_id).unwrap();
              let Some(value) = &ast_prop.value else {
                panic!(" Prop has no value! {}", ast_prop.tok.blame(1, 1, "", None));
              };

              match self.ast_expr_to_ref(value, rule, ref_index, i + type_slot * 100) {
                Some(ref_) => {
                  let (string, ref_) = self.create_type_initializer_value(
                    Some(ref_),
                    &(&property.type_val).into(),
                    property.optional,
                  );
                  if let Some(ref_) = ref_ {
                    predecessors.push(ref_);
                  }

                  string
                }
                _ => self.ascript_type_to_default_string(&(&prop.type_val).into(), prop.optional),
              }
            } else {
              self.ascript_type_to_default_string(&(&prop.type_val).into(), prop.optional)
            }
          }
          _ => self.ascript_type_to_default_string(&(&AScriptTypeVal::Undefined), false),
        };

        (prop_id.name.clone(), struct_prop_val, AScriptTypeVal::Undefined)
      })
      .collect();

    let mut writer = CodeWriter::new(vec![]);

    (self.struct_construction)(
      self,
      &mut writer,
      archetype_struct.type_name.clone(),
      prop_assignments,
      archetype_struct.tokenized,
    );

    (*ref_index) += 1;

    let mut ref_ = Ref::ast_obj(
      *ref_index,
      type_slot,
      String::from_utf8(writer.into_output()).unwrap(),
      AScriptTypeVal::Struct(*struct_type),
    );

    ref_.add_predecessors(predecessors);

    SherpaResult::Ok(ref_)
  }

  /// Used to convert a value into an appropriate form to assign as
  /// a struct's  value.
  pub fn create_type_initializer_value(
    &self,
    ref_: Option<Ref>,
    type_val: &AScriptTypeVal,
    optional: bool,
  ) -> (String, Option<Ref>) {
    match self.handlers.prop_handlers.get(&type_val.get_discriminant()) {
      Some(AscriptPropHandler { expr }) => (*expr)(&self, ref_, type_val, optional),
      _ => (ref_.clone().map(|ref_| ref_.get_ref_name()).unwrap_or_default(), ref_),
    }
  }
}

pub(crate) struct AscriptWriter<'a, W: Write> {
  pub store: &'a AScriptStore,
  pub utils: &'a AscriptWriterUtils<'a>,
  writer:    CodeWriter<W>,
}

impl<'a, W: Write> AscriptWriter<'a, W> {
  /// Writes a multiline block structure to the output.
  pub fn new(utils: &'a AscriptWriterUtils, writer: CodeWriter<W>) -> Self {
    Self { store: utils.store, writer, utils }
  }

  fn get_struct_data(&self) -> BTreeMap<AScriptStructId, StructData<'a>> {
    self
      .store
      .structs
      .values()
      .map(|s| {
        (s.id, StructData {
          name:      s.type_name.clone(),
          tokenized: s.tokenized,
          props:     s
            .prop_ids
            .iter()
            .filter_map(|p_id| {
              self.store.props.get(p_id).map(|p| StructProp {
                name:        p_id.name.clone(),
                type_string: self.utils.ascript_type_to_string(&(&p.type_val).into(), p.optional),
                optional:    p.optional,
                type_:       &p.type_val,
              })
            })
            .collect(),
        })
      })
      .collect()
  }

  pub fn block(
    &mut self,
    block_header: &str,
    open_delim: &str,
    closing_delim: &str,
    content_writer: &dyn Fn(&mut Self) -> SherpaResult<()>,
  ) -> SherpaResult<()> {
    self.writer.write_line("\n")?;

    if !block_header.is_empty() {
      self.writer.write(&block_header)?;
    }

    self.writer.write(open_delim)?;
    self.writer.increase_indent();

    content_writer(self)?;

    self.writer.decrease_indent();
    self.writer.write_line(closing_delim)?;

    SherpaResult::Ok(())
  }

  fn checkpoint<B: Write + Default>(&self) -> AscriptWriter<'a, B> {
    AscriptWriter {
      store:  self.store,
      writer: self.writer.checkpoint::<B>(),
      utils:  self.utils,
    }
  }

  // Writes a single stmt string on a new line.
  pub fn stmt(&mut self, stmt: String) -> SherpaResult<()> {
    self.writer.write_line(&stmt)?;
    SherpaResult::Ok(())
  }

  pub fn method(
    &mut self,
    preamble: &str,
    args_open_delim: &str,
    args_close_delim: &str,
    args_seperator: &str,
    args: &dyn Fn(&mut Self) -> Vec<String>,
    postamble: &str,
    open_delim: &str,
    closing_delim: &str,
    content_writer: &mut dyn FnMut(&mut Self) -> SherpaResult<()>,
  ) -> SherpaResult<()> {
    self.writer.write_line("\n")?;
    self.writer.write(preamble)?;
    self.writer.write(" ")?;
    self.writer.write(args_open_delim)?;
    let args = args(self).join(args_seperator);
    self.writer.write(&args)?;
    self.writer.write(args_close_delim)?;
    self.writer.write(postamble)?;
    self.writer.write(" ")?;
    self.writer.write(open_delim)?;
    self.writer.increase_indent();
    content_writer(self)?;
    self.writer.decrease_indent();
    self.writer.write_line(closing_delim)?;
    SherpaResult::Ok(())
  }

  pub fn list<S: Display>(&mut self, delim: &str, data: Vec<S>) -> SherpaResult<()> {
    for datum in data {
      self.writer.write_line(&datum.to_string())?;
      self.writer.write(delim)?;
    }

    SherpaResult::Ok(())
  }

  pub fn write_struct_data<'b: 'a>(
    &mut self,
    struct_write_script: &dyn Fn(&mut Self, &StructData) -> SherpaResult<()>,
  ) -> SherpaResult<()> {
    for (_, struct_data) in self.get_struct_data() {
      struct_write_script(self, &struct_data)?;
    }

    SherpaResult::Ok(())
  }

  fn write_slot_extraction(
    &mut self,
    rule: &Rule,
    obj_indices: BTreeSet<usize>,
    token_indices: BTreeSet<usize>,
  ) -> SherpaResult<()> {
    for slot_index in 0..rule.get_real_len() {
      let ref_index = slot_index + 1;
      let (n, t) = (
        match (slot_index, token_indices.contains(&slot_index)) {
          (0, _) => Some((self.utils.get_token_name)(ref_index)),
          (i, _) if i == (rule.get_real_len() - 1) => Some((self.utils.get_token_name)(ref_index)),
          (_, true) => Some((self.utils.get_token_name)(ref_index)),
          _ => None,
        },
        obj_indices.contains(&slot_index).then(|| (&self.utils.get_slot_obj_name)(ref_index)),
      );

      self.writer.wrtln(&(self.utils.slot_extract)(n, t, slot_index))?;
    }
    SherpaResult::Ok(())
  }

  fn write_node_token(&mut self, rule: &Rule) -> SherpaResult<()> {
    let type_ = AScriptTypeVal::Token;
    let string = (self.utils.assignment_writer)(
      self.utils,
      &type_,
      (self.utils.get_token_name)(0),
      if rule.get_real_len() > 1 {
        (self.utils.token_concat)(
          (self.utils.get_token_name)(1),
          (self.utils.get_token_name)(rule.get_real_len()),
        )
      } else {
        (self.utils.get_token_name)(1)
      },
      false,
    );
    self.writer.write_line(&string)?;
    SherpaResult::Ok(())
  }

  pub fn write_reduce_functions(
    &mut self,
    preamble: &str,
    args_open_delim: &str,
    args_close_delim: &str,
    args_seperator: &str,
    args: &dyn Fn(&mut AscriptWriter<Vec<u8>>) -> Vec<String>,
    postamble: &str,
    open_delim: &str,
    closing_delim: &str,
    reduce_fn_writer: &dyn Fn(&mut Self, &Vec<String>) -> SherpaResult<()>,
  ) -> SherpaResult<()> {
    let store = self.store;
    let g = store.g.clone();

    let ordered_rules = g
      .rules
      .iter()
      .filter_map(|(_, rule)| {
        if g.parse_productions.contains(&rule.prod_id) {
          Some((rule.bytecode_id, rule))
        } else {
          None
        }
      })
      .collect::<BTreeMap<_, _>>();

    let mut reduce_functions_map = Vec::new();

    for (bc_id, rule) in &ordered_rules {
      let prod_id = rule.prod_id;
      let prod_data = store.prod_types.get(&prod_id).unwrap();

      #[cfg(debug_assertions)]
      {
        if prod_data.len() != 1 {
          unreachable!(
            "\n\nProduction result not been resolved\n[{}] == {}\n\n\n{}\n\n",
            g.get_production_plain_name(&prod_id),
            rule.tok.blame(1, 1, "", BlameColor::RED),
            prod_data
              .iter()
              .map(|(p, _)| { p.debug_string(Some(&g)) })
              .collect::<Vec<_>>()
              .join("\n")
          )
        };
      }

      let mut w = self.checkpoint::<Vec<u8>>();
      let fn_name = format!("reducer_{:0>3}", bc_id.unwrap());

      match w.method(
        &format!("{}", preamble.replace("%%", &fn_name)),
        args_open_delim,
        args_close_delim,
        args_seperator,
        args,
        postamble,
        open_delim,
        closing_delim,
        &mut move |w| -> SherpaResult<()> {
          if rule.ast_definition.is_none() {
            let last_index = rule.get_real_len() - 1;
            let last_index_name = (&w.utils.get_slot_obj_name)(last_index + 1);
            w.write_slot_extraction(
              rule,
              BTreeSet::from_iter(vec![last_index]),
              BTreeSet::from_iter(vec![0, last_index]),
            )?;
            w.write_node_token(rule)?;
            w.writer.wrtln(&(w.utils.slot_assign)(
              w.utils,
              &AScriptTypeVal::Any,
              last_index_name,
            ))?;
            SherpaResult::Ok(())
          } else {
            let mut ref_index = rule.get_real_len();
            match rule.ast_definition.as_ref().map(|n| &n.ast) {
              Some(ASTNode::AST_Struct(box ast_struct)) => {
                if let AScriptTypeVal::Struct(struct_type) = get_struct_type_from_node(ast_struct) {
                  let _ref = w.utils.build_struct_constructor(
                    rule,
                    &struct_type,
                    ast_struct,
                    &mut ref_index,
                    0,
                  )?;

                  let obj_indices = _ref.get_ast_obj_indices();
                  let token_indices = _ref.get_token_indices();

                  w.write_slot_extraction(rule, obj_indices, token_indices)?;
                  w.write_node_token(rule)?;
                  w.writer.write_line(&_ref.to_init_string(w.utils))?;
                  w.writer.wrtln(&(w.utils.slot_assign)(
                    w.utils,
                    &AScriptTypeVal::Struct(struct_type),
                    _ref.get_ref_name(),
                  ))?;
                }
                SherpaResult::Ok(())
              }
              Some(ASTNode::AST_Statements(box statements)) => {
                let mut reference = String::new();
                let mut return_type = AScriptTypeVal::Undefined;
                let mut refs = BTreeSet::new();
                let mut tokens = BTreeSet::new();
                let mut stmt = w.checkpoint();

                for (i, statement) in statements.statements.iter().enumerate() {
                  match stmt.utils.ast_expr_to_ref(statement, rule, &mut ref_index, i) {
                    Some(_ref) => {
                      refs.append(&mut _ref.get_ast_obj_indices());
                      tokens.append(&mut _ref.get_token_indices());
                      return_type = _ref.ast_type.clone();
                      reference = _ref.get_ref_name();
                      stmt.writer.write_line(&_ref.to_init_string(w.utils))?;
                    }
                    _ => panic!("Could not resolve: {statement:?}"),
                  }
                }

                w.write_slot_extraction(rule, refs, tokens)?;

                w.write_node_token(rule)?;

                w.writer.merge_checkpoint(stmt.writer)?;

                let return_type = match return_type {
                  AScriptTypeVal::Undefined | AScriptTypeVal::GenericVec(None) => {
                    prod_data.iter().next().unwrap().0.into()
                  }
                  r => r,
                };

                w.writer.wrtln(&(w.utils.slot_assign)(w.utils, &return_type, reference))?;

                SherpaResult::Ok(())
              }
              type_ => unreachable!("Type {type_:?} should not be a root ascript node."),
            }
          }
        },
      ) {
        SherpaResult::Ok(()) => {
          self.writer.merge_checkpoint(w.writer)?;
          reduce_functions_map.push(fn_name);
        }
        _ => panic!("Invalid Result"),
      }
    }

    reduce_fn_writer(self, &reduce_functions_map)?;

    SherpaResult::Ok(())
  }

  pub fn into_writer(self) -> CodeWriter<W> {
    self.writer
  }
}

// Writing stages.
// Pramble data -
//  - Base type info

#[derive(Clone, Copy)]
pub(crate) enum RefIndex {
  Tok(usize),
  Obj(usize),
}

#[derive(Clone)]
pub(crate) struct Ref {
  slot_type: RefIndex,
  type_slot: usize,
  init_expression: String,
  pub ast_type: AScriptTypeVal,
  predecessors: Option<Vec<Box<Ref>>>,
  post_init_statements: Option<Vec<String>>,
  is_mutable: bool,
}

impl Ref {
  pub fn ast_obj(
    slot_index: usize,
    type_slot: usize,
    init_expression: String,
    ast_type: AScriptTypeVal,
  ) -> Self {
    Ref {
      slot_type: RefIndex::Obj(slot_index),
      type_slot,
      init_expression,
      ast_type,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub(crate) fn token(utils: &AscriptWriterUtils, slot_index: usize, type_slot: usize) -> Self {
    Ref {
      slot_type: RefIndex::Tok(slot_index),
      type_slot,
      init_expression: (utils.create_token)(
        (utils.get_token_name)(slot_index + 1),
        TokenCreationType::Token,
      ),
      ast_type: AScriptTypeVal::Token,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub(crate) fn node_token(
    utils: &AscriptWriterUtils,
    slot_index: usize,
    type_slot: usize,
  ) -> Self {
    Ref {
      slot_type: RefIndex::Tok(slot_index),
      type_slot,
      init_expression: (utils.create_token)((utils.get_token_name)(0), TokenCreationType::Token),
      ast_type: AScriptTypeVal::Token,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub(crate) fn to_string(self, utils: &AscriptWriterUtils, ast_type: AScriptTypeVal) -> Self {
    let i = match self.get_root_slot_index() {
      RefIndex::Obj(i) | RefIndex::Tok(i) => i,
    };

    Ref {
      slot_type: RefIndex::Tok(i),
      type_slot: self.type_slot,
      init_expression: (utils.create_token)(
        (utils.get_token_name)(i + 1),
        TokenCreationType::String,
      ),
      ast_type,
      predecessors: None,
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub(crate) fn get_type(&self) -> AScriptTypeVal {
    self.ast_type.clone()
  }

  pub(crate) fn from(self, init_expression: String, ast_type: AScriptTypeVal) -> Self {
    Ref {
      slot_type: self.slot_type,
      type_slot: self.type_slot,
      init_expression,
      ast_type,
      predecessors: Some(vec![Box::new(self)]),
      post_init_statements: None,
      is_mutable: false,
    }
  }

  pub(crate) fn make_mutable(&mut self) -> &mut Self {
    self.is_mutable = true;
    self
  }

  pub(crate) fn get_ref_name(&self) -> String {
    match self.slot_type {
      RefIndex::Obj(i) => format!("obj_{i}_{}", self.type_slot),
      RefIndex::Tok(i) => format!("tok_{i}_{}", self.type_slot),
    }
  }

  pub(crate) fn get_root_slot_index(&self) -> RefIndex {
    if let Some(predecessors) = &self.predecessors {
      for predecessor in predecessors {
        return predecessor.get_root_slot_index();
      }
    }
    self.slot_type
  }

  pub(crate) fn get_ast_obj_indices(&self) -> BTreeSet<usize> {
    let mut set = BTreeSet::new();

    if let RefIndex::Obj(index) = self.slot_type {
      set.insert(index);
    }
    if let Some(predecessors) = &self.predecessors {
      for predecessor in predecessors {
        set.append(&mut predecessor.get_ast_obj_indices());
      }
    }

    set
  }

  pub(crate) fn get_token_indices(&self) -> BTreeSet<usize> {
    let mut set = BTreeSet::new();

    if let RefIndex::Tok(index) = self.slot_type {
      set.insert(index);
    } else {
      if let Some(predecessors) = &self.predecessors {
        for predecessor in predecessors {
          set.append(&mut predecessor.get_token_indices());
        }
      }
    }

    set
  }

  pub(crate) fn add_post_init_stmt(&mut self, string: String) -> &mut Self {
    self.post_init_statements.get_or_insert(vec![]).push(string);
    self
  }

  /// Convert the ref into a string of statements that convert original
  /// type into it current form.
  pub(crate) fn to_init_string(&self, utils: &AscriptWriterUtils) -> String {
    let mut strings = Vec::new();

    if let Some(predecessors) = &self.predecessors {
      for predecessor in predecessors {
        strings.push(predecessor.to_init_string(utils));
      }
    }

    let ref_string = self.get_ref_name();

    strings.push((utils.assignment_writer)(
      utils,
      &self.ast_type,
      ref_string.clone(),
      self.init_expression.replace("%%", &ref_string),
      self.is_mutable,
    ));

    if let Some(statements) = &self.post_init_statements {
      strings.append(&mut statements.clone());
    }

    strings.join("\n").replace("%%", &ref_string)
  }

  pub(crate) fn add_predecessor(&mut self, predecessor: Ref) -> &mut Self {
    self.predecessors.get_or_insert(vec![]).push(Box::new(predecessor));

    self
  }

  pub(crate) fn add_predecessors(&mut self, predecessors: Vec<Ref>) -> &mut Self {
    let prev = self.predecessors.get_or_insert(vec![]);

    for predecessor in predecessors {
      prev.push(Box::new(predecessor))
    }

    self
  }
}

pub(crate) fn get_ascript_export_data(
  g: &GrammarStore,
  utils: &AscriptWriterUtils,
) -> Vec<(Option<Ref>, AScriptTypeVal, String, String, String)> {
  let export_node_data = g
    .get_exported_productions()
    .iter()
    .map(|ExportedProduction { export_name, production, guid_name, .. }| {
      let mut ref_index = 0;
      let ref_ = utils.ast_expr_to_ref(
        &ASTNode::AST_NamedReference(Box::new(AST_NamedReference {
          tok:   Token::default(),
          value: "--first--".to_string(),
        })),
        &Rule {
          syms: vec![RuleSymbol {
            scanner_index: 1,
            scanner_length: 1,
            sym_id: SymbolID::Production(production.id, GrammarId(0)),
            grammar_ref: g.id.clone(),
            ..Default::default()
          }],
          len: 1,
          prod_id: production.id,
          id: RuleId(0),
          bytecode_id: None,
          ast_definition: None,
          grammar_ref: g.id.clone(),
          tok: Token::default(),
          ..Default::default()
        },
        &mut ref_index,
        0,
      );
      let ast_type = ref_.as_ref().unwrap().get_type();
      let ast_type_string = utils.ascript_type_to_string(&ast_type, false);
      (ref_, ast_type, ast_type_string, export_name.to_string(), guid_name.to_string())
    })
    .collect::<Vec<_>>();
  export_node_data
}
