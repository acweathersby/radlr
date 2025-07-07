use crate::{AscriptDatabase, AscriptType, AscriptTypes};
use radlr_core::CachedString;
use radlr_formatter::*;
use std::fmt::Debug;

#[derive(Debug)]
pub struct AscriptMulti {
  pub(crate) converts_from:     Vec<usize>,
  pub(crate) types:             AscriptTypes,
  pub(crate) requires_template: bool,
  pub(crate) name:              String,
  pub(crate) used:              bool,
  pub(crate) root:              bool,
}

impl ValueObj for AscriptMulti {
  fn get_keys<'scope>(&'scope self) -> &'static [&'static str] {
    &["name", "types"]
  }

  fn get_val<'scope>(&'scope self, key: &str, s_store: &radlr_core::IStringStore) -> Value<'scope> {
    match key {
      "name" => Value::Str(self.name.intern(s_store)),
      "types" => Value::Obj(&self.types),
      "requires_template" => Value::Int(self.requires_template as isize),
      "is_used" => Value::Int(self.used as isize),
      "is_root" => Value::Int(self.root as isize),
      "converts_from" => Value::Obj(&self.converts_from),
      _ => Value::None,
    }
  }

  fn get_type<'scope>(&'scope self) -> &'scope str {
    "AscriptMultiEnum"
  }
}

#[derive(Debug)]
/// Provides services to remap AnyTypes to enums of Value
pub struct AscriptMultis<'db> {
  #[allow(unused)]
  pub(crate) db:    &'db AscriptDatabase,
  pub(crate) types: Vec<AscriptMulti>,
  pub(crate) len:   usize,
}

impl<'db> AscriptMultis<'db> {
  pub fn new(db: &'db AscriptDatabase) -> Self {
    let mut used_indices = Vec::from_iter(db.multi_type_lu.iter().cloned().map(|_| 0));
    Self {
      db,
      len: used_indices.len(),
      types: db
        .multi_types
        .iter()
        .enumerate()
        .map(|(i, (..))| {
          let remapped_index = db.multi_type_lu[i];
          let entry = used_indices.get_mut(remapped_index);
          let (name, types, converts) = &db.multi_types[remapped_index];

          let (used, root) = if let Some(entry) = entry {
            *entry += 1;
            (true, remapped_index == i)
          } else {
            (false, false)
          };

          let types = AscriptTypes(types.iter().map(|t| AscriptType::Scalar(*t)).collect());

          let name = db.db.nonterm_friendly_name(*name).to_string(db.db.string_store());

          let converts_from = converts.iter().cloned().filter(|j| *j != remapped_index && *j != i).collect();
          let name = format!("{}_Value", name);
          let requires_template = types.iter().any(|t| t.requires_template(db));

          AscriptMulti { name, requires_template, types, used, root, converts_from }
        })
        .collect(),
    }
  }
}

impl<'a> ValueObj for AscriptMultis<'a> {
  fn get_type<'scope>(&'scope self) -> &'scope str {
    "multi_enums"
  }

  fn get_len<'scope>(&'scope self) -> usize {
    self.len
  }

  fn get_index<'scope>(&'scope self, index: usize, _: &radlr_core::IStringStore) -> Value<'scope> {
    //let remapped_index = self.db.multi_type_lu[index];
    Value::Obj(&self.types[index])
  }

  fn get_iter<'scope>(&'scope self, _: &radlr_core::IStringStore) -> Vec<(Value<'scope>, Value<'scope>)> {
    self.types.iter().filter(|i| i.used && i.root).enumerate().map(|(i, any)| (Value::Int(i as isize), Value::Obj(any))).collect()
  }
}
