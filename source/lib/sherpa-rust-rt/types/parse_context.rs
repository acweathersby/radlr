use super::{
  ast::{AstObject, AstSlot, AstStackSlice, Reducer},
  bytecode::{FAIL_STATE_FLAG, NORMAL_STATE_FLAG},
  *,
};
use crate::bytecode_parser::{get_next_action, DebugEvent, DebugFn};
use std::{fmt::Debug, sync::Arc};

#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct Goto {
  pub goto_fn: *const usize,
  pub state:   u32,
  pub meta:    u32,
}

impl Default for Goto {
  fn default() -> Self {
    Self { goto_fn: 0 as *const usize, state: 0, meta: 0 }
  }
}

type GetBlockFunction<T> = extern "C" fn(
  self_: &mut T,
  &mut *const u8,
  &mut *const u8,
  &mut *const u8,
  &mut *const u8,
  &mut *const u8,
  &mut *const u8,
);

#[repr(C)]
pub struct ParseContext<T: ByteReader, M = u32> {
  // Input data ----------
  /// The head of the input block
  pub begin_ptr:      usize,
  /// The the end of the last shifted token
  pub anchor_ptr:     usize,
  /// The the start of the evaluated token, which may be
  /// the same as base_ptr unless we are using peek shifts.
  pub base_ptr:       usize,
  /// The the start of the evaluated token, which may be
  /// the same as base_ptr unless we are using peek shifts.
  pub head_ptr:       usize,
  /// The start of all unevaluated characters
  pub scan_ptr:       usize,
  /// The end of the input block
  pub end_ptr:        usize,
  /// The number of characters that comprize the current
  /// token. This should be 0 if the tok_id is also 0
  pub tok_len:        usize,
  // Goto stack data -----
  pub goto_stack_ptr: *mut Goto,
  pub goto_size:      u32,
  pub goto_free:      u32,
  // Parse objects ----------------
  pub get_input_info: GetBlockFunction<T>,
  pub reader:         *mut T,
  // User context --------
  pub meta_ctx:       *mut M,
  pub custom_lex:     fn(&mut T, &mut M, &ParseContext<T, M>) -> (u32, u32, u32),
  // Line info ------------
  /// The offset of the last line character recognized that proceeds the anchor
  pub start_line_off: u32,
  /// The offset of the last line character recognized that proceeds the chkp
  pub chkp_line_off:  u32,
  /// The offset of the last line character recognized that proceeds the tail
  pub end_line_off:   u32,
  /// The number of line character recognized that proceed the anchor
  pub start_line_num: u32,
  /// The number of line character recognized that proceed the chkp
  pub chkp_line_num:  u32,
  /// The number of line character recognized that proceed the tail
  pub end_line_num:   u32,
  // Parser State ----------
  /// When reducing, stores the the number of of symbols to reduce.
  pub sym_len:        u32,
  /// Tracks whether the context is a fail mode or not.
  pub state:          u32,
  /// Set to the value of a production when a rule is reduced, or
  pub prod_id:        u32,
  /// Set to the value of a token when one is recognized. Also stores the number
  /// of symbols that are to be reduced.
  pub tok_id:         u32,
  /// When reducing, stores the rule id that is being reduced.
  pub rule_id:        u32,
  pub line_incr:      u8,
  pub is_active:      bool,
  // Miscellaneous
  pub in_peek_mode:   bool,
}

impl<T: ByteReader, M> ParseContext<T, M> {
  pub fn reset(&mut self) {
    self.anchor_ptr = 0;
    self.scan_ptr = 0;
    self.tok_len = 0;
    self.head_ptr = 0;
    self.base_ptr = 0;
    self.end_ptr = 0;
    self.begin_ptr = 0;
    self.goto_size = 0;
    self.goto_free = 0;
    self.start_line_num = 0;
    self.chkp_line_num = 0;
    self.end_line_num = 0;
    self.state = 0;
    self.is_active = true;
    self.in_peek_mode = false;
    self.goto_stack_ptr = 0 as usize as *mut Goto;
  }

  pub fn set_meta(&mut self, meta: *mut M) {
    self.meta_ctx = meta;
  }

  pub unsafe fn get_meta_mut(&mut self) -> &mut M {
    &mut *self.meta_ctx
  }

  pub unsafe fn get_meta(&self) -> &M {
    &*self.meta_ctx
  }

