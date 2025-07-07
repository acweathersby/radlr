use crate::{AscriptDatabase, MultiTypeRef};

use super::StringId;
use radlr_core::{proxy::OrderedSet, CachedString, DBNonTermKey, IString};
use radlr_formatter::*;
use std::{hash::Hash, rc::Rc};

formatted_typed_ordered_set!(AscriptTypes, AscriptType, "types");
formatted_typed_ordered_set!(AscriptScalarTypes, AscriptScalarType, "types");

#[derive(Debug)]
pub enum AscriptPropType {}

impl ValueObj for AscriptPropType {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AscriptPropType"
  }

  fn get_val<'scope>(&'scope self, key: &str, _: &radlr_core::IStringStore) -> Value<'scope> {
    match key {
      _ => Value::None,
    }
  }

  fn vals<'scope>(&'scope self, funct: &mut dyn FnMut(String, Value<'scope>)) {
    funct("db".to_string(), Value::Obj(self))
  }
}

#[derive(Debug, Clone, Copy, Default, Eq, Hash)]
pub enum AscriptType {
  #[default]
  Undefined,
  Scalar(AscriptScalarType),
  Aggregate(AscriptAggregateType),
  NonTerminalVal(DBNonTermKey),
}

impl AscriptType {
  pub fn requires_template(&self, db: &AscriptDatabase) -> bool {
    use AscriptType::*;
    match self {
      Scalar(scalar) => scalar.requires_template(db),
      Aggregate(agg) => agg.requires_template(db),
      _ => false,
    }
  }

  pub fn get_structs<'b>(&self, struct_types: &'b mut Vec<StringId>, multi_types: &Vec<MultiTypeRef>) -> &'b mut Vec<StringId> {
    match self {
      Self::Scalar(scalar) => {
        scalar.get_structs(struct_types, multi_types);
      }
      Self::Aggregate(agg) => {
        agg.get_structs(struct_types, multi_types);
      }
      _ => {}
    };

    struct_types
  }

  pub fn friendly_name(&self) -> String {
    use AscriptType::*;
    match self {
      Scalar(scalar) => scalar.friendly_name().to_string(),
      Aggregate(aggregate) => aggregate.friendly_name(),
      NonTerminalVal(_) => "Undefined".into(),
      Undefined => "Undefined".into(),
    }
  }

  pub fn to_cardinal(&self) -> AscriptType {
    match self {
      AscriptType::Aggregate(agg) => AscriptType::Aggregate(agg.to_cardinal()),
      AscriptType::Scalar(scalar) => AscriptType::Scalar(scalar.to_cardinal()),
      ty @ _ => ty.clone(),
    }
  }
}

impl PartialOrd for AscriptType {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AscriptType {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    use AscriptType::*;
    let result = match (self, other) {
      (Undefined, Undefined) => Equal,

      (Undefined, _) => Less,

      (_, Undefined) => Greater,

      (Scalar(c), Scalar(d)) => c.cmp(d),
      (Aggregate(c), Aggregate(d)) => c.cmp(d),
      (NonTerminalVal(c), NonTerminalVal(d)) => c.cmp(d),

      (NonTerminalVal(_), Scalar(_)) => Less,
      (Scalar(_), NonTerminalVal(_)) => Greater,

      (Scalar(_), Aggregate(_)) => Less,
      (Aggregate(_), Scalar(_)) => Greater,

      (NonTerminalVal(_), Aggregate(_)) => Less,
      (Aggregate(_), NonTerminalVal(_)) => Greater,
    };

    result
  }
}

impl PartialEq for AscriptType {
  fn eq(&self, other: &Self) -> bool {
    if std::mem::discriminant(self) != std::mem::discriminant(other) {
      false
    } else if let (Some(a), Some(b)) = (self.as_aggregate(), other.as_aggregate()) {
      match (a, b) {
        (
          AscriptAggregateType::Map { key_type: a_key, val_type: a_val },
          AscriptAggregateType::Map { key_type: b_key, val_type: b_val },
        ) => a_key == b_key && a_val == b_val,

        (AscriptAggregateType::Vec { val_type: a_val }, AscriptAggregateType::Vec { val_type: b_val }) => a_val == b_val,
        _ => false,
      }
    } else if let (Some(a), Some(b)) = (self.as_scalar(), other.as_scalar()) {
      a == b
    } else {
      false
    }
  }
}

