use std::{cmp::Ordering, rc::Rc};

use crate::parsers::{fork::CHAR_USAGE_SCORE, Parser};

use super::*;

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug)]
pub struct ParserStackTrackers {
  // Input pointers -----------------------------------------------------------
  /// The head of the input window.
  pub begin_ptr:    usize,
  /// Positioned at the end of the last shifted token
  pub anchor_ptr:   usize,
  /// Positioned at the start of the current incoming token, and following any
  /// characters that have been skipped. (This only differs from `anchor_ptr`
  /// when peeking, in which case there may have been skipped tokens that the
  /// peeking process has encountered and shifted over)
  pub sym_ptr:      usize,
  /// The start of all unevaluated bytes. This differs from `sym_ptr` when
  /// using scanner states to evaluate incoming bytes
  pub input_ptr:    usize,
  /// The end of the input window. This is a fixed reference that should
  /// not change during parsing unless the end of the input window has been
  /// reached and a larger window is requested.
  pub end_ptr:      usize,
  /// The number of characters that comprize the current
  /// token. This should be 0 if the tok_id is also 0
  pub tok_id:       u32,
  /// The byte length of the current token
  pub tok_byte_len: u32,
  /// The byte length of the most current input character. This is usually 1
  /// byte unless the input contains UTF codepoints outside the ASCII range.
  pub byte_len:     u32,
}

impl From<&ParserContext> for ParserStackTrackers {
  fn from(v: &ParserContext) -> Self {
    Self {
      begin_ptr:    v.begin_ptr,
      anchor_ptr:   v.anchor_ptr,
      sym_ptr:      v.sym_ptr,
      input_ptr:    v.input_ptr,
      end_ptr:      v.end_ptr,
      tok_id:       v.tok_id,
      tok_byte_len: v.tok_byte_len,
      byte_len:     v.byte_len,
    }
  }
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
/// Stores a stack of active states and lexer/parser symbol pointers.
pub struct ParserContext {
  // Goto stack data ----------------------------------------------------------
  pub stack:        Vec<ParserState>,
  // Input pointers -----------------------------------------------------------
  /// The head of the input window.
  pub begin_ptr:    usize,
  /// Positioned at the end of the last shifted token
  pub anchor_ptr:   usize,
  /// Positioned at the start of the current incoming token, and following any
  /// characters that have been skipped. (This only differs from `anchor_ptr`
  /// when peeking, in which case there may have been skipped tokens that the
  /// peeking process has encountered and shifted over)
  pub sym_ptr:      usize,
  /// The start of all unevaluated bytes. This differs from `sym_ptr` when
  /// using scanner states to evaluate incoming bytes
  pub input_ptr:    usize,
  /// The end of the input window. This is a fixed reference that should
  /// not change during parsing unless the end of the input window has been
  /// reached and a larger window is requested.
  pub end_ptr:      usize,
  /// The unique id of the token that is set to be produced by the scanner
  pub tok_id:       u32,
  /// The byte length of the current token
  pub tok_byte_len: u32,
  /// The byte length of the most current input character. This is usually 1
  /// byte unless the input contains UTF codepoints outside the ASCII range.
  pub byte_len:     u32,

  /// Flags -------------------------------------------------------------------
  pub is_finished: bool,

  pub nonterm: u32,

  pub goal_nonterm: u32,

  // Line info ------------ TO BE DEPRECATED
  /// The offset of the last line character recognized that proceeds the anchor
  pub start_line_off:  u32,
  /// The offset of the last line character recognized that proceeds the chkp
  pub chkp_line_off:   u32,
  /// The offset of the last line character recognized that proceeds the tail
  pub end_line_off:    u32,
  /// The number of line character recognized that proceed the anchor
  pub start_line_num:  u32,
  /// The number of line character recognized that proceed the chkp
  pub chkp_line_num:   u32,
  /// The number of line character recognized that proceed the tail
  pub end_line_num:    u32,
  /// If  this is a non-zero value, then when the parser fails to match an
  /// incoming token, it will insert a zero-length token with an id of
  /// `default_id` into the token stream, and then attempt to continue parsing.
  pub recovery_tok_id: u32,

  pub node: Option<Rc<CSTNode>>,
}

impl Default for ParserContext {
  fn default() -> Self {
    ParserContext {
      stack:           vec![],
      begin_ptr:       0,
      anchor_ptr:      0,
      sym_ptr:         0,
      input_ptr:       0,
      end_ptr:         0,
      tok_byte_len:    0,
      byte_len:        0,
      chkp_line_num:   0,
      chkp_line_off:   0,
      end_line_num:    0,
      end_line_off:    0,
      nonterm:         0,
      start_line_num:  0,
      start_line_off:  0,
      tok_id:          0,
      recovery_tok_id: 0,
      goal_nonterm:    u32::MAX,
      is_finished:     false,
      node:            None,
    }
  }
}

impl ParserContext {
  pub fn current_tok(&self) -> TokenRange {
    TokenRange {
      len:      self.tok_byte_len as u32,
      off:      self.sym_ptr as u32,
      line_num: self.start_line_num,
      line_off: self.start_line_off,
    }
  }

  pub fn push_state(&mut self, mut state: ParserState) {
    state.info.stack_address = self.stack.len() as u16;
    self.stack.push(state);
  }

