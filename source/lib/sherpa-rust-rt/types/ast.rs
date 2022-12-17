use super::*;

pub type ReduceFunction<T> = fn(slots: &mut AstSlots<(T, Token, Token)>);

#[deprecated]
pub type ReduceFunction_old<T> = fn(args: &mut Vec<HCObj<T>>, tok: Token);

/// Names of default AST types
pub static DEFAULT_AST_TYPE_NAMES: [&str; 27] = [
  "NODES", "STRING", "STRINGS", "F64", "F32", "I64", "I32", "I16", "I8", "U64", "U32", "U16", "U8",
  "F32Vec", "F64Vec", "I64Vec", "I32Vec", "I16Vec", "I8Vec", "U64Vec", "U32Vec", "U16Vec", "U8Vec",
  "TOKEN", "TOKENS", "BOOL", "NONE",
];

#[derive(Debug, Clone)]

pub enum HCObj<T: 'static> {
  NONE,
  LAZY(Box<Lazy>),
  NODE(T),
  NODES(Vec<T>),
  STRING(String),
  STRINGS(Vec<String>),
  F64(f64),
  F32(f32),
  I64(i64),
  I32(i32),
  I16(i16),
  I8(i8),
  U64(u64),
  U32(u32),
  U16(u16),
  U8(u8),
  F32Vec(Vec<f32>),
  F64Vec(Vec<f64>),
  I64Vec(Vec<i64>),
  I32Vec(Vec<i32>),
  I16Vec(Vec<i16>),
  I8Vec(Vec<i8>),
  U64Vec(Vec<u64>),
  U32Vec(Vec<u32>),
  U16Vec(Vec<u16>),
  U8Vec(Vec<u8>),
  TOKEN(Token),
  TOKENS(Vec<Token>),
  BOOL(bool),
  OBJECTS(Vec<HCObj<T>>),
}

macro_rules! into_vec {
  ($fn_name:ident, $out_type: ty, $type:ident) => {
    pub fn $fn_name(self) -> Vec<$out_type> {
      if let HCObj::$type(v) = self {
        v
      } else {
        vec![]
      }
    }
  };
}

macro_rules! to_numeric {
  ($fn_name:ident,  $Num:ty) => {
    fn $fn_name(&self) -> $Num {
      if self.is_numeric() || matches!(self, HCObj::STRING(..) | HCObj::TOKEN(..)) {
        match self {
          HCObj::STRING(str) => str.parse::<i64>().unwrap_or(0) as $Num,
          HCObj::TOKEN(tok) => tok.to_numeric_or_length() as $Num,
          HCObj::F64(val) => *val as $Num,
          HCObj::F32(val) => *val as $Num,
          HCObj::I64(val) => *val as $Num,
          HCObj::I32(val) => *val as $Num,
          HCObj::I16(val) => *val as $Num,
          HCObj::U64(val) => *val as $Num,
          HCObj::U32(val) => *val as $Num,
          HCObj::U16(val) => *val as $Num,
          HCObj::U8(val) => *val as $Num,
          HCObj::BOOL(val) => (*val as usize) as $Num,
          _ => 0 as $Num,
        }
      } else {
        0 as $Num
      }
    }
  };
}

impl<T: HCObjTrait> HCObj<T> {
  into_vec!(into_nodes, T, NODES);

  into_vec!(into_f64_vec, f64, F64Vec);

  into_vec!(into_f32_vec, f32, F32Vec);

  into_vec!(into_i64_vec, i64, I64Vec);

  into_vec!(into_i32_vec, i32, I32Vec);

  into_vec!(into_i16_vec, i16, I16Vec);

  into_vec!(into_i8_vec, i8, I8Vec);

  into_vec!(into_u64_vec, u64, U64Vec);

  into_vec!(into_u32_vec, u32, U32Vec);

  into_vec!(into_u16_vec, u16, U16Vec);

  into_vec!(into_u8_vec, u8, U8Vec);

  into_vec!(into_tokens, Token, TOKENS);

  pub fn is_numeric(&self) -> bool {
    matches!(
      self,
      HCObj::F64(..)
        | HCObj::F32(..)
        | HCObj::I64(..)
        | HCObj::I32(..)
        | HCObj::I16(..)
        | HCObj::U64(..)
        | HCObj::U32(..)
        | HCObj::U16(..)
        | HCObj::U8(..)
    )
  }

  pub fn into_node(self) -> Option<T> {
    match self {
      Self::NODE(node) => Some(node),
      _ => None,
    }
  }
}

pub trait HCObjTrait
where
  Self: Sized,
{
  fn to_string(&self) -> String {
    String::new()
  }

  fn to_f64(&self) -> f64 {
    0.0
  }

  fn to_f32(&self) -> f32 {
    0.0
  }

  fn to_i64(&self) -> i64 {
    0
  }

  fn to_i32(&self) -> i32 {
    0
  }

  fn to_i16(&self) -> i16 {
    0
  }
  fn to_i8(&self) -> i8 {
    0
  }

  fn to_u64(&self) -> u64 {
    0
  }

  fn to_u32(&self) -> u32 {
    0
  }
  fn to_u16(&self) -> u16 {
    0
  }
  fn to_u8(&self) -> u8 {
    0
  }

  fn to_bool(&self) -> bool {
    false
  }

  fn to_token(&self) -> Token {
    Token::empty()
  }
  fn into_strings(self) -> Vec<String> {
    Default::default()
  }
}

impl<T: HCObjTrait> HCObjTrait for HCObj<T> {
  to_numeric!(to_i8, i8);

  to_numeric!(to_i16, i16);

  to_numeric!(to_i32, i32);

  to_numeric!(to_i64, i64);

  to_numeric!(to_u8, u8);

  to_numeric!(to_u16, u16);

  to_numeric!(to_u32, u32);

  to_numeric!(to_u64, u64);

  to_numeric!(to_f32, f32);

  to_numeric!(to_f64, f64);

  fn into_strings(self) -> Vec<String> {
    match self {
      HCObj::STRINGS(strings) => strings,
      _ => Default::default(),
    }
  }

  fn to_string(&self) -> String {
    match self {
      HCObj::NODE(node) => node.to_string(),
      &HCObj::BOOL(val) => {
        if val {
          String::from("true")
        } else {
          String::from("false")
        }
      }
      HCObj::STRING(string) => string.to_owned(),
      HCObj::TOKEN(val) => val.to_string(),
      _ => String::from(""),
    }
  }

  fn to_token(&self) -> Token {
    match self {
      HCObj::TOKEN(val) => val.clone(),
      _ => Token::empty(),
    }
  }

  fn to_bool(&self) -> bool {
    self.to_u8() != 0
  }
}

#[derive(Debug, Clone)]

pub struct Lazy {
  tok:           Token,
  entry_pointer: u32,
  bytecode:      &'static [u32],
}

// impl Lazy<_> {
// fn parse_scope(&self) {
// let string = "";
// let reader = UTF8StringReader::new(string.as_bytes());
// let result = completer(reader, self.bytecode, self.entry_pointer,
// self.functions); }
// }