#[derive(Debug)]
pub enum PendingType {
  Resolved(AscriptType),
  Unresolved { non_terms: OrderedSet<DBNonTermKey>, ty: AscriptType },
  Invalid,
}

impl AscriptType {
  pub fn is_multi(&self) -> bool {
    match self {
      AscriptType::Aggregate(agg_type) => match agg_type {
        AscriptAggregateType::Map { val_type, .. } => val_type.is_multi(),
        AscriptAggregateType::Vec { val_type } => val_type.is_multi(),
      },

      AscriptType::Scalar(val_type) => val_type.is_multi(),

      _ => false,
    }
  }

  pub fn get_multi_type_index(&self) -> Option<usize> {
    match self {
      AscriptType::Aggregate(agg_type) => match agg_type {
        AscriptAggregateType::Map { val_type, .. } => val_type.get_multi_type_index(),
        AscriptAggregateType::Vec { val_type } => val_type.get_multi_type_index(),
      },

      AscriptType::Scalar(val_type) => val_type.get_multi_type_index(),
      _ => None,
    }
  }

  pub fn is_numeric(&self) -> bool {
    if let Some(s) = self.as_scalar() {
      s.is_numeric()
    } else {
      false
    }
  }

  pub fn is_bool(&self) -> bool {
    if let Some(s) = self.as_scalar() {
      s.is_bool()
    } else {
      false
    }
  }

  pub fn is_string(&self) -> bool {
    if let Some(s) = self.as_scalar() {
      s.is_string()
    } else {
      false
    }
  }

  pub fn is_token(&self) -> bool {
    if let Some(s) = self.as_scalar() {
      s.is_token()
    } else {
      false
    }
  }

  pub fn is_unknown(&self) -> bool {
    matches!(self, AscriptType::Undefined)
  }

  pub fn is_map(&self) -> bool {
    matches!(self, AscriptType::Aggregate(AscriptAggregateType::Map { .. }))
  }

  pub fn is_vector(&self) -> bool {
    matches!(self, AscriptType::Aggregate(AscriptAggregateType::Vec { .. }))
  }

  pub fn as_scalar(&self) -> Option<AscriptScalarType> {
    match self {
      Self::Scalar(scalar) => Some(*scalar),
      _ => None,
    }
  }

  pub fn as_aggregate(&self) -> Option<AscriptAggregateType> {
    match self {
      Self::Aggregate(scalar) => Some(scalar.clone()),
      _ => None,
    }
  }
}

