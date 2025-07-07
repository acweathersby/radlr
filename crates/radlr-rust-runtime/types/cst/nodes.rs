//! A CST implementation, heavily inspired by rowan, suitable for
//! handling ambiguous and error corrected trees (technically a shared packed
//! parser forest)

use super::super::*;
use std::{
  collections::hash_map::DefaultHasher,
  fmt::Debug,
  hash::{Hash, Hasher},
  rc::Rc,
};

/// A hash of the node that only takes into account non-spacial values.
pub trait CSTHashes: Hash {
  /// Represent the abstract value of the node, without considering details
  /// that make the node unique.
  ///
  ///  For instance, All non-terminal nodes that are created from the same rule
  /// are canonically identical, even if their internal symbols differ or they
  /// have a different set of skipped symbols
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H);
  /// Represents the concrete value of the node
  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H);
}

#[derive(Clone, Debug)]
pub enum CSTErrorNodeType {
  MissingToken(u16),
  ErroneousInput,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum NodeType {
  Missing,
  Errata,
  Skipped,
  Token,
  Nonterm,
  Alternative,
  Alternatives,
  None,
}

pub trait NodeTraits {
  fn len(&self) -> usize;
  fn ty(&self) -> NodeType;
  fn entropy(&self) -> isize;
  fn is_token(&self) -> bool {
    matches!(self.ty(), NodeType::Token)
  }
  fn is_missing(&self) -> bool {
    matches!(self.ty(), NodeType::Missing)
  }
  fn is_skipped(&self) -> bool {
    matches!(self.ty(), NodeType::Skipped)
  }
  fn is_nonterm(&self) -> bool {
    matches!(self.ty(), NodeType::Nonterm)
  }
  fn is_alts(&self) -> bool {
    matches!(self.ty(), NodeType::Alternatives)
  }
  fn is_alt(&self) -> bool {
    matches!(self.ty(), NodeType::Alternative)
  }
  fn as_token(&self) -> Option<&TokenNode> {
    None
  }
  fn as_nonterm(&self) -> Option<&NonTermNode> {
    None
  }
  fn as_alt(&self) -> Option<&Alternative> {
    None
  }
  fn as_alts(&self) -> Option<&Alts> {
    None
  }
}

#[derive(Clone)]
pub enum CSTNode {
  /// A general terminal token type.
  Token(TokenNode),
  /// A non-terminal node produce from the reduction of a rule.
  NonTerm(NonTermNode),
  /// A non-terminal that does not have any internal nodes. Used when overcoming
  /// errors. Us a fuzzer to replace this value with actual data.
  PlaceholderNonTerm(u32),
  /// A series of alternative interpretations of a sequence of
  /// characters
  Alts(Alts),
}

impl Eq for CSTNode {}

impl PartialEq for CSTNode {
  fn eq(&self, other: &Self) -> bool {
    let mut hasher_a = DefaultHasher::new();
    let mut hasher_b = DefaultHasher::new();
    self.canonical_hash(&mut hasher_a);
    other.canonical_hash(&mut hasher_b);
    hasher_a.finish() == hasher_b.finish()
  }
}

impl CSTHashes for CSTNode {
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    use CSTNode::*;
    match self {
      Token(tok) => tok.canonical_hash(state),
      NonTerm(nt) => nt.canonical_hash(state),
      Alts(m) => m.canonical_hash(state),
      _ => {}
    }
  }

  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    use CSTNode::*;
    match self {
      Token(tok) => tok.dedup_hash(state),
      NonTerm(nt) => nt.dedup_hash(state),
      Alts(m) => m.dedup_hash(state),
      _ => {}
    }
  }
}

impl Hash for CSTNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dedup_hash(state)
  }
}

impl NodeTraits for CSTNode {
  fn len(&self) -> usize {
    use CSTNode::*;
    match self {
      Token(tok) => tok.len(),
      NonTerm(nt) => nt.length as usize,
      Alts(m) => m.length as usize,
      _ => 0,
    }
  }

  fn ty(&self) -> NodeType {
    use CSTNode::*;
    match self {
      Alts { .. } => NodeType::Alternatives,
      NonTerm { .. } => NodeType::Nonterm,
      Token(tok) => tok.ty(),
      PlaceholderNonTerm(_) => NodeType::Missing,
    }
  }

  fn entropy(&self) -> isize {
    use CSTNode::*;
    match self {
      Token(tok) => tok.entropy(),
      NonTerm(_) => 0,
      PlaceholderNonTerm(_) => 1,
      Alts(m) => m.alternatives.first().map(|f| f.entropy).unwrap_or_default(),
    }
  }

  fn as_alts(&self) -> Option<&Alts> {
    if let CSTNode::Alts(alt) = &self {
      Some(alt)
    } else {
      None
    }
  }

