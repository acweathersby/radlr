use std::collections::VecDeque;
use std::fmt::Debug;

use regex::internal::Input;

use super::*;
pub struct ParseContext<T: ImmutCharacterReader + MutCharacterReader>
{
  pub(crate) peek_token: ParseToken,
  pub(crate) anchor_token: ParseToken,
  pub(crate) assert_token: ParseToken,
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
  pub local_state_stack: [u32; 32],
}

impl<T: ImmutCharacterReader + MutCharacterReader> ParseContext<T>
{
  pub fn new(reader: &mut T) -> Self
  {
    let mut ctx = Self {
      peek_token: ParseToken::default(),
      anchor_token: ParseToken::default(),
      assert_token: ParseToken::default(),
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
      local_state_stack: [0; 32],
    };
    ctx
  }

  pub fn bytecode_context() -> Self
  {
    const stack_32_bit_size: usize = 32;
    let mut ctx = Self {
      peek_token: ParseToken::default(),
      anchor_token: ParseToken::default(),
      assert_token: ParseToken::default(),
      action_ptr: 0 as *mut ParseAction,
      stack_base: [].as_mut_ptr(),
      stack_top: 0,
      stack_size: (stack_32_bit_size as u32) >> 1,
      state: 0,
      production: 0,
      input_block: 0,
      input_block_length: 0,
      input_block_offset: 0,
      reader: 0 as *mut T,
      local_state_stack: [0; stack_32_bit_size],
      in_peek_mode: 0,
    };

    ctx
  }

  /// The following methods are used exclusively by the
  /// the rust functions in [hctk::runtime::parser_functions.rs]

  #[inline]
  pub(crate) fn in_fail_mode(&self) -> bool
  {
    self.input_block_offset > 0
  }

  #[inline]
  pub(crate) fn set_fail_mode_to(&mut self, is_in_fail_mode: bool)
  {
    self.state = if is_in_fail_mode {
      FAIL_STATE_FLAG
    } else {
      NORMAL_STATE_FLAG
    }
  }

  #[inline]
  pub(crate) fn in_peek_mode(&self) -> bool
  {
    self.in_peek_mode > 0
  }

  #[inline]
  pub(crate) fn set_peek_mode_to(&mut self, is_in_peek_mode: bool)
  {
    self.in_peek_mode = is_in_peek_mode as u32;
  }

  #[inline]
  pub(crate) fn is_interrupted(&self) -> bool
  {
    self.action_ptr as usize > 0
  }

  #[inline]
  pub(crate) fn set_interrupted_to(&mut self, is_interrupted: bool)
  {
    self.action_ptr = is_interrupted as usize as *mut ParseAction
  }

  /// Used by the bytecode interpreter
  #[inline]
  pub(crate) fn is_scanner(&self) -> bool
  {
    self.input_block > 0
  }

  /// Used by the bytecode interpreter
  #[inline]
  pub(crate) fn make_scanner(&mut self)
  {
    self.input_block = 1;
  }

  #[inline]
  pub(crate) fn get_active_state(&mut self) -> u32
  {
    self.state as u32
  }

  #[inline]
  pub(crate) fn set_active_state_to(&mut self, state: u32)
  {
    self.state = state;
  }

  #[inline]
  pub(crate) fn get_production(&mut self) -> u32
  {
    self.production
  }

  #[inline]
  pub(crate) fn set_production_to(&mut self, production: u32)
  {
    self.production = production;
  }

  #[inline]
  pub(crate) fn pop_state(&mut self) -> u32
  {
    if self.stack_top > 0 {
      self.stack_top -= 1;
      self.local_state_stack[self.stack_top] as u32
    } else {
      0
    }
  }

  #[inline]
  pub(crate) fn push_state(&mut self, state: u32)
  {
    if (self.stack_top >= self.stack_size as usize) {
      panic!("Out of parse stack space!");
    }

    self.local_state_stack[self.stack_top] = state;

    self.stack_top += 1;
  }

  #[inline]
  pub fn init_normal_state(&mut self, entry_point: u32)
  {
    self.stack_top = 0;

    self.push_state((NORMAL_STATE_FLAG | entry_point));
  }
}