impl ValueObj for AscriptType {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AscriptType"
  }

  fn get_keys<'scope>(&'scope self) -> &'static [&'static str] {
    &["agg_type", "key", "val", "literal"]
  }

  fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    fn _scalar_type<'scope>(scaler: &AscriptScalarType, s_store: &radlr_core::IStringStore) -> Value<'scope> {
      match scaler {
        AscriptScalarType::Flag(..) => Value::Str("flag".intern(s_store)),
        AscriptScalarType::Bool(_) => Value::Str("bool".intern(s_store)),
        AscriptScalarType::U8(_) => Value::Str("u8".intern(s_store)),
        AscriptScalarType::U16(_) => Value::Str("u16".intern(s_store)),
        AscriptScalarType::U32(_) => Value::Str("u32".intern(s_store)),
        AscriptScalarType::U64(_) => Value::Str("u64".intern(s_store)),
        AscriptScalarType::I8(_) => Value::Str("i8".intern(s_store)),
        AscriptScalarType::I16(_) => Value::Str("i16".intern(s_store)),
        AscriptScalarType::I32(_) => Value::Str("i32".intern(s_store)),
        AscriptScalarType::I64(_) => Value::Str("i64".intern(s_store)),
        AscriptScalarType::F32(_) => Value::Str("f32".intern(s_store)),
        AscriptScalarType::F64(_) => Value::Str("f64".intern(s_store)),
        AscriptScalarType::Struct(..) => Value::Str("struct".intern(s_store)),
        AscriptScalarType::Multi(_) => Value::Str("any".intern(s_store)),
        AscriptScalarType::Token => Value::Str("tok".intern(s_store)),
        AscriptScalarType::TokenRange => Value::Str("tok_range".intern(s_store)),
        AscriptScalarType::String(_) => Value::Str("str".intern(s_store)),
        AscriptScalarType::Undefined => Value::None,
      }
    }

    match key {
      "agg_type" => match self {
        AscriptType::Aggregate(agg) => match agg {
          AscriptAggregateType::Map { .. } => Value::Str("Map".intern(s_store)),
          AscriptAggregateType::Vec { .. } => Value::Str("Vec".intern(s_store)),
        },
        _ => Value::None,
      },
      "key" => match self {
        AscriptType::Aggregate(agg) => match agg {
          AscriptAggregateType::Map { key_type, .. } => Value::Obj(key_type),
          _ => Value::None,
        },
        AscriptType::Scalar(scl) => match scl {
          AscriptScalarType::Multi(index) => Value::Int(*index as isize),
          _ => Value::None,
        },
        _ => Value::None,
      },
      "val" => match self {
        AscriptType::Aggregate(agg) => match agg {
          AscriptAggregateType::Map { val_type, .. } => Value::Obj(val_type),
          AscriptAggregateType::Vec { val_type, .. } => Value::Obj(val_type),
        },
        AscriptType::Scalar(scaler) => Value::Obj(scaler),
        _ => Value::None,
      },
      "literal" => match self {
        AscriptType::Scalar(scaler) => scaler.get_val(key, s_store),
        _ => Value::None,
      },
      _ => Value::None,
    }
  }

  fn get_len<'scope>(&'scope self) -> usize {
    match self {
      AscriptType::Aggregate(..) => 2,
      _ => 1,
    }
  }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AscriptMultiType(pub OrderedSet<AscriptScalarType>);

impl Hash for AscriptMultiType {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    for ty in &self.0 {
      ty.hash(state)
    }
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AscriptScalarType {
  U8(Option<usize>),
  U16(Option<usize>),
  U32(Option<usize>),
  U64(Option<usize>),
  I8(Option<isize>),
  I16(Option<isize>),
  I32(Option<isize>),
  I64(Option<isize>),
  F32(Option<f64>),
  F64(Option<f64>),
  Flag(StringId, u64),
  String(Option<IString>),
  Bool(bool),
  Struct(StringId, bool),
  Multi(usize),
  Token,
  TokenRange,
  #[default]
  Undefined,
}

impl ValueObj for AscriptScalarType {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    self.friendly_name()
  }

  fn get_keys<'scope>(&'scope self) -> &'static [&'static str] {
    &["name: friendly name of the type", "literal: the default literal value"]
  }

  fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    use AscriptScalarType::*;
    match key {
      "name" => match self {
        Struct(strct, _) => Value::Str(*strct.as_ref()),
        _ => Value::Str(self.get_type().intern(s_store)),
      },
      "is_empty" => match self {
        Struct(_, is_empty) => Value::Int(*is_empty as isize),
        _ => Value::None,
      },
      "index" => match self {
        Multi(index) => Value::Int(*index as isize),
        _ => Value::Str(self.get_type().intern(s_store)),
      },
      "literal" => match self {
        Bool(val) => Value::Str(val.to_string().intern(s_store)),
        U8(val) => Value::Int(val.unwrap_or_default() as isize),
        U16(val) => Value::Int(val.unwrap_or_default() as isize),
        U32(val) => Value::Int(val.unwrap_or_default() as isize),
        U64(val) => Value::Int(val.unwrap_or_default() as isize),
        I8(val) => Value::Int(val.unwrap_or_default() as isize),
        I16(val) => Value::Int(val.unwrap_or_default() as isize),
        I32(val) => Value::Int(val.unwrap_or_default() as isize),
        I64(val) => Value::Int(val.unwrap_or_default() as isize),
        F32(val) => Value::Num(val.unwrap_or_default() as f64),
        F64(val) => Value::Num(val.unwrap_or_default() as f64),
        String(val) => Value::Str(val.unwrap_or_default()),
        _ => Value::None,
      },
      _ => Value::None,
    }
  }
}