  pub fn pop_state(&mut self) -> ParserState {
    unsafe { self.stack.pop().unwrap_unchecked() }
  }
}

pub trait ForkableContext: QueuedContext {
  fn symbols(&mut self) -> &mut Vec<(ParserState, Rc<CSTNode>)>;
  fn ctx(&self) -> &ParserContext;
  fn ctx_mut(&mut self) -> &mut ParserContext;
  fn entropy(&self) -> &isize;
  fn entropy_mut(&mut self) -> &mut isize;
  fn split(&self) -> Self;
  fn handle_shift(&mut self);
  fn prority(&self) -> usize {
    usize::MAX - self.ctx().sym_ptr
  }
}

impl<T: ForkableContext> QueuedContext for T {
  fn queued_priority(&self) -> usize {
    usize::MAX - self.ctx().sym_ptr
  }
}

#[derive(Clone, Debug)]
pub struct ForkContext {
  pub(crate) entropy: isize,
  pub(crate) offset:  usize,
  pub(crate) ctx:     ParserContext,
  pub(crate) symbols: Vec<(ParserState, Rc<CSTNode>)>,
}

impl ForkableContext for Box<ForkContext> {
  fn handle_shift(&mut self) {}

  #[inline]
  fn ctx(&self) -> &ParserContext {
    &self.ctx
  }

  #[inline]
  fn ctx_mut(&mut self) -> &mut ParserContext {
    &mut self.ctx
  }

  #[inline]
  fn entropy(&self) -> &isize {
    &self.entropy
  }

  #[inline]
  fn entropy_mut(&mut self) -> &mut isize {
    &mut self.entropy
  }

  #[inline]
  fn symbols(&mut self) -> &mut Vec<(ParserState, Rc<CSTNode>)> {
    &mut self.symbols
  }

  #[inline]
  fn split(&self) -> Self {
    Box::new(ForkContext {
      offset:  self.offset,
      entropy: self.entropy,
      ctx:     self.ctx.clone(),
      symbols: self.symbols.clone(),
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
  Normal,
  CodepointDiscard { start_offset: usize, count: usize },
  SymbolDiscard { start_offset: usize, end_offset: usize, count: usize },
  Unrecoverable(usize),
}

/// This Context is used by the recoverable parse to handle error correction
/// process during parsing.
pub type RecCTX = Box<RecoverableContext>;

pub fn create_recovery_ctx<I: ParserInput, DB: ParserProducer<I>>(
  input: &mut I,
  parser: &mut Box<dyn Parser<I>>,
  entry: EntryPoint,
) -> Result<RecCTX, ParserError> {
  let mut ctx = parser.init(entry)?;
  let len = input.len();
  ctx.end_ptr = len;

  Ok(Box::new(RecoverableContext {
    offset: 0,
    entropy: len as isize * CHAR_USAGE_SCORE,
    symbols: vec![],
    ctx,
    mode: RecoveryMode::Normal,
    failed_state: Default::default(),
    last_failed_state: Default::default(),
  }))
}

#[derive(Clone, Debug)]
pub struct RecoverableContext {
  pub(crate) offset:     usize,
  pub entropy:           isize,
  pub ctx:               ParserContext,
  pub symbols:           Vec<(ParserState, Rc<CSTNode>)>,
  pub mode:              RecoveryMode,
  pub last_failed_state: ParserState,
  pub failed_state:      ParserState,
}

impl RecoverableContext {
  pub fn nodes(&self) -> impl Iterator<Item = &Rc<CSTNode>> {
    self.symbols.iter().map(|(_, node)| node)
  }

  pub fn node_len(&self) -> usize {
    self.symbols.iter().fold(0, |val, (_, sym)| sym.len() + val)
  }

  pub fn is_single_node(&self) -> bool {
    let mut sym_nodes = 0;
    for (_, sym) in &self.symbols {
      match sym.ty() {
        NodeType::Nonterm | NodeType::Token | NodeType::Skipped | NodeType::Alternatives => {
          sym_nodes += 1;
        }
        _ => {}
      }
    }
    sym_nodes == 1
  }

  pub fn into_first() {}
}

impl ForkableContext for RecCTX {
  fn handle_shift(&mut self) {
    self.last_failed_state = Default::default();
    self.mode = RecoveryMode::Normal;
  }

  #[inline]
  fn ctx(&self) -> &ParserContext {
    &self.ctx
  }

  #[inline]
  fn ctx_mut(&mut self) -> &mut ParserContext {
    &mut self.ctx
  }

  #[inline]
  fn entropy(&self) -> &isize {
    &self.entropy
  }

  #[inline]
  fn entropy_mut(&mut self) -> &mut isize {
    &mut self.entropy
  }

  #[inline]
  fn symbols(&mut self) -> &mut Vec<(ParserState, Rc<CSTNode>)> {
    &mut self.symbols
  }

  #[inline]
  fn split(&self) -> Self {
    let mut ctx = self.ctx.clone();
    ctx.is_finished = false;

    Box::new(RecoverableContext {
      ctx,
      mode: RecoveryMode::Unrecoverable(0),
      symbols: self.symbols.clone(),
      ..*self.as_ref()
    })
  }

  /*  #[inline]
  fn get_offset(&self) -> usize {
    self.as_ref().get_offset()
  } */

  //#[inline]
  //fn set_offset(&mut self, offset: usize) {
  //  self.as_mut().set_offset(offset);
  //} /*  */
}

impl PartialEq for RecoverableContext {
  fn eq(&self, _other: &Self) -> bool {
    false
  }
}

impl PartialOrd for RecoverableContext {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    let a = (self.entropy, usize::MAX - self.offset);
    let b = (other.entropy, usize::MAX - other.offset);

    if a < b {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Greater)
    }
  }
}

impl Eq for RecoverableContext {}
impl Ord for RecoverableContext {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}
