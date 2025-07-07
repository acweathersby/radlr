use radlr_core::{
  proxy::{OrderedMap, OrderedSet},
  CachedString,
  IString,
  IStringStore,
};
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
};

#[derive(Default, Clone, Copy, Debug)]
pub enum Value<'scope> {
  Obj(&'scope dyn ValueObj),
  Num(f64),
  Int(isize),
  Str(IString),
  #[default]
  None,
}

impl<'a> ToValue for Value<'a> {
  fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
    *self
  }
}

macro_rules! integer_value {
  ($val:ty) => {
    impl<'scope> From<$val> for Value<'scope> {
      fn from(value: $val) -> Self {
        Value::Int(value as isize)
      }
    }

    impl ToValue for $val {
      fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
        (*self).into()
      }
    }
  };
}

macro_rules! float_value {
  ($val:ty) => {
    impl<'scope> From<$val> for Value<'scope> {
      fn from(value: $val) -> Self {
        Value::Num(value as f64)
      }
    }

    impl ToValue for $val {
      fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
        (*self).into()
      }
    }
  };
}
integer_value!(u8);
integer_value!(u16);
integer_value!(u32);
integer_value!(u64);
integer_value!(usize);

integer_value!(i8);
integer_value!(i16);
integer_value!(i32);
integer_value!(i64);
integer_value!(isize);

float_value!(f32);
float_value!(f64);

impl<'scope> From<IString> for Value<'scope> {
  fn from(value: IString) -> Self {
    Value::Str(value)
  }
}

pub trait ValueObj: Debug {
  /// Returns the `Value` associated with the given `key` or `Value::None` if
  /// the key does not exist.
  #[allow(unused)]
  fn get_val<'scope>(&'scope self, key: &str, s_store: &IStringStore) -> Value<'scope> {
    Value::None
  }

  #[allow(unused)]
  fn get_index<'scope>(&'scope self, index: usize, s_store: &IStringStore) -> Value<'scope> {
    Value::None
  }

  #[allow(unused)]
  fn vals<'scope>(&'scope self, funct: &mut dyn FnMut(String, Value<'scope>)) {}

  /// Returns a unique type name that can be access through this objects
  /// `.#type` property.
  fn get_type<'scope>(&'scope self) -> &'scope str {
    "undefinedObj"
  }

  /// Returns a  list of keys that can be used to debug invalid property access
  /// errors.
  fn get_keys<'scope>(&'scope self) -> &'static [&'static str] {
    &["type", "len"]
  }

  /// Return an iterable object that can be used in `.iter#<fn_name>()` property
  /// calls. Returning an empty vec disables this property.
  fn get_iter<'scope>(&'scope self, _: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    Default::default()
  }

  fn get_len<'scope>(&'scope self) -> usize {
    1
  }
}

impl<T: ValueObj> ToValue for T {
  fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
    Value::Obj(self)
  }
}

pub trait ToValue {
  fn into_val<'scope>(&'scope self, s_store: &IStringStore) -> Value<'scope>;
}

impl ToValue for String {
  fn into_val<'scope>(&'scope self, s_store: &IStringStore) -> Value<'scope> {
    Value::Str(self.intern(s_store))
  }
}

impl ToValue for &String {
  fn into_val<'scope>(&'scope self, s_store: &IStringStore) -> Value<'scope> {
    Value::Str(self.intern(s_store))
  }
}

impl ToValue for &str {
  fn into_val<'scope>(&'scope self, s_store: &IStringStore) -> Value<'scope> {
    Value::Str(self.intern(s_store))
  }
}

impl ToValue for IString {
  fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
    Value::Str(*self)
  }
}

impl ToValue for &IString {
  fn into_val<'scope>(&'scope self, _: &IStringStore) -> Value<'scope> {
    Value::Str(**self)
  }
}

impl<'a, K: ToValue + From<String> + Eq + PartialEq + Hash + Ord + Debug, T: ToValue + Debug> ValueObj for HashMap<K, T> {
  fn get_val<'scope>(&'scope self, key: &str, s_store: &IStringStore) -> Value<'scope> {
    let v = key.to_string().into();
    match self.get(&v) {
      None => Value::None,
      Some(val) => val.into_val(s_store),
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "obj"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().map(|(k, v)| (k.into_val(s_store), v.into_val(s_store))).collect()
  }
}