impl AscriptScalarType {
  pub fn requires_template(&self, db: &AscriptDatabase) -> bool {
    use AscriptScalarType::*;
    match self {
      Multi(index) => db.multi_types[*index].1.iter().any(|i| i.requires_template(db)),
      Token | TokenRange => true,
      Struct(id, _) => db.structs.get(id).is_some_and(|s| s.requires_template),
      _ => false,
    }
  }

  pub fn get_structs<'b>(&self, struct_types: &'b mut Vec<StringId>, multi_types: &Vec<MultiTypeRef>) -> &'b mut Vec<StringId> {
    match self {
      Self::Multi(id) => {
        for ty in multi_types[*id].1.iter() {
          ty.get_structs(struct_types, multi_types);
        }
      }
      Self::Struct(struct_id, _) => struct_types.push(*struct_id),
      _ => {}
    };

    struct_types
  }

  pub fn precedence(&self) -> u64 {
    use AscriptScalarType::*;
    match self {
      U8(..) => 0,
      U16(..) => 1,
      U32(..) => 2,
      U64(..) => 3,
      I8(..) => 4,
      I16(..) => 5,
      I32(..) => 6,
      I64(..) => 7,
      F32(..) => 8,
      F64(..) => 9,
      String(..) => 10,
      Bool(..) => 11,
      Struct(..) => 12,
      Multi(..) => 13,
      Flag(..) => 14,
      Token => 15,
      TokenRange => 16,
      Undefined => 17,
    }
  }

  pub fn byte_size(&self) -> u64 {
    use AscriptScalarType::*;
    match self {
      Bool(..) | I8(..) | U8(..) => 1,
      I16(..) | U16(..) => 2,
      U32(..) | I32(..) | F32(..) => 4,
      U64(..) | I64(..) | F64(..) => 8,
      String(..) => 16,
      Struct(..) => 128,
      Multi(..) => 0,
      Flag(..) => 8,
      Token => 16,
      TokenRange => 8,
      Undefined => 0,
    }
  }

  pub fn friendly_name(&self) -> &'static str {
    use AscriptScalarType::*;
    match self {
      U8(..) => "U8",
      U16(..) => "U16",
      U32(..) => "U32",
      U64(..) => "U64",
      I8(..) => "I8",
      I16(..) => "I16",
      I32(..) => "I32",
      I64(..) => "I64",
      F32(..) => "F32",
      F64(..) => "F64",
      String(..) => "String",
      Bool(..) => "Bool",
      Struct(..) => "Struct",
      Multi(..) => "Multi",
      Flag(..) => "Flag",
      Token => "Token",
      TokenRange => "TokenRange",
      Undefined => "Undefined",
    }
  }

  pub fn to_cardinal(&self) -> AscriptScalarType {
    use AscriptScalarType::*;
    match self {
      U8(..) => U8(None),
      U16(..) => U16(None),
      U32(..) => U32(None),
      U64(..) => U64(None),
      I8(..) => I8(None),
      I16(..) => I16(None),
      I32(..) => I32(None),
      I64(..) => I64(None),
      F32(..) => F32(None),
      F64(..) => F64(None),
      String(..) => String(None),
      Bool(..) => Bool(false),
      ty @ _ => *ty,
    }
  }

  pub fn is_multi(&self) -> bool {
    match self {
      AscriptScalarType::Multi(_) => true,
      _ => false,
    }
  }

  pub fn get_multi_type_index(&self) -> Option<usize> {
    match self {
      AscriptScalarType::Multi(index) => Some(*index),
      _ => None,
    }
  }

  pub fn is_numeric(&self) -> bool {
    use AscriptScalarType::*;
    match self {
      Bool(..) | U8(..) | U16(..) | U32(..) | U64(..) | I8(..) | I16(..) | I32(..) | I64(..) | F32(..) | F64(..) => true,
      _ => false,
    }
  }

  pub fn is_bool(&self) -> bool {
    use AscriptScalarType::*;
    match self {
      Bool(..) => true,
      _ => false,
    }
  }

  pub fn is_string(&self) -> bool {
    use AscriptScalarType::*;
    match self {
      String(..) => true,
      _ => false,
    }
  }

  pub fn is_token(&self) -> bool {
    use AscriptScalarType::*;
    match self {
      TokenRange | Token => true,
      _ => false,
    }
  }
}