  /// The following methods are used exclusively by the
  /// the rust functions in [sherpa::runtime::parser_functions.rs]

  #[inline]
  pub fn in_fail_mode(&self) -> bool {
    self.state == FAIL_STATE_FLAG
  }

  #[inline]
  pub fn set_fail_mode_to(&mut self, is_in_fail_mode: bool) {
    self.state = if is_in_fail_mode { FAIL_STATE_FLAG } else { NORMAL_STATE_FLAG }
  }

  #[inline]
  pub fn in_peek_mode(&self) -> bool {
    self.in_peek_mode
  }

  #[inline]
  pub fn set_peek_mode_to(&mut self, is_in_peek_mode: bool) {
    self.in_peek_mode = is_in_peek_mode;
  }

  #[inline]
  pub fn get_production(&self) -> u32 {
    self.prod_id
  }

  #[inline]
  pub fn set_production_to(&mut self, production: u32) {
    self.prod_id = production;
  }

  #[inline]
  pub fn is_scanner(&self) -> bool {
    self.rule_id > 0
  }

  pub fn set_is_scanner(&mut self, is_scanner: bool) {
    self.rule_id = is_scanner as u32;
  }

  pub fn get_curr_line_num(&self) -> u32 {
    self.start_line_num
  }

  pub fn get_curr_line_offset(&self) -> u32 {
    self.start_line_off
  }

  pub fn get_anchor_offset(&self) -> u32 {
    (self.anchor_ptr - self.begin_ptr) as u32
  }

  pub fn get_token_length(&self) -> u32 {
    (self.tok_len) as u32
  }

  pub fn get_token_offset(&self) -> u32 {
    (self.head_ptr - self.begin_ptr) as u32
  }

  pub fn get_token_line_number(&self) -> u32 {
    self.start_line_num
  }

  pub fn get_token_line_offset(&self) -> u32 {
    self.start_line_off
  }

  pub fn get_production_id(&self) -> u32 {
    self.prod_id
  }

  /// Returns shift data from current context state.
  pub fn get_shift_data(&self) -> ParseAction {
    ParseAction::Shift {
      anchor_byte_offset: self.get_anchor_offset(),
      token_byte_offset:  self.get_token_offset(),
      token_byte_length:  self.get_token_length(),
      token_line_offset:  self.get_curr_line_offset(),
      token_line_count:   self.get_curr_line_num(),
    }
  }
}

impl<T: ByteReader, M> Default for ParseContext<T, M> {
  fn default() -> Self {
    Self {
      anchor_ptr:     0,
      scan_ptr:       0,
      tok_len:        0,
      head_ptr:       0,
      base_ptr:       0,
      end_ptr:        0,
      prod_id:        0,
      begin_ptr:      0,
      end_line_num:   0,
      start_line_num: 0,
      chkp_line_num:  0,
      chkp_line_off:  0,
      end_line_off:   0,
      start_line_off: 0,
      state:          0,
      tok_id:         0,
      sym_len:        0,
      rule_id:        0,
      goto_size:      0,
      goto_free:      0,
      line_incr:      0,
      in_peek_mode:   false,
      is_active:      false,
      goto_stack_ptr: 0 as *mut Goto,
      meta_ctx:       0 as *mut M,
      custom_lex:     Self::default_custom_lex,
      get_input_info: Self::default_get_input_info,
      reader:         0 as *mut T,
    }
  }
}

impl<T: ByteReader, M> ParseContext<T, M> {
  pub fn new_bytecode(reader: &mut T) -> Self {
    Self {
      custom_lex: Self::default_custom_lex,
      get_input_info: Self::default_get_input_info,
      reader: reader,
      ..Default::default()
    }
  }

  extern "C" fn default_get_input_info(
    _: &mut T,
    _: &mut *const u8,
    _: &mut *const u8,
    _: &mut *const u8,
    _: &mut *const u8,
    _: &mut *const u8,
    _: &mut *const u8,
  ) {
  }

  pub(crate) fn default_custom_lex(_: &mut T, _: &mut M, _: &Self) -> (u32, u32, u32) {
    (0, 0, 0)
  }
}

impl<T: MutByteReader + ByteReader, M> ParseContext<T, M> {
  pub fn get_reader_mut(&mut self) -> &mut T {
    unsafe { (&mut *self.reader) as &mut T }
  }
}