  fn as_token(&self) -> Option<&TokenNode> {
    if let CSTNode::Token(tok) = &self {
      Some(tok)
    } else {
      None
    }
  }

  fn as_nonterm(&self) -> Option<&NonTermNode> {
    if let CSTNode::NonTerm(nt) = &self {
      Some(nt)
    } else {
      None
    }
  }
}

impl Debug for CSTNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("[0x{:0>10X}]", self as *const CSTNode as usize))?;
    match self {
      Self::Token(node) => match node.ty() {
        NodeType::Errata => {
          f.write_str(node.str())?;
          f.write_str(" [errata]:")?;
          f.write_str(&node.len().to_string())
        }
        NodeType::Missing => {
          f.write_str(&node.tok_id().to_string())?;
          f.write_str(" [missing]")?;
          f.write_str(&node.len().to_string())
        }
        NodeType::Skipped => {
          f.write_str(node.str())?;
          f.write_str(" [skipped]")?;
          f.write_str(&node.len().to_string())
        }
        NodeType::Token => {
          f.write_str("\"")?;
          f.write_str(node.str())?;
          f.write_str("\":")?;
          f.write_str(&node.len().to_string())
        }
        _ => unreachable!(),
      },
      Self::PlaceholderNonTerm(..) => f.write_str("[NT?]"),
      Self::Alts(multi) => multi.fmt(f),
      Self::NonTerm(nt) => nt.fmt(f),
    }
  }
}

#[derive(Clone, Hash)]
pub struct NonTermNode {
  pub id:      u16,
  pub rule:    u16,
  pub length:  u32,
  pub symbols: Vec<Rc<CSTNode>>,
}

impl CSTHashes for NonTermNode {
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }

  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.symbols.iter().for_each(|s| s.dedup_hash(state))
  }
}

impl Debug for NonTermNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut hasher = DefaultHasher::new();
    self.canonical_hash(&mut hasher);

    let mut dedup = DefaultHasher::new();
    self.dedup_hash(&mut dedup);

    f.write_fmt(format_args!("[nonterm: {} @ {} : {} - {}]", self.id, self as *const _ as usize, hasher.finish(), self.length))?;
    let mut list = f.debug_list();
    for obj in &self.symbols {
      list.entry(obj);
    }
    list.finish()
  }
}

impl NonTermNode {
  pub fn new(id: u16, rule: u16, symbols: Vec<Rc<CSTNode>>, length: u32) -> Self {
    Self { id, rule, symbols, length }
  }

  pub fn typed(id: u16, rule: u16, symbols: Vec<Rc<CSTNode>>, length: usize) -> CSTNode {
    debug_assert!(length <= u32::MAX as usize);

    CSTNode::NonTerm(Self::new(id, rule, symbols, length as u32))
  }

  pub fn len(&self) -> usize {
    self.length as usize
  }
}

#[derive(Clone, Hash)]
pub struct Alts {
  pub length:       u32,
  pub alternatives: Vec<Rc<Alternative>>,
  #[cfg(debug_assertions)]
  pub meta_label:   &'static str,
}

impl Alts {
  pub fn new(alternatives: Vec<Rc<Alternative>>, meta_label: &'static str) -> Self {
    let length = alternatives[0].length;

    #[cfg(debug_assertions)]
    {
      Self { alternatives, length, meta_label }
    }
    #[cfg(not(debug_assertions))]
    Self { alternatives, length }
  }

  pub fn typed(alternatives: Vec<Rc<Alternative>>, meta_label: &'static str) -> CSTNode {
    CSTNode::Alts(Self::new(alternatives, meta_label))
  }
}

impl CSTHashes for Alts {
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.alternatives[0].canonical_hash(state)
  }

  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.alternatives.iter().for_each(|s| s.dedup_hash(state))
  }
}

impl Debug for Alts {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    #[cfg(debug_assertions)]
    f.write_fmt(format_args!("[multi: {}]", self.meta_label))?;
    #[cfg(not(debug_assertions))]
    f.write_fmt(format_args!("[multi]"))?;
    let mut list = f.debug_list();
    for obj in &self.alternatives {
      list.entry(obj);
    }
    list.finish()
  }
}

#[derive(Clone, Hash)]
pub struct Alternative {
  pub length:  u32,
  pub entropy: isize,
  pub symbols: Vec<Rc<CSTNode>>,
}

impl CSTHashes for Alternative {
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.symbols.iter().for_each(|s| s.canonical_hash(state))
  }

  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.symbols.iter().for_each(|s| s.dedup_hash(state))
  }
}

impl Debug for Alternative {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut hasher = DefaultHasher::new();
    self.symbols.iter().for_each(|s| s.dedup_hash(&mut hasher));
    f.write_fmt(format_args!("[alternative [{}] : {}]", self.entropy, hasher.finish()))?;
    let mut list = f.debug_list();
    for obj in &self.symbols {
      list.entry(obj);
    }
    list.finish()
  }
}