impl Hash for AscriptScalarType {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    let ty = self.to_cardinal();

    unsafe {
      let v = (&ty) as *const _ as *const u8;
      let slice = std::slice::from_raw_parts(v, std::mem::size_of::<AscriptScalarType>());
      state.write(slice);
    }

    let _ = state.finish();
  }
}

impl PartialOrd for AscriptScalarType {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (AscriptScalarType::Multi(a), AscriptScalarType::Multi(b)) => a.partial_cmp(b),
      (AscriptScalarType::Struct(a, _), AscriptScalarType::Struct(b, _)) => a.partial_cmp(b),
      _ => self.precedence().partial_cmp(&other.precedence()),
    }
  }
}

impl Ord for AscriptScalarType {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match (self, other) {
      (AscriptScalarType::Multi(a), AscriptScalarType::Multi(b)) => a.cmp(b),
      (AscriptScalarType::Struct(a, _), AscriptScalarType::Struct(b, _)) => a.cmp(b),
      _ => self.precedence().cmp(&other.precedence()),
    }
  }
}

impl Eq for AscriptScalarType {}

impl PartialEq for AscriptScalarType {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (AscriptScalarType::Multi(a), AscriptScalarType::Multi(b)) => a.cmp(b).is_eq(),
      (AscriptScalarType::Struct(a, _), AscriptScalarType::Struct(b, _)) => a.cmp(b).is_eq(),
      _ => std::mem::discriminant(self) == std::mem::discriminant(other),
    }
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaseType {
  Bool,
  Uint,
  Int,
  Float,
  Token,
  String,
  Other,
}