impl<T: ByteReader, M> ParseContext<T, M> {
  pub fn get_reader(&self) -> &T {
    unsafe { (&*self.reader) as &T }
  }
}

impl<T: ByteReader + UTF8Reader, M> ParseContext<T, M> {
  pub fn get_str(&self) -> &str {
    unsafe { (*self.reader).get_str() }
  }
}

#[derive(Debug)]
#[repr(C, u64)]
pub enum ParseResult<Node> {
  Complete((Node, TokenRange, TokenRange)),
  Error(TokenRange, Vec<(Node, TokenRange, TokenRange)>),
  NeedMoreInput(Vec<(Node, TokenRange, TokenRange)>),
}

pub enum ShiftsAndSkipsResult {
  Accepted {
    shifts: Vec<String>,
    skips:  Vec<String>,
  },

  IncorrectProduction {
    shifts: Vec<String>,
    skips: Vec<String>,
    expected_prod_id: u32,
    actual_prod_id: u32,
  },

  FailedParse(SherpaParseError),
}

pub trait SherpaParser<R: ByteReader + MutByteReader, M> {
  /// Returns true of the `head_ptr` is positioned at the end of the input.
  ///
  /// That is `head_ptr - beg_ptr == input.len()`
  fn head_at_end(&self) -> bool;

  /// Returns the byte length of active token
  fn get_token_length(&self) -> u32;

  /// Returns the byte offset of the head of the active token
  fn get_token_offset(&self) -> u32;

  /// Returns the 0 indexed line number active token
  fn get_token_line_number(&self) -> u32;

  /// Returns the offset the newline character proceeding
  /// the active token
  fn get_token_line_offset(&self) -> u32;

  /// Returns the production id of the most recently reduced symbols
  fn get_production_id(&self) -> u32;

  /// Parse input up to the next required parse action and return
  /// its value.
  fn get_next_action(&mut self, debug: &mut Option<DebugFn>) -> ParseAction;

  /// Returns a reference to the internal Reader
  fn get_reader(&self) -> &R;

  /// Returns a reference to the input string
  fn get_input(&self) -> &str;

  fn init_parser(&mut self, entry_point: u32);

  fn get_ctx(&self) -> &ParseContext<R, M>;

  fn parse_ast<Node: AstObject>(
    &mut self,
    reducers: &[Reducer<R, M, Node>],
    debug: &mut Option<DebugFn>,
  ) -> Result<AstSlot<Node>, SherpaParseError> {
    let mut ast_stack: Vec<AstSlot<Node>> = vec![];
    loop {
      match self.get_next_action(debug) {
        ParseAction::Accept { .. } => {
          return Ok(ast_stack.pop().unwrap());
        }
        ParseAction::Reduce { rule_id, symbol_count, .. } => {
          let reduce_fn = reducers[rule_id as usize];
          let len = ast_stack.len();
          let count = symbol_count as usize;
          reduce_fn(
            &self.get_ctx(),
            &AstStackSlice::from_slice(&mut ast_stack[(len - count)..len]),
          );
          ast_stack.resize(len - (count - 1), AstSlot::<Node>::default());
        }
        ParseAction::Shift {
          anchor_byte_offset,
          token_byte_offset,
          token_byte_length,
          token_line_offset,
          token_line_count,
        } => {
          let peek = TokenRange {
            len: token_byte_offset - anchor_byte_offset,
            off: anchor_byte_offset,
            ..Default::default()
          };

          let tok = TokenRange {
            len:      token_byte_length,
            off:      token_byte_offset,
            line_num: token_line_count,
            line_off: token_line_offset,
          };
          ast_stack.push(AstSlot(Node::default(), tok, peek));
        }
        ParseAction::Error { .. } => {
          return Err(SherpaParseError {
            inline_message: Default::default(),
            last_production: 0,
            loc: Default::default(),
            message: Default::default(),
          });
        }
        _ => {
          return Err(SherpaParseError {
            inline_message: Default::default(),
            last_production: 0,
            loc: Default::default(),
            message: Default::default(),
          });
        }
      }
    }
  }

