use super::*;
use crate::utf8::get_token_class_from_codepoint;
use std::{
  alloc::{alloc, dealloc, Layout},
  ffi::c_void,
  fmt::Debug,
};

const STACK_32_BIT_SIZE: usize = 128;
pub struct ParseContext<T: ByteReader + MutByteReader> {
  pub peek: (TokenRange, u32),
  pub anchor: (TokenRange, u32),
  pub assert: (TokenRange, u32),
  pub action_ptr: *mut ParseAction,
  pub reader: *mut T,
  pub input_block: usize,
  pub stack_base: *const u64,
  pub stack_top: usize,
  pub stack_size: u32,
  pub input_block_length: u32,
  pub input_block_offset: u32,
  pub production: u32,
  pub state: u32,
  pub in_peek_mode: u32,
  pub local_state_stack: Vec<u32>,
}

impl<T: ByteReader + MutByteReader> ParseContext<T> {
  pub fn new(reader: &mut T) -> Self {
    let mut ctx = Self {
      peek: Default::default(),
      anchor: Default::default(),
      assert: Default::default(),
      action_ptr: 0 as *mut ParseAction,
      stack_base: [].as_mut_ptr(),
      stack_top: 0,
      state: NORMAL_STATE_FLAG,
      production: 0,
      input_block: 0,
      input_block_length: 0,
      input_block_offset: 0,
      stack_size: 0,
      reader,
      in_peek_mode: 0,
      local_state_stack: vec![],
    };
    ctx
  }

  pub fn bytecode_context() -> Self {
    let mut ctx = Self {
      peek: Default::default(),
      anchor: Default::default(),
      assert: Default::default(),
      action_ptr: 0 as *mut ParseAction,
      stack_base: [].as_mut_ptr(),
      stack_top: 0,
      stack_size: (STACK_32_BIT_SIZE as u32) >> 1,
      state: 0,
      production: 0,
      input_block: 0,
      input_block_length: 0,
      input_block_offset: 0,
      reader: 0 as *mut T,
      local_state_stack: vec![],
      in_peek_mode: 0,
    };

    ctx
  }

  /// The following methods are used exclusively by the
  /// the rust functions in [sherpa::runtime::parser_functions.rs]

  #[inline]
  pub fn in_fail_mode(&self) -> bool {
    self.input_block_offset > 0
  }

  #[inline]
  pub fn set_fail_mode_to(&mut self, is_in_fail_mode: bool) {
    self.input_block_offset = if is_in_fail_mode { 1 } else { 0 }
  }

  #[inline]
  pub fn in_peek_mode(&self) -> bool {
    self.in_peek_mode > 0
  }

  #[inline]
  pub fn set_peek_mode_to(&mut self, is_in_peek_mode: bool) {
    self.in_peek_mode = is_in_peek_mode as u32;
  }

  #[inline]
  pub fn is_interrupted(&self) -> bool {
    self.action_ptr as usize > 0
  }

  #[inline]
  pub fn set_interrupted_to(&mut self, is_interrupted: bool) {
    self.action_ptr = is_interrupted as usize as *mut ParseAction
  }

  /// Used by the bytecode interpreter
  #[inline]
  pub fn is_scanner(&self) -> bool {
    self.input_block > 0
  }

  /// Used by the bytecode interpreter
  #[inline]
  pub fn make_scanner(&mut self) {
    self.input_block = 1;
  }

  #[inline]
  pub fn get_active_state(&mut self) -> u32 {
    self.state as u32
  }

  #[inline]
  pub fn set_active_state_to(&mut self, state: u32) {
    self.state = state;
  }

  #[inline]
  pub fn get_production(&mut self) -> u32 {
    self.production
  }

  #[inline]
  pub fn set_production_to(&mut self, production: u32) {
    self.production = production;
  }

  #[inline]
  pub fn pop_state(&mut self) -> u32 {
    if self.stack_top > 0 {
      self.stack_top -= 1;
      return self.local_state_stack.pop().unwrap();
    } else {
      0
    }
  }

  #[inline]
  pub fn push_state(&mut self, state: u32) {
    self.local_state_stack.push(state);

    self.stack_top += 1;
  }

  #[inline]
  pub fn init_normal_state(&mut self, entry_point: u32) {
    self.stack_top = 0;

    self.push_state((NORMAL_STATE_FLAG | entry_point));
  }
}

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