#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct Goto
{
  pub goto_fn: *const usize,
  pub state:   u32,
  pub meta:    u32,
}

impl Default for Goto
{
  fn default() -> Self
  {
    Self {
      goto_fn: 0 as *const usize,
      state:   0,
      meta:    0,
    }
  }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct InputBlock
{
  /// The pointer to the beginning of the block window slice.
  pub block:  *const u8,
  /// The offset of this block, calculated as the relative distance
  /// from the start of the input string to the InputBlock's pointer
  /// position.
  pub offset: u32,
  /// The number of bytes the block window can view
  pub length: u32,
}

impl Default for InputBlock
{
  fn default() -> Self
  {
    Self {
      block:  0 as *const u8,
      offset: 0,
      length: 0,
    }
  }
}

#[derive(Clone)]
#[repr(C)]
pub struct LLVMParseContext<
  T: LLVMCharacterReader + ByteCharacterReader + ImmutCharacterReader,
> {
  pub local_goto_stack: [Goto; 8],
  pub anchor_token: ParseToken,
  pub assert_token: ParseToken,
  pub peek_token: ParseToken,
  pub input_block: InputBlock,
  pub stack_base: *const usize,
  pub stack_top: *const usize,
  pub get_byte_block_at_cursor: fn(&mut T, &mut InputBlock),
  pub reader: *mut T,
  pub stack_size: u32,
  pub production: u32,
  pub state: u32,
  pub in_peek_mode: u32,
}

impl<T: LLVMCharacterReader + ByteCharacterReader + ImmutCharacterReader> Debug
  for LLVMParseContext<T>
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    let mut dbgstr = f.debug_struct("LLVMParseContext");
    dbgstr.field("local_goto_stack", &self.local_goto_stack);
    dbgstr.field("anchor_token", &self.anchor_token);
    dbgstr.field("assert_token", &self.assert_token);
    dbgstr.field("peek_token", &self.peek_token);
    dbgstr.field("input_block", &self.input_block);
    dbgstr.field("stack_base", &self.stack_base);
    dbgstr.field("stack_top", &self.stack_top);
    dbgstr.field("get_byte_block_at_cursor", &"MASKED");
    dbgstr.field("reader", &self.reader);
    dbgstr.field("stack_size", &self.stack_size);
    dbgstr.field("production", &self.production);
    dbgstr.field("state", &self.state);
    dbgstr.field("in_peek_mode", &self.in_peek_mode);
    dbgstr.finish()
  }
}

impl<T: LLVMCharacterReader + ByteCharacterReader + ImmutCharacterReader>
  LLVMParseContext<T>
{
  pub fn new(reader: &mut T) -> Self
  {
    let mut ctx = Self {
      peek_token: ParseToken::default(),
      anchor_token: ParseToken::default(),
      assert_token: ParseToken::default(),
      stack_base: 0 as *const usize,
      stack_top: 0 as *const usize,
      state: 0,
      production: 0,
      input_block: InputBlock {
        block:  0 as *const u8,
        length: 0,
        offset: 0,
      },
      stack_size: 0,
      reader: reader,
      get_byte_block_at_cursor: T::get_byte_block_at_cursor,
      in_peek_mode: 0,
      local_goto_stack: [Goto::default(); 8],
    };
    ctx
  }
}

#[no_mangle]
pub extern "C" fn hctk_get_stack_pointer<'a>(stack: &mut Vec<usize>) -> *mut usize
{
  let ptr = stack.as_mut_ptr();

  ptr
}
#[no_mangle]
pub extern "C" fn hctk_get_stack_size(stack: &Vec<usize>) -> usize
{
  let size = stack.len() << 3;

  size
}

#[no_mangle]
pub extern "C" fn hctk_extend_stack(stack: &mut Vec<usize>) -> usize
{
  let old_size = stack.len();
  if let Err(err) = stack.try_reserve(stack.len() << 1) {
    println!("Error on parse stack extension {}", err);
    0
  } else {
    // pad out stack if there is more
    // then double the original size
    for _ in 0..(stack.capacity() - (old_size << 1)) {
      stack.push(0);
    }

    // move all element to back of stack.
    for i in 0..old_size {
      stack.push(stack[i]);
    }
    1
  }
}