impl From<AscriptScalarType> for BaseType {
  fn from(value: AscriptScalarType) -> Self {
    use AscriptScalarType::*;
    match value {
      AscriptScalarType::Bool(..) => BaseType::Bool,
      U8(..) | U16(..) | U32(..) | U64(..) => BaseType::Uint,
      I8(..) | I16(..) | I32(..) | I64(..) => BaseType::Int,
      F32(..) | F64(..) => BaseType::Float,
      TokenRange | Token => BaseType::Token,
      String(..) => BaseType::String,
      _ => BaseType::Other,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AscriptAggregateType {
  Vec { val_type: AscriptScalarType },
  Map { key_type: AscriptScalarType, val_type: AscriptScalarType },
}

impl PartialOrd for AscriptAggregateType {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AscriptAggregateType {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    use AscriptAggregateType::*;
    match (self, other) {
      (Vec { val_type: a }, Vec { val_type: b }) => a.cmp(b),
      (Map { val_type: a, key_type: b }, Map { val_type: c, key_type: d }) => match a.cmp(c) {
        std::cmp::Ordering::Equal => b.cmp(d),
        cmp => cmp,
      },
      (Vec { .. }, Map { .. }) => std::cmp::Ordering::Less,
      (Map { .. }, Vec { .. }) => std::cmp::Ordering::Greater,
    }
  }
}

impl AscriptAggregateType {
  pub fn requires_template(&self, db: &AscriptDatabase) -> bool {
    use AscriptAggregateType::*;
    match self {
      Vec { val_type } => val_type.requires_template(db),
      Map { key_type, val_type } => key_type.requires_template(db) || val_type.requires_template(db),
    }
  }

  pub fn get_structs<'b>(&self, struct_types: &'b mut Vec<StringId>, multi_types: &Vec<MultiTypeRef>) -> &'b mut Vec<StringId> {
    match self {
      Self::Vec { val_type } => {
        val_type.get_structs(struct_types, multi_types);
      }
      Self::Map { val_type, .. } => {
        val_type.get_structs(struct_types, multi_types);
      }
    };

    struct_types
  }

  pub fn precedence(&self) -> usize {
    match self {
      AscriptAggregateType::Map { .. } => 0,
      AscriptAggregateType::Vec { .. } => 1,
    }
  }

  pub fn scalar_precedence(&self) -> u64 {
    match self {
      AscriptAggregateType::Map { key_type, val_type } => key_type.precedence() << 32 | val_type.precedence(),
      AscriptAggregateType::Vec { val_type } => val_type.precedence(),
    }
  }

  pub fn friendly_name(&self) -> String {
    use AscriptAggregateType::*;
    match self {
      Vec { val_type: base_type } => base_type.friendly_name().to_string() + &"[]",
      Map { key_type, val_type } => "{ ".to_string() + &key_type.friendly_name() + ":" + &val_type.friendly_name() + "}",
    }
  }

  pub fn to_cardinal(&self) -> AscriptAggregateType {
    match self {
      AscriptAggregateType::Map { key_type, val_type } => {
        AscriptAggregateType::Map { key_type: key_type.to_cardinal(), val_type: val_type.to_cardinal() }
      }
      AscriptAggregateType::Vec { val_type: base_type } => AscriptAggregateType::Vec { val_type: base_type.to_cardinal() },
    }
  }
}

#[derive(Debug, Clone)]
pub enum GraphNode {
  Add(Rc<GraphNode>, Rc<GraphNode>, AscriptType),
  Sub(Rc<GraphNode>, Rc<GraphNode>, AscriptType),
  Mul(Rc<GraphNode>, Rc<GraphNode>, AscriptType),
  Div(Rc<GraphNode>, Rc<GraphNode>, AscriptType),
  Map(Rc<GraphNode>, Rc<GraphNode>, AscriptType),
  Vec(GraphNodeVecInits, AscriptType),
  Str(Option<Rc<GraphNode>>, AscriptType),
  Bool(Option<Rc<GraphNode>>, AscriptType),
  Num(Option<Rc<GraphNode>>, AscriptType),
  Tok(Rc<GraphNode>, AscriptType),
  Trim(Rc<GraphNode>, isize, isize, AscriptType),
  Sym(usize, bool, AscriptType),
  TokSym(usize, bool, AscriptType),
  TokRule(AscriptType),
  MultiConvert(Rc<GraphNode>, AscriptType),
  Undefined(AscriptType),
}

formatted_typed_vector!(GraphNodeVecInits, GraphNode, "nodes", derive(Clone, Debug));

impl GraphNode {
  pub fn get_type(&self) -> &AscriptType {
    use GraphNode::*;
    match self {
      Add(.., ty)
      | Sub(.., ty)
      | Mul(.., ty)
      | Div(.., ty)
      | Map(.., ty)
      | Vec(.., ty)
      | Trim(.., ty)
      | Sym(.., ty)
      | Num(.., ty)
      | Str(.., ty)
      | Bool(.., ty)
      | TokRule(ty)
      | TokSym(.., ty)
      | Tok(.., ty)
      | MultiConvert(.., ty)
      | Undefined(ty) => ty,
    }
  }