pub const LLVM_BASE_STACK_SIZE: usize = 8;

#[derive(Clone)]
#[repr(C)]
pub struct LLVMParseContext<T: LLVMByteReader + ByteReader, M> {
  // Input data ----------
  pub token_ptr:       *mut u8,
  pub peek_ptr:        *mut u8,
  pub scan_ptr:        *mut u8,
  pub tok_input_len:   u32,
  pub peek_input_len:  u32,
  pub scan_input_len:  u32,
  pub tok_input_trun:  bool,
  pub peek_input_trun: bool,
  pub scan_input_trun: bool,
  // Miscellaneous
  pub in_peek_mode:    bool,
  // Offset info ----------
  /// The start of the portion of characters currently being recognized
  pub anchor_off:      u32,
  /// Maintains the start position of a token. The difference between this and the anchor
  /// offset determines the number characters that have been skipped.
  pub token_off:       u32,
  /// Represents the most advanced offset of  peeked characters
  pub peek_off:        u32,
  /// Maintains the reference to then end of a recognized tokens when in a scan context
  pub scan_anchor_off: u32,
  /// Represents the most advanced portion of scanned characters
  pub scan_off:        u32,
  /// Represents the byte length of the currently recognized symbol
  pub scan_len:        u32,
  /// Set to the value of a production when a rule is reduced, or
  pub prod_id:         u32,
  /// Set to the value of a token when one is recognized.
  pub tok_id:          u32,
  // Line info ------------
  /// The offset of the last line character recognized that proceeds the anchor offset
  pub anchor_line_off: u32,
  /// The number of line character recognized that proceed the anchor offset
  pub anchor_line_num: u32,
  /// The offset of the last line character recognized that proceeds the token offset
  pub tok_line_off:    u32,
  /// The number of line character recognized that proceed the token offset
  pub tok_line_num:    u32,
  /// The offset of the last line character recognized that proceeds the peek offset
  pub peek_line_off:   u32,
  /// The number of line character recognized that proceed the peek offset
  pub peek_line_num:   u32,
  // Goto stack data -----
  pub goto_stack_ptr:  *mut Goto,
  pub goto_size:       u32,
  pub goto_free:       u32,
  // Input data ----------
  pub get_input_info:  extern "C" fn(&mut T, u32, u32) -> InputInfo,
  // Reader --------------
  pub reader:          *mut T,
  // User context --------
  pub meta_ctx:        *mut M,
  pub custom_lex:      fn(&mut T, &mut M, &LLVMParseContext<T, M>) -> (u32, u32, u32),
  /// Tracks whether the context is a fail mode or not.
  pub state:           u32,
  pub is_active:       bool,
}

#[test]
fn llvm_context_is_152_bytes() {
  assert_eq!(std::mem::size_of::<LLVMParseContext<UTF8StringReader, u64>>(), 152)
}

impl<T: LLVMByteReader + ByteReader, M> Debug for LLVMParseContext<T, M> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut dbgstr = f.debug_struct("LLVMParseContext");
    dbgstr.field("token_ptr", &self.token_ptr);
    dbgstr.field("peek_ptr", &self.peek_ptr);
    dbgstr.field("scan_ptr", &self.scan_ptr);
    dbgstr.field("tok_input_len", &self.tok_input_len);
    dbgstr.field("peek_input_len", &self.peek_input_len);
    dbgstr.field("scan_input_len", &self.scan_input_len);
    dbgstr.field("anchor_off", &self.anchor_off);
    dbgstr.field("token_off", &self.token_off);
    dbgstr.field("peek_off", &self.peek_off);
    dbgstr.field("scan_anchor_off", &self.scan_anchor_off);
    dbgstr.field("scan_off", &self.scan_off);
    dbgstr.field("scan_len", &self.scan_len);
    dbgstr.field("tok_or_prod_id", &self.prod_id);
    dbgstr.field("anchor_line_off", &self.anchor_line_off);
    dbgstr.field("anchor_line_num", &self.anchor_line_num);
    dbgstr.field("tok_line_off", &self.tok_line_off);
    dbgstr.field("tok_line_num", &self.tok_line_num);
    dbgstr.field("peek_line_off", &self.peek_line_off);
    dbgstr.field("peek_line_num", &self.peek_line_num);
    dbgstr.field("state", &self.state);
    dbgstr.field("in_peek_mode", &self.in_peek_mode);
    dbgstr.field("is_active", &self.is_active);
    dbgstr.field("goto_stack_ptr", &self.goto_stack_ptr);
    dbgstr.field("goto_size", &self.goto_size);
    dbgstr.field("goto_used", &self.goto_free);
    dbgstr.field("get_input_info", &"FN Pointer".to_string());
    dbgstr.field("reader", &self.reader);
    dbgstr.field("meta_ctx", &self.meta_ctx);
    dbgstr.field("custom_lex", &"FN Pointer".to_string());
    dbgstr.finish()
  }
}