impl<'a, K: ToValue + From<String> + Debug + Ord, T: ToValue + Debug> ValueObj for OrderedMap<K, T> {
  fn get_val<'scope>(&'scope self, key: &str, s_store: &IStringStore) -> Value<'scope> {
    let v = key.to_string().into();
    match self.get(&v) {
      None => Value::None,
      Some(val) => val.into_val(s_store),
    }
  }

  fn get_index<'scope>(&'scope self, index: usize, s_store: &IStringStore) -> Value<'scope> {
    match self.iter().enumerate().find(|(i, _)| *i == index) {
      None => Value::None,
      Some((_, (_, val))) => val.into_val(s_store),
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "obj"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().map(|(k, v)| (k.into_val(s_store), v.into_val(s_store))).collect()
  }
}

impl<'a, T: ToValue + Debug> ValueObj for Vec<T> {
  fn get_index<'scope>(&'scope self, index: usize, s_store: &IStringStore) -> Value<'scope> {
    if index < self.len() {
      self[index].into_val(s_store)
    } else {
      Value::None
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "list"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().enumerate().map(|(i, v)| (Value::Int(i as isize), v.into_val(s_store))).collect()
  }
}

impl<'a, T: ToValue + Debug + Default> ValueObj for OrderedSet<T> {
  fn get_index<'scope>(&'scope self, _: usize, _: &IStringStore) -> Value<'scope> {
    Value::None
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "ordered_set"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().enumerate().map(|(i, v)| (Value::Int(i as isize), v.into_val(s_store))).collect()
  }
}

impl<'a, T: ToValue + Debug + Default> ValueObj for HashSet<T> {
  fn get_index<'scope>(&'scope self, _: usize, _: &IStringStore) -> Value<'scope> {
    Value::None
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "hash_set"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().enumerate().map(|(i, v)| (Value::Int(i as isize), v.into_val(s_store))).collect()
  }
}

impl<'a, T: ToValue + Debug> ValueObj for &[T] {
  fn get_index<'scope>(&'scope self, index: usize, s_store: &IStringStore) -> Value<'scope> {
    if index < self.len() {
      self[index].into_val(s_store)
    } else {
      Value::None
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "list"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len()
  }

  fn get_iter<'scope>(&'scope self, s_store: &IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.iter().enumerate().map(|(i, v)| (Value::Int(i as isize), v.into_val(s_store))).collect()
  }
}

#[macro_export]
macro_rules! formatted_typed_set {
  ($name: ident, $element_type:ty, $type_name:literal, $attr:meta) => {
    #[$attr]
    pub struct $name(pub(crate) Set<$element_type>);

    impl std::ops::Deref for $name {
      type Target = Set<$element_type>;

      fn deref(&self) -> &Set<$element_type> {
        &self.0
      }
    }

    impl std::ops::DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Set<$element_type> {
        &mut self.0
      }
    }

    impl ValueObj for $name {
      fn get_index<'scope>(&'scope self, index: usize, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        Value::None
      }

      fn get_len<'scope>(&'scope self) -> usize {
        self.0.get_len()
      }

      fn get_type<'scope>(&'scope self) -> &str {
        $type_name
      }

      fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        Value::None
      }

      fn get_iter<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
        self.0.get_iter(s_store)
      }
    }
  };
  ($name: ident, $element_type:ty, $type_name:literal) => {
    formatted_typed_set!($name, $element_type, $type_name, derive(Debug, Default));
  };
}

