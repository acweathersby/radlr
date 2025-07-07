use crate::{build_db, AscriptMultis, AscriptRules, AscriptScalarType, AscriptStructs, AscriptTypes};
use radlr_core::{
  proxy::OrderedSet,
  CachedString,
  DBNonTermKey,
  GrammarDatabase,
  IString,
  RadlrError,
  RadlrGrammarDatabase,
  RadlrResult,
};
use radlr_formatter::*;
use std::{fmt::Debug, io::Write, sync::Arc};

pub type MultiTypeRef = (DBNonTermKey, OrderedSet<AscriptScalarType>, OrderedSet<usize>);

/// Stores resolved type and construction information used to create
/// AST constructors based on `:ast` expressions extracted from a
/// grammar.
pub struct AscriptDatabase {
  pub(crate) structs:       AscriptStructs,
  /// If errors are present then it is likely that the
  /// database is in an incomplete state and SHOULD NOT be
  /// used to construct parser outputs.
  pub(crate) errors:        Vec<RadlrError>,
  /// Initializing information for converting a rules into AST nodes.
  pub(crate) rules:         AscriptRules,
  /// A set of all non-struct types produced by the the grammar's rules
  pub(crate) types:         AscriptTypes,
  /// A set of `multi` type groups that should be converted into special enum
  /// types.
  pub(crate) multi_type_lu: Vec<usize>,
  /// A set of `multi` type groups that should be converted into special enum
  /// types.
  pub(crate) multi_types:   Vec<MultiTypeRef>,
  ///  The underlying grammar database from which Ascript types are
  /// derived.
  pub(crate) db:            Arc<GrammarDatabase>,
}

impl Debug for AscriptDatabase {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("AscriptDatabase");
    s.field("rules", &self.rules);
    s.field("structs", &self.structs);
    s.field("types", &self.types);
    s.field("erros", &self.errors);
    s.field("multi_type_lu", &self.multi_type_lu);
    s.field("multi_types", &self.multi_types);
    s.finish()
  }
}

impl AscriptDatabase {
  /// Format and print this database to a string using the a formatting
  /// script.
  pub fn format_string(
    &self,
    script: &str,
    max_width: usize,
    ast_struct_name: &str,
    extra_config_properites: &[(&str, &str)],
  ) -> RadlrResult<String> {
    unsafe {
      Ok(String::from_utf8_unchecked(self.format(
        script,
        Vec::with_capacity(1024),
        max_width,
        ast_struct_name,
        extra_config_properites,
      )?))
    }
  }

  /// Format and print this database to the given buffer using a formatting
  /// script.
  pub fn format<W: Write>(
    &self,
    script: &str,
    writer: W,
    max_width: usize,
    ast_struct_name: &str,
    extra_config_properites: &[(&str, &str)],
  ) -> RadlrResult<W> {
    let mut ctx: FormatterContext = FormatterContext::new("AscriptForm", self.db.string_store().clone());

    let multi = AscriptMultis::new(self);
    ctx.set_val("STRUCTS", Value::Obj(&self.structs));
    ctx.set_val("TYPES", Value::Obj(&self.types));
    ctx.set_val("RULES", Value::Obj(&self.rules));
    ctx.set_val("MULTI_ENUMS", Value::Obj(&multi));
    ctx.set_val("AST_NAME", Value::Str(ast_struct_name.intern(self.db.string_store())));
    ctx.set_val("ALLOW_UPPER_ATTRIBUTES", Value::Int(0));

    for (name, value) in extra_config_properites {
      ctx.set_val(name, Value::Str(value.intern(self.db.string_store())));
    }

    // The type of heap allocation to use for ast nodes. example: `std::sync::Arc`
    ctx.max_width = max_width;

    let f: Formatter = FormatterResult::from(script).into_result()?;
    f.write_to_output(&mut ctx, writer)
  }

  pub fn get_errors(&self) -> Option<&[RadlrError]> {
    if self.errors.len() > 0 {
      Some(&self.errors)
    } else {
      None
    }
  }
}

impl ValueObj for AscriptDatabase {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AscriptDatabase"
  }

  fn get_val<'scope>(&'scope self, key: &str, _: &radlr_core::IStringStore) -> Value<'scope> {
    match key {
      _ => Value::None,
    }
  }
}

impl From<&RadlrGrammarDatabase> for AscriptDatabase {
  fn from(value: &RadlrGrammarDatabase) -> Self {
    build_db::build_database((*value).clone())
  }
}

impl From<RadlrGrammarDatabase> for AscriptDatabase {
  fn from(value: RadlrGrammarDatabase) -> Self {
    build_db::build_database(value)
  }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct StringId(pub(crate) IString);

impl ToValue for StringId {
  fn into_val<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    self.0.into_val(s_store)
  }
}

impl AsRef<IString> for StringId {
  fn as_ref(&self) -> &IString {
    &self.0
  }
}

impl Into<IString> for StringId {
  fn into(self) -> IString {
    self.0
  }
}

impl From<&str> for StringId {
  fn from(value: &str) -> Self {
    Self(value.to_token())
  }
}

impl From<String> for StringId {
  fn from(value: String) -> Self {
    Self(value.to_token())
  }
}