impl<T: LLVMByteReader + ByteReader, M> LLVMParseContext<T, M> {
  pub fn new() -> Self {
    Self {
      token_ptr:       0 as *mut u8,
      peek_ptr:        0 as *mut u8,
      scan_ptr:        0 as *mut u8,
      peek_input_trun: true,
      scan_input_trun: true,
      tok_input_trun:  true,
      tok_input_len:   0,
      peek_input_len:  0,
      scan_input_len:  0,
      anchor_off:      0,
      token_off:       0,
      peek_off:        0,
      scan_anchor_off: 0,
      scan_off:        0,
      scan_len:        0,
      prod_id:         0,
      anchor_line_off: 0,
      anchor_line_num: 0,
      tok_line_off:    0,
      tok_line_num:    0,
      peek_line_off:   0,
      peek_line_num:   0,
      state:           0,
      tok_id:          0,
      in_peek_mode:    false,
      is_active:       false,
      goto_stack_ptr:  0 as *mut Goto,
      goto_size:       0,
      goto_free:       0,
      get_input_info:  T::get_byte_block_at_cursor,
      reader:          0 as *mut T,
      meta_ctx:        0 as *mut M,
      custom_lex:      Self::default_custom_lex,
    }
  }

  fn get_source(&mut self) -> SharedSymbolBuffer {
    unsafe { (*self.reader).get_source() }
  }

  fn default_custom_lex(_: &mut T, _: &mut M, _: &Self) -> (u32, u32, u32) {
    (0, 0, 0)
  }
}

impl<T: UTF8Reader + LLVMByteReader + ByteReader, M> LLVMParseContext<T, M> {
  pub fn get_str<'a>(&'a self) -> &'a str {
    unsafe { (*self.reader).get_str() }
  }
}

#[no_mangle]
pub extern "C" fn sherpa_allocate_stack(byte_size: usize) -> *mut Goto {
  // Each goto slot is 16bytes, so we shift left num_of_slots by 4 to get the bytes size of
  // the stack.

  let layout = Layout::from_size_align(byte_size, 16).unwrap();

  unsafe {
    let ptr = alloc(layout) as *mut Goto;

    #[cfg(debug_assertions)]
    {
      eprintln!(
        "ALLOCATION OF {} bytes for {} slots at address: {:p}",
        byte_size,
        byte_size >> 4,
        ptr
      );
    }

    ptr
  }
}

pub trait AstSlot: Debug + Clone + Default + Sized {}

/// Used within an LLVM parser to provide access to intermediate AST
/// data stored on the stack within a dynamically resizable array.
#[repr(C)]
pub struct AstSlots<T: AstSlot> {
  stack_data: *mut T,
  stack_size: u32,
}

impl<T: AstSlot> AstSlots<T> {
  #[track_caller]
  fn get_pointer(&self, position: usize) -> *mut T {
    if position >= (self.stack_size as usize) {
      panic!(
        "Could not get AST node at slot ${} from stack with a length of {}",
        position, self.stack_size
      );
    }
    let slot_size = std::mem::size_of::<T>();
    // We are using the stack space for these slots,
    // which we ASSUME grows downward, hence the "higher" slots
    // are accessed through lower addresses.
    (self.stack_data as usize - (position * slot_size)) as *mut T
  }

  /// Assigns the given data to a garbage slot, ignoring any existing value
  /// the slot may contain. This is only used when shifting token data into
  /// an "empty" slot through the Shift action.
  unsafe fn assign_to_garbage(&self, position: usize, val: T) {
    let pointer = self.get_pointer(position);
    std::mem::forget(std::mem::replace(&mut (*pointer), val));
  }

