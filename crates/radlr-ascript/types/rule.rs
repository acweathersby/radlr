use crate::{Initializer, StructInitializer};
use radlr_core::DBRuleKey;
use radlr_formatter::*;
use std::fmt::Debug;

formatted_typed_vector!(AscriptRules, AscriptRule, "rules");

#[derive(Debug)]
pub enum AscriptRule {
  Struct(usize, StructInitializer),
  Expression(usize, Initializer),
  ListInitial(usize, Initializer),
  ListContinue(usize, Initializer),
  LastSymbol(usize, Initializer),
  Invalid(usize),
}

impl AscriptRule {
  pub fn get_db_key(&self) -> DBRuleKey {
    match self {
      AscriptRule::Struct(id, ..)
      | AscriptRule::Expression(id, ..)
      | AscriptRule::ListInitial(id, ..)
      | AscriptRule::ListContinue(id, ..)
      | AscriptRule::LastSymbol(id, ..)
      | AscriptRule::Invalid(id, ..) => DBRuleKey::from(*id as u32),
    }
  }
}

impl ValueObj for AscriptRule {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    use AscriptRule::*;
    match self {
      Struct(..) => "StructRule",
      Expression(..) => "ExpressionRule",
      ListInitial(..) => "ListInitialRule",
      ListContinue(..) => "ListContinueRule",
      LastSymbol(..) => "LastSymbolRule",
      Invalid(..) => "InvalidRule",
    }
  }

  fn get_val<'scope>(&'scope self, key: &str, _: &radlr_core::IStringStore) -> Value<'scope> {
    use AscriptRule::*;
    match key {
      "init" => match self {
        Struct(_, init) => Value::Obj(init),
        Expression(_, init) => Value::Obj(init),
        ListInitial(_, init) => Value::Obj(init),
        ListContinue(_, init) => Value::Obj(init),
        LastSymbol(_, init) => Value::Obj(init),
        Invalid(..) => Value::None,
      },
      "id" => match self {
        Struct(id, ..)
        | Expression(id, ..)
        | ListInitial(id, ..)
        | ListContinue(id, ..)
        | LastSymbol(id, ..)
        | Invalid(id, ..) => Value::Int(*id as isize),
      },
      _ => Value::None,
    }
  }
}
