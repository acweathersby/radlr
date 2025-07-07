use std::{
  cell::Cell,
  collections::{hash_map::DefaultHasher, HashMap},
  fmt::Debug,
  hash::Hasher,
  rc::Rc,
};

use super::{CSTHashes, CSTNode};

pub struct CSTStore {
  _internal: Cell<HashMap<u64, Rc<CSTNode>>>,
}

impl Debug for CSTStore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_tuple("CSTStore");
    s.field(unsafe { &*self._internal.as_ptr() });
    s.finish()
  }
}

impl Default for CSTStore {
  fn default() -> Self {
    Self { _internal: Cell::new(HashMap::default()) }
  }
}

impl CSTStore {
  fn node_hash(node: &CSTNode) -> u64 {
    let mut hasher = DefaultHasher::default();
    node.dedup_hash(&mut hasher);
    hasher.finish()
  }

  pub fn get_unique(&self, candidate: CSTNode) -> Rc<CSTNode> {
    let _store = unsafe { &mut *self._internal.as_ptr() };

    match &candidate {
      CSTNode::Token(..) => match _store.entry(Self::node_hash(&candidate)) {
        std::collections::hash_map::Entry::Occupied(e) => e.get().clone(),
        std::collections::hash_map::Entry::Vacant(e) => e.insert(Rc::new(candidate.clone())).clone(),
      },
      // --------------------------------------
      CSTNode::NonTerm(node) => {
        if node.symbols.len() <= 3 {
          match _store.entry(Self::node_hash(&candidate)) {
            std::collections::hash_map::Entry::Occupied(e) => e.get().clone(),
            std::collections::hash_map::Entry::Vacant(e) => e.insert(Rc::new(candidate.clone())).clone(),
          }
        } else {
          Rc::new(candidate.clone())
        }
      }
      CSTNode::PlaceholderNonTerm(..) => Rc::new(candidate.clone()),
      CSTNode::Alts(_) => Rc::new(candidate.clone()),
    }
  }
}

#[test]
pub fn creates_single_instances_of_nodes() {
  let store = CSTStore::default();

  let nt_node_1 = CSTNode::NonTerm(super::NonTermNode { id: 1, rule: 1, length: 1, symbols: vec![] });
  let nt_node_2 = CSTNode::NonTerm(super::NonTermNode { id: 1, rule: 1, length: 1, symbols: vec![] });

  let u_nt_1 = store.get_unique(nt_node_1);
  let u_nt_2 = store.get_unique(nt_node_2);

  assert_ne!(Rc::as_ptr(&u_nt_1), Rc::as_ptr(&u_nt_2));

  let tk_node_1 = CSTNode::Token(Default::default());
  let tk_node_2 = CSTNode::Token(Default::default());

  let u_tk_1 = store.get_unique(tk_node_1);
  let u_tk_2 = store.get_unique(tk_node_2);

  assert_eq!(Rc::as_ptr(&u_tk_1), Rc::as_ptr(&u_tk_2));

  let tk_node_1 = CSTNode::Token(Default::default());
  let tk_node_2 = CSTNode::Token(Default::default());

  let u_tk_1 = store.get_unique(tk_node_1);
  let u_tk_2 = store.get_unique(tk_node_2);

  assert_ne!(Rc::as_ptr(&u_tk_1), Rc::as_ptr(&u_tk_2));
}