  /// Moves the last slot into the first's slots position,
  /// and drops the values of all other slots.
  pub fn drop_all_but_last(&self) {
    if self.len() == 1 {
      return;
    }

    let last = self.take(self.len() - 1);
    self.assign(0, last);

    for index in 1..self.len() {
      self.take(index);
    }
  }

  pub fn assign(&self, position: usize, val: T) {
    unsafe {
      let pointer = self.get_pointer(position);
      *pointer = val;
    }
  }

  /// Removes the value at the given position from the stack and returns it.
  ///
  pub fn take(&self, position: usize) -> T {
    unsafe { std::mem::take(&mut (*self.get_pointer(position))) }
  }

  pub fn clone(&self, position: usize) -> T {
    unsafe { (*self.get_pointer(position)).clone() }
  }

  pub fn len(&self) -> usize {
    self.stack_size as usize
  }

  pub fn destroy(self) {
    self.to_vec();
  }

  pub fn to_vec(&self) -> Vec<T> {
    let mut output = vec![];
    for i in 0..self.stack_size {
      output.push(self.take(i as usize));
    }
    output
  }
}

impl<T: AstSlot> Debug for AstSlots<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut dbgstr = f.debug_struct("SlotSlice");
    dbgstr.field("stack_size", &self.stack_size);
    let slot_size = std::mem::size_of::<T>();
    dbgstr.field("[slot byte size]", &slot_size);
    for i in 0..self.stack_size {
      dbgstr.field(&format!("slot[{}]", i), &(self.clone(i as usize)));
    }

    dbgstr.finish()
  }
}

impl AstSlot for u32 {}

impl<V: AstSlot> AstSlot for (V, TokenRange, TokenRange) {}

pub unsafe fn llvm_map_shift_action<
  'a,
  T: LLVMByteReader + ByteReader + MutByteReader,
  M,
  V: AstSlot,
>(
  ctx: &mut LLVMParseContext<T, M>,
  action: &ParseAction,
  slots: &mut AstSlots<(V, TokenRange, TokenRange)>,
) {
  match *action {
    ParseAction::Shift {
      anchor_byte_offset,
      token_byte_offset,
      token_byte_length,
      token_line_offset,
      token_line_count,
      ..
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

      slots.assign_to_garbage(0, (V::default(), tok, peek));
    }
    _ => slots.assign_to_garbage(0, (V::default(), TokenRange::default(), TokenRange::default())),
  }
}

pub unsafe fn llvm_map_result_action<
  'a,
  T: LLVMByteReader + ByteReader + MutByteReader,
  M,
  Node: AstSlot,
>(
  ctx: &mut LLVMParseContext<T, M>,
  action: &ParseAction,
  slots: &mut AstSlots<(Node, TokenRange, TokenRange)>,
) -> ParseResult<Node> {
  match *action {
    ParseAction::Accept { .. } => {
      ParseResult::Complete(slots.take(0))
    }
    ParseAction::EndOfInput { .. } => {
      ParseResult::NeedMoreInput(slots.to_vec())
    }
    ParseAction::Error {last_input, .. } => {
      let vec = slots.to_vec();
      ParseResult::Error(last_input, vec)
    }
    _ => unreachable!("This function should only be called when the parse action is  [Error, Accept, or EndOfInput]"),
  }
}

#[no_mangle]
pub extern "C" fn sherpa_free_stack(ptr: *mut Goto, byte_size: usize) {
  #[cfg(debug_assertions)]
  {
    eprintln!("Freeing {} bytes for {} slots at address {:p}", byte_size, byte_size >> 4, ptr);
  }
  // Each goto slot is 16bytes, so we shift left num_of_slots by 4 to get the bytes size of
  // the stack.
  let layout = Layout::from_size_align(byte_size, 16).unwrap();

  unsafe { dealloc(ptr as *mut u8, layout) }
}

#[no_mangle]
pub extern "C" fn sherpa_get_token_class_from_codepoint(codepoint: u32) -> u32 {
  get_token_class_from_codepoint(codepoint)
}

#[derive(Debug)]
#[repr(C, u64)]
pub enum ParseResult<Node> {
  Complete((Node, TokenRange, TokenRange)),
  Error(TokenRange, Vec<(Node, TokenRange, TokenRange)>),
  NeedMoreInput(Vec<(Node, TokenRange, TokenRange)>),
}