  pub fn get_type_mut(&mut self) -> &mut AscriptType {
    use GraphNode::*;
    match self {
      Add(.., ty)
      | Sub(.., ty)
      | Mul(.., ty)
      | Div(.., ty)
      | Map(.., ty)
      | Vec(.., ty)
      | Trim(.., ty)
      | Sym(.., ty)
      | Num(.., ty)
      | Str(.., ty)
      | Bool(.., ty)
      | TokRule(ty)
      | TokSym(.., ty)
      | Tok(.., ty)
      | MultiConvert(.., ty)
      | Undefined(ty) => ty,
    }
  }
}

impl ValueObj for GraphNode {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    use GraphNode::*;
    match self {
      Add(..) => "AddNode",
      Sub(..) => "SubNode",
      Mul(..) => "MulNode",
      Div(..) => "DivNode",
      Map(..) => "MapNode",
      Vec(..) => "VecNode",
      Sym(..) => "SymNode",
      Num(..) => "NumNode",
      Str(..) => "StrNode",
      Trim(..) => "TrimNode",
      Bool(..) => "BoolNode",
      MultiConvert(..) => "MultiConvertNode",
      TokRule(..) => "TokRuleNode",
      TokSym(..) | Tok(..) => "TokNode",
      Undefined(..) => "UndefinedNode",
    }
  }

  fn get_keys<'scope>(&'scope self) -> &'static [&'static str] {
    &["left", "right", "key", "index", "init", "ast_type"]
  }

  fn get_val<'scope>(&'scope self, key: &str, core: &radlr_core::IStringStore) -> Value<'scope> {
    use GraphNode::*;
    match key {
      "left" => match self {
        Add(left, ..) => Value::Obj(left.as_ref()),
        Sub(left, ..) => Value::Obj(left.as_ref()),
        Mul(left, ..) => Value::Obj(left.as_ref()),
        Div(left, ..) => Value::Obj(left.as_ref()),
        Map(key, ..) => Value::Obj(key.as_ref()),
        _ => Value::None,
      },
      "right" => match self {
        Add(_, right, ..) => Value::Obj(right.as_ref()),
        Sub(_, right, ..) => Value::Obj(right.as_ref()),
        Mul(_, right, ..) => Value::Obj(right.as_ref()),
        Div(_, right, ..) => Value::Obj(right.as_ref()),
        Map(_, val, ..) => Value::Obj(val.as_ref()),
        _ => Value::None,
      },
      "start" => match self {
        Trim(_, start, ..) => Value::Num(*start as f64),
        _ => Value::None,
      },
      "end" => match self {
        Trim(_, _, end, ..) => Value::Num(*end as f64),
        _ => Value::None,
      },
      "index" => match self {
        Sym(index, ..) => Value::Int(*index as isize),
        TokSym(index, ..) => Value::Int(*index as isize),
        MultiConvert(_, index) => Value::Int(index.get_multi_type_index().unwrap() as isize),
        _ => Value::None,
      },
      "is_last_ref" => match self {
        Sym(_, last_ref, ..) => Value::Int(*last_ref as isize),
        TokSym(_, last_ref, ..) => Value::Int(*last_ref as isize),
        _ => Value::None,
      },
      "to" => match self {
        MultiConvert(_, ty) => Value::Obj(ty),
        _ => Value::None,
      },
      "from" => match self {
        MultiConvert(val, ..) => Value::Obj(val.as_ref()),
        _ => Value::None,
      },
      "init" => match self {
        MultiConvert(val, ..) => Value::Obj(val.as_ref()),
        Map(_, val, ..) => Value::Obj(val.as_ref()),
        Trim(val, ..) => Value::Obj(val.as_ref()),
        Vec(val, ..) => Value::Obj(val),
        Str(val, ..) => val.as_ref().map(convert_rcto_value).unwrap_or(Value::None),
        Bool(val, ..) => val.as_ref().map(convert_rcto_value).unwrap_or(Value::None),
        Num(val, ..) => val.as_ref().map(convert_rcto_value).unwrap_or(Value::None),
        _ => Value::None,
      },
      "ast_type" => Value::Obj(self.get_type()),
      _ => GraphNode::get_type(self).get_val(key, core),
    }
  }
}

fn convert_rcto_value(v: &Rc<GraphNode>) -> Value {
  match v.as_ref() {
    GraphNode::Undefined(_) => Value::None,
    v => Value::Obj(v),
  }
}
