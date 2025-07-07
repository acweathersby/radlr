use super::AscriptType;
use crate::StringId;
use radlr_core::{proxy::OrderedMap, CachedString, GrammarIdentities};
use radlr_formatter::*;
use radlr_rust_runtime::types::Token;
use std::fmt::Debug;

formatted_typed_ordered_map!(AscriptStructs, StringId, AscriptStruct, "structs");

#[derive(Debug)]
pub struct AscriptStruct {
  #[allow(unused)]
  pub(crate) id:                StringId,
  pub(crate) name:              String,
  pub(crate) properties:        AscriptStructProps,
  pub(crate) has_token:         bool,
  /// The node itself or one of its properties requires a template specifier
  /// for TK
  pub(crate) requires_template: bool,
  pub(crate) initialized:       bool,
}

formatted_typed_ordered_map!(AscriptStructProps, StringId, AscriptProp, "AscriptStructProps");

impl ValueObj for AscriptStruct {
  fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    match key {
      "is_empty" => Value::Int((self.properties.len() == 0) as isize),
      "name" => Value::Str(self.name.intern(s_store)),
      "props" => Value::Obj(&self.properties),
      "has_token" => Value::Int(self.has_token as isize),
      "requires_template" => Value::Int(self.requires_template as isize),
      _ => Value::None,
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AscriptStruct"
  }
}

pub struct AscriptProp {
  pub(crate) is_optional: bool,
  pub(crate) name:        String,
  pub(crate) ty:          AscriptType,
  pub(crate) tok:         Token,
  pub(crate) g_id:        GrammarIdentities,
}

impl Debug for AscriptProp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("AscriptProp");

    s.field("name", &self.name);
    s.field("ty", &self.ty);
    s.field("optional", &self.is_optional);

    s.finish()
  }
}

impl ValueObj for AscriptProp {
  fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    match key {
      "name" => Value::Str(self.name.intern(s_store)),
      "type" => Value::Obj(&self.ty),

      "optional" => {
        if self.is_optional {
          Value::Int(1)
        } else {
          Value::None
        }
      }
      _ => Value::None,
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AScriptProp"
  }
}