  fn collect_shifts_and_skips(
    &mut self,
    entry_point: u32,
    target_production_id: u32,
    debug: &mut Option<DebugFn>,
  ) -> ShiftsAndSkipsResult {
    self.init_parser(entry_point);

    let mut shifts = vec![];
    let mut skips = vec![];
    loop {
      match self.get_next_action(debug) {
        ParseAction::Accept { production_id } => {
          #[cfg(debug_assertions)]
          if let Some(debug) = debug {
            debug(&DebugEvent::Complete { production_id });
          }
          break if production_id != target_production_id {
            ShiftsAndSkipsResult::IncorrectProduction {
              shifts,
              skips,
              expected_prod_id: target_production_id,
              actual_prod_id: production_id,
            }
          } else {
            ShiftsAndSkipsResult::Accepted { shifts, skips }
          };
        }
        ParseAction::Error { last_input, .. } => {
          #[cfg(debug_assertions)]
          if let Some(debug) = debug {
            debug(&DebugEvent::Failure {});
          }
          let mut token: Token = last_input.to_token(self.get_reader());

          token.set_source(Arc::new(Vec::from(self.get_input().to_string().as_bytes())));
          break ShiftsAndSkipsResult::FailedParse(SherpaParseError {
            message: "Could not recognize the following input:".to_string(),
            inline_message: "".to_string(),
            loc: token,
            last_production: self.get_production_id(),
          });
        }
        ParseAction::Fork { .. } => {
          panic!("No implementation of fork resolution is available")
        }
        ParseAction::Shift { anchor_byte_offset, token_byte_length, token_byte_offset, .. } => {
          if (token_byte_offset - anchor_byte_offset) > 0 {
            skips.push(
              self.get_input()[anchor_byte_offset as usize..(token_byte_offset) as usize]
                .to_string(),
            );
            #[cfg(debug_assertions)]
            if let Some(debug) = debug {
              debug(&DebugEvent::SkipToken {
                offset_start: anchor_byte_offset as usize,
                offset_end:   token_byte_offset as usize,
                string:       self.get_input(),
              });
            }
          }
          let offset_start = token_byte_offset as usize;
          let offset_end = (token_byte_offset + token_byte_length) as usize;

          #[cfg(debug_assertions)]
          if let Some(debug) = debug {
            debug(&DebugEvent::ShiftToken { offset_start, offset_end, string: self.get_input() });
          }
          shifts.push(self.get_input()[offset_start..offset_end].to_string());
        }
        ParseAction::Reduce { rule_id, .. } =>
        {
          #[cfg(debug_assertions)]
          if let Some(debug) = debug {
            debug(&DebugEvent::Reduce { rule_id });
          }
        }
        _ => panic!("Unexpected Action!"),
      }
    }
  }
}

pub struct ByteCodeParser<'a, R: ByteReader + MutByteReader, M> {
  ctx:   ParseContext<R, M>,
  stack: Vec<u32>,
  bc:    &'a [u8],
}

impl<'a, R: ByteReader + MutByteReader, M> ByteCodeParser<'a, R, M> {
  pub fn new(reader: &'a mut R, bc: &'a [u8]) -> Self {
    ByteCodeParser { ctx: ParseContext::<R, M>::new_bytecode(reader), stack: vec![], bc }
  }
}

impl<'a, R: ByteReader + MutByteReader + UTF8Reader, M> SherpaParser<R, M>
  for ByteCodeParser<'a, R, M>
{
  fn get_ctx(&self) -> &ParseContext<R, M> {
    &self.ctx
  }

  fn head_at_end(&self) -> bool {
    self.ctx.head_ptr == self.get_reader().len()
  }

  fn get_token_length(&self) -> u32 {
    self.ctx.get_token_length()
  }

  fn get_token_offset(&self) -> u32 {
    self.ctx.get_token_offset()
  }

  fn get_token_line_number(&self) -> u32 {
    self.ctx.start_line_num
  }

  fn get_token_line_offset(&self) -> u32 {
    self.ctx.start_line_off
  }

  fn get_production_id(&self) -> u32 {
    self.ctx.prod_id
  }

  fn get_reader(&self) -> &R {
    self.ctx.get_reader()
  }

  fn get_input(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(self.get_reader().get_bytes()) }
  }

  fn init_parser(&mut self, entry_point: u32) {
    self.stack = vec![0, 0, NORMAL_STATE_FLAG, entry_point];
  }

  fn get_next_action(&mut self, debug: &mut Option<DebugFn>) -> ParseAction {
    let ByteCodeParser { ctx, stack, bc } = self;
    get_next_action(ctx, stack, bc, debug)
  }
}