pub trait CSTtoASTProducer<I: ParserInput, ASTNode>: ASTProducer<I, ASTNode> {
  fn cst_to_ast(&self, node: &CSTNode) -> Option<ASTBaseNode<ASTNode>> {
    use CSTNode::*;
    match node {
      Token(tk) => match tk.ty() {
        NodeType::Missing | NodeType::Token => Some(ASTBaseNode::Token(Rc::new(tk.clone()))),
        _ => None,
      },
      PlaceholderNonTerm(..) => {
        todo!()
      }
      NonTerm(non_term) => {
        let reduce_fn = self.get_reduce_functions()[non_term.rule as usize];

        let mut symbols = non_term.symbols.iter().filter_map(|n| self.cst_to_ast(n));

        Some(reduce_fn(&mut symbols))
      }
      Alts(multi) => {
        if let Some(first) = multi.alternatives.first() {
          first.symbols.iter().filter_map(|n| self.cst_to_ast(n)).next()
        } else {
          None
        }
      }
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct TokenNode(_Token_);

impl Hash for TokenNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dedup_hash(state)
  }
}

impl CSTHashes for TokenNode {
  fn canonical_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.tok_id().hash(state);
  }

  fn dedup_hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.tok_id().hash(state);
    self.str().hash(state);
  }
}

impl NodeTraits for TokenNode {
  fn len(&self) -> usize {
    use _Token_::*;
    match &self.0 {
      MissingToken { .. } => 0,
      LargeToken { val, len, .. } => {
        if *len < u8::MAX {
          *len as usize
        } else {
          val.len()
        }
      }
      SmallToken { len, .. } => *len as usize,
    }
  }

  fn entropy(&self) -> isize {
    use _Token_::*;
    match &self.0 {
      MissingToken { entropy, .. } => *entropy as isize,
      _ => self.len() as isize,
    }
  }

  fn ty(&self) -> NodeType {
    use _Token_::*;
    match &self.0 {
      MissingToken { .. } => NodeType::Missing,
      LargeToken { ty, .. } | SmallToken { ty, .. } => *ty,
    }
  }

  fn as_token(&self) -> Option<&TokenNode> {
    Some(self)
  }
}

impl TokenNode {
  pub fn skipped_type(tok_id: u16, val: &str) -> Self {
    Self(_Token_::new(NodeType::Skipped, tok_id, val))
  }

  pub fn token_type(tok_id: u16, val: &str) -> Self {
    Self(_Token_::new(NodeType::Token, tok_id, val))
  }

  pub fn missing_type(tok_id: u16, entropy: usize) -> Self {
    Self(_Token_::new_missing(tok_id, entropy))
  }

  pub fn error_type(val: &str) -> Self {
    Self(_Token_::new(NodeType::Errata, 0, val))
  }

  pub fn str(&self) -> &str {
    use _Token_::*;
    match &self.0 {
      MissingToken { .. } => "Missing",
      LargeToken { val, .. } => val.as_str(),
      SmallToken { len, data, .. } => unsafe { std::str::from_utf8_unchecked(&data[0..*len as usize]) },
    }
  }

  pub fn tok_id(&self) -> usize {
    use _Token_::*;
    match &self.0 {
      MissingToken { tok_id, .. } | SmallToken { tok_id, .. } | LargeToken { tok_id, .. } => *tok_id as usize,
    }
  }
}

#[derive(Debug, Clone)]
enum _Token_ {
  LargeToken { ty: NodeType, val: Rc<String>, tok_id: u16, len: u8 },
  SmallToken { ty: NodeType, len: u8, tok_id: u16, data: [u8; 11] },
  MissingToken { tok_id: u16, entropy: usize },
}

impl Default for _Token_ {
  fn default() -> Self {
    Self::SmallToken { ty: NodeType::Token, len: 0, tok_id: 0, data: [0; 11] }
  }
}

impl _Token_ {
  pub fn new_missing(tok_id: u16, entropy: usize) -> Self {
    Self::MissingToken { tok_id, entropy }
  }

  pub fn new(ty: NodeType, tok_id: u16, val: &str) -> Self {
    debug_assert!(matches!(ty, NodeType::Errata | NodeType::Token | NodeType::Skipped | NodeType::Missing));

    let bytes = val.as_bytes();
    match bytes.len() {
      len @ 0..=11 => {
        let mut data = [0; 11];
        for i in 0..len {
          data[i] = bytes[i];
        }
        Self::SmallToken { ty, len: len as u8, tok_id, data }
      }
      len @ 12..=254 => Self::LargeToken { ty, val: Rc::new(val.to_string()), tok_id, len: len as u8 },
      _ => Self::LargeToken { ty, val: Rc::new(val.to_string()), tok_id, len: 255 },
    }
  }
}