#[macro_export]
macro_rules! formatted_typed_ordered_set {
  ($name: ident, $element_type:ty, $type_name:literal, $attr:meta) => {
    #[$attr]
    pub struct $name(pub(crate) OrderedSet<$element_type>);

    impl std::ops::Deref for $name {
      type Target = OrderedSet<$element_type>;

      fn deref(&self) -> &OrderedSet<$element_type> {
        &self.0
      }
    }

    impl std::ops::DerefMut for $name {
      fn deref_mut(&mut self) -> &mut OrderedSet<$element_type> {
        &mut self.0
      }
    }

    impl ValueObj for $name {
      fn get_index<'scope>(&'scope self, index: usize, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_index(index, s_store)
      }

      fn get_len<'scope>(&'scope self) -> usize {
        self.0.get_len()
      }

      fn get_type<'scope>(&'scope self) -> &str {
        $type_name
      }

      fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_val(key, s_store)
      }

      fn get_iter<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
        self.0.get_iter(s_store)
      }
    }
  };

  ($name: ident, $element_type:ty, $type_name:literal) => {
    formatted_typed_ordered_set!($name, $element_type, $type_name, derive(Debug, Default));
  };
}

#[macro_export]
macro_rules! formatted_typed_vector {
  ($name: ident, $element_type:ty, $type_name:literal, $attr:meta) => {
    #[$attr]
    pub struct $name(pub(crate) Vec<$element_type>);

    impl std::ops::Deref for $name {
      type Target = Vec<$element_type>;

      fn deref(&self) -> &Vec<$element_type> {
        &self.0
      }
    }

    impl std::ops::DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Vec<$element_type> {
        &mut self.0
      }
    }

    impl std::ops::Index<usize> for $name {
      type Output = $element_type;

      fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
      }
    }

    impl std::ops::IndexMut<usize> for $name {
      fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
      }
    }

    impl ValueObj for $name {
      fn get_index<'scope>(&'scope self, index: usize, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_index(index, s_store)
      }

      fn get_len<'scope>(&'scope self) -> usize {
        self.0.get_len()
      }

      fn get_type<'scope>(&'scope self) -> &str {
        $type_name
      }

      fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_val(key, s_store)
      }

      fn get_iter<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
        self.0.get_iter(s_store)
      }
    }
  };
  ($name: ident, $element_type:ty, $type_name:literal) => {
    formatted_typed_vector!($name, $element_type, $type_name, derive(Debug, Default));
  };
}

#[macro_export]
macro_rules! formatted_typed_hash_map {
  ($name: ident, $key_type:ty, $val_type:ty, $type_name:literal) => {
    #[derive(Debug, Default)]
    pub struct $name(pub(crate) HashMap<$key_type, $val_type>);

    impl std::ops::Deref for $name {
      type Target = HashMap<$key_type, $val_type>;

      fn deref(&self) -> &HashMap<$key_type, $val_type> {
        &self.0
      }
    }

    impl std::ops::DerefMut for $name {
      fn deref_mut(&mut self) -> &mut HashMap<$key_type, $val_type> {
        &mut self.0
      }
    }

    impl ValueObj for $name {
      fn get_index<'scope>(&'scope self, index: usize, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_index(index, s_store)
      }

      fn get_len<'scope>(&'scope self) -> usize {
        self.0.get_len()
      }

      fn get_type<'scope>(&'scope self) -> &str {
        $type_name
      }

      fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_val(key, s_store)
      }

      fn get_iter<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
        self.0.get_iter(s_store)
      }
    }
  };
}

#[macro_export]
macro_rules! formatted_typed_ordered_map {
  ($name: ident, $key_type:ty, $val_type:ty, $type_name:literal) => {
    #[derive(Debug, Default)]
    pub struct $name(pub(crate) OrderedMap<$key_type, $val_type>);

    impl std::ops::Deref for $name {
      type Target = OrderedMap<$key_type, $val_type>;

      fn deref(&self) -> &OrderedMap<$key_type, $val_type> {
        &self.0
      }
    }

    impl std::ops::DerefMut for $name {
      fn deref_mut(&mut self) -> &mut OrderedMap<$key_type, $val_type> {
        &mut self.0
      }
    }

    impl ValueObj for $name {
      fn get_index<'scope>(&'scope self, index: usize, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_index(index, s_store)
      }

      fn get_len<'scope>(&'scope self) -> usize {
        self.0.get_len()
      }

      fn get_type<'scope>(&'scope self) -> &str {
        $type_name
      }

      fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
        self.0.get_val(key, s_store)
      }

      fn get_iter<'scope>(&'scope self, s_store: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
        self.0.get_iter(s_store)
      }
    }
  };
}
