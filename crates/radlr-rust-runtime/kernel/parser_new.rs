#![allow(unused)]
use crate::{
  types::{
    bytecode::{ByteCodeIterator, Instruction, MatchInputType, Opcode, NORMAL_STATE_FLAG, STATE_HEADER},
    *,
  },
  utf8::{get_token_class_from_codepoint, get_utf8_byte_length_from_code_point},
};
use std::{collections::HashMap, rc::Rc};

struct OpResult<'a> {
  action:    ParseAction,
  next:      Option<Instruction<'a>>,
  is_goto:   bool,
  can_debug: bool,
}

/// Yields parser Actions from parsing an input using the
/// current active grammar bytecode.
fn dispatch<'a, 'debug>(
  base_state: ParserState,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  bc: &'a [u8],
  debug: &mut Option<&'debug mut DebugFnNew>,
  is_scanner: bool,
) -> (ParseAction, Option<Instruction<'a>>, usize) {
  use ParseAction::*;

  let mut block_base: Instruction = (bc, base_state.address).into();
  let mut i: Instruction = block_base.clone();

  loop {
    use Opcode::*;

    let opcode = i.get_opcode();
    #[cfg(any(debug_assertions, feature = "wasm-lab"))]
    {
      if !matches!(opcode, VectorBranch | HashBranch) {
        emit_instruction_debug(debug, i, input, ParserStackTrackers::from(&*ctx), is_scanner);
      }
    }

    i = match match opcode {
      ByteSequence => byte_sequence(i, ctx, input),
      ShiftToken => shift_token(i, ctx, base_state),
      ShiftTokenScanless => shift_token_scanless(i, ctx, base_state),
      ShiftChar => scan_shift(i, ctx),
      SkipToken => skip_token(block_base, ctx),
      SkipTokenScanless => skip_token_scanless(block_base, ctx),
      PeekSkipToken => peek_skip_token(block_base, ctx),
      PeekSkipTokenScanless => peek_skip_token_scanless(block_base, ctx),
      PeekToken => peek_token(i, ctx),
      PeekTokenScanless => peek_token_scanless(i, ctx),
      PeekReset => peek_reset(i, ctx),
      Reduce => reduce(i, ctx),
      Goto => goto(i),
      PushGoto => push_state(i, ctx),
      PushExceptionHandler => push_exception_handler(i, ctx),
      PopGoto => pop_goto(i, ctx),
      AssignToken => assign_token(i, ctx),
      VectorBranch => vector_branch(i, ctx, input, debug, is_scanner),
      HashBranch => hash_branch(i, ctx, input, debug, is_scanner),
      ReadCodepoint => read_codepoint(i, ctx, input, debug, is_scanner),
      Fail => OpResult {
        action:    FailState,
        next:      Option::None,
        is_goto:   false,
        can_debug: true,
      },
      Pass => OpResult {
        action:    CompleteState,
        next:      Option::None,
        is_goto:   false,
        can_debug: true,
      },
      Fork => fork(i),
      Accept => {
        ctx.is_finished = true;
        OpResult {
          action:    ParseAction::Accept {
            nonterminal_id:    ctx.nonterm,
            final_offset:      ctx.sym_ptr,
            token_line_count:  ctx.end_line_num,
            token_line_offset: ctx.end_line_off,
          },
          next:      Option::None,
          is_goto:   false,
          can_debug: true,
        }
      }
      NoOp => OpResult {
        action:    None,
        next:      i.next(),
        is_goto:   false,
        can_debug: true,
      },
    } {
      OpResult { action: None, next: Option::None, .. } => {
        todo!("Expected next instruction!")
      }

      OpResult { action: None, next: Some(next_instruction), is_goto, .. } => {
        if is_goto {
          block_base = next_instruction;
        }

        next_instruction
      }
      OpResult { action, next, .. } => {
        break (action, next, block_base.address());
      }
    }
  }
}

fn byte_sequence<'a>(i: Instruction<'a>, ctx: &mut ParserContext, input: &impl ParserInput) -> OpResult<'a> {
  let mut iter = i.iter();
  let length = iter.next_u16_le().unwrap() as usize;
  let default_offset = iter.next_u32_le().unwrap() as usize;
  let offset = ctx.input_ptr;
  let mut line_incr = 0;
  let mut line_offset = ctx.end_line_off;
  for b in 0..length {
    let byte = iter.next_u8();
    if byte != Some(input.byte(offset + b)) {
      if default_offset > 0 {
        return OpResult {
          action:    ParseAction::None,
          next:      Some((i.bytecode(), i.address() + default_offset).into()),
          is_goto:   false,
          can_debug: true,
        };
      } else {
        return OpResult {
          action:    ParseAction::FailState,
          next:      None,
          is_goto:   false,
          can_debug: true,
        };
      }
    }
    if byte == Some(10) {
      line_incr += 1;
      line_offset = (offset + b) as u32;
    }
  }

  ctx.end_line_off = line_offset;
  ctx.end_line_num += line_incr;

  ctx.byte_len = length as u32;

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::TokenShift] operation
fn shift_token<'a>(i: Instruction<'a>, ctx: &mut ParserContext, emitting_state: ParserState) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::ShiftToken;

  debug_assert!(
    ctx.start_line_off as usize <= ctx.sym_ptr,
    "
  The `start_line_offset` should not be advanced further than the `head_ptr` at this point
  head_ptr: {}  start_line_off:{}",
    ctx.sym_ptr,
    ctx.start_line_off
  );
  let is_recovery = ctx.recovery_tok_id > 0;

  let action = ParseAction::Shift {
    byte_offset: ctx.sym_ptr as u32,
    byte_length: ctx.tok_byte_len as u32,
    token_line_count: ctx.start_line_num,
    token_line_offset: ctx.start_line_off,
    token_id: if is_recovery { ctx.recovery_tok_id } else { ctx.tok_id },
    emitting_state,
    next_instruction_address: i.next().unwrap().address(),
    is_recovery,
  };

  ctx.start_line_num = ctx.chkp_line_num;
  ctx.start_line_off = ctx.chkp_line_off;

  ctx.end_line_num = ctx.chkp_line_num;
  ctx.end_line_off = ctx.chkp_line_off;

  let new_offset = ctx.sym_ptr + ctx.tok_byte_len as usize;

  ctx.anchor_ptr = new_offset;
  ctx.sym_ptr = new_offset;
  ctx.input_ptr = new_offset;
  ctx.tok_id = 0;
  ctx.recovery_tok_id = 0;
  ctx.tok_byte_len = 0;

  OpResult { action, next: i.next(), is_goto: false, can_debug: true }
}

/// Performs the [Opcode::ShiftTokenScanless] operation
fn shift_token_scanless<'a>(i: Instruction<'a>, ctx: &mut ParserContext, emitting_state: ParserState) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::ShiftTokenScanless;
  ctx.tok_byte_len = ctx.byte_len;
  shift_token(i, ctx, emitting_state)
}

/// Performs the [Opcode::ShiftChar] operation
fn scan_shift<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::ShiftChar;

  ctx.input_ptr = ctx.input_ptr + ctx.byte_len as usize;
  ctx.byte_len = 0;

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::PeekToken] operation
fn peek_token<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PeekToken;

  let offset = ctx.sym_ptr + ctx.tok_byte_len as usize;

  ctx.sym_ptr = offset;
  ctx.input_ptr = offset;
  ctx.tok_id = 0;
  ctx.recovery_tok_id = 0;
  ctx.tok_byte_len = 0;

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::PeekTokenScanless] operation
fn peek_token_scanless<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PeekTokenScanless;
  ctx.tok_byte_len = ctx.byte_len;
  peek_token(i, ctx)
}

fn __skip_token_core__<'a>(base_instruction: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  let original_offset = ctx.sym_ptr;
  let offset = ctx.sym_ptr + ctx.tok_byte_len as usize;
  let tok_len = ctx.tok_byte_len;
  let token_id = ctx.tok_id;
  ctx.input_ptr = offset;
  ctx.sym_ptr = offset;
  ctx.tok_id = 0;
  ctx.recovery_tok_id = 0;

  OpResult {
    action:    ParseAction::Skip {
      byte_offset:       original_offset as u32,
      byte_length:       tok_len as u32,
      token_line_count:  ctx.start_line_num,
      token_line_offset: ctx.start_line_off,
      token_id:          token_id as u32,
    },
    next:      Some(base_instruction),
    is_goto:   false,
    can_debug: false,
  }
}

/// Performs the [Opcode::SkipToken] operation
fn skip_token<'a>(base_instruction: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::SkipToken;
  let result = __skip_token_core__(base_instruction, ctx);

  ctx.end_line_num = ctx.chkp_line_num;
  ctx.end_line_off = ctx.chkp_line_off;

  ctx.start_line_num = ctx.chkp_line_num;
  ctx.start_line_off = ctx.chkp_line_off;

  result
}

/// Performs the [Opcode::SkipTokenScanless] operation
fn skip_token_scanless<'a>(base_instruction: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::SkipTokenScanless;
  ctx.tok_byte_len = ctx.byte_len;
  skip_token(base_instruction, ctx)
}

/// Performs the [Opcode::PeekSkipToken] operation
fn peek_skip_token<'a>(base_instruction: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PeekSkipToken;
  __skip_token_core__(base_instruction, ctx);

  OpResult {
    action:    ParseAction::None,
    next:      Some(base_instruction),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::PeekSkipTokenScanless] operation
fn peek_skip_token_scanless<'a>(base_instruction: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PeekSkipTokenScanless;
  ctx.tok_byte_len = ctx.byte_len;
  peek_skip_token(base_instruction, ctx)
}

/// Performs the [Opcode::Reduce] operation
fn reduce<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::Reduce;
  let mut iter = i.iter();
  let nonterminal_id = iter.next_u32_le().unwrap();
  let rule_id = iter.next_u32_le().unwrap();
  let symbol_count = iter.next_u16_le().unwrap() as u32;

  ctx.nonterm = nonterminal_id;

  OpResult {
    action:    ParseAction::Reduce { nonterminal_id, rule_id, symbol_count },
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::Fork] operation
fn fork<'a>(i: Instruction<'a>) -> OpResult<'a> {
  let mut iter = i.iter();
  let length = iter.next_u16_le().unwrap();
  let mut goto_states = vec![];

  for _ in 0..length {
    let address = iter.next_u32_le().unwrap() as usize;
    goto_states.push(ParserState::state_entry(address))
  }

  OpResult {
    action:    ParseAction::Fork(goto_states),
    next:      None,
    is_goto:   false,
    can_debug: false,
  }
}

/// Performs the [Opcode::PushGoto] operation
fn push_state<'a, 'debug>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PushGoto;
  let mut iter = i.iter();
  iter.next_u8().unwrap();
  let address = iter.next_u32_le().unwrap() as usize;

  ctx.push_state(ParserState::state_entry(address));

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::PushExceptionHandler] operation
fn push_exception_handler<'a, 'debug>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PushExceptionHandler;
  let mut iter = i.iter();
  iter.next_u8().unwrap();
  let address = iter.next_u32_le().unwrap() as usize;

  ctx.push_state(ParserState::state_entry(address));

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::Goto] operation
fn goto<'a, 'debug>(i: Instruction<'a>) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::Goto;
  let mut iter = i.iter();
  let _state_mode_ = iter.next_u8().unwrap();
  let address = iter.next_u32_le().unwrap();

  OpResult {
    action:    ParseAction::None,
    next:      Some((i.bytecode(), address as usize).into()),
    is_goto:   true,
    can_debug: true,
  }
}

/// Performs the [Opcode::PopGoto] operation
fn pop_goto<'a, 'debug>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PopGoto;

  ctx.pop_state();

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::AssignToken] operation
fn assign_token<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::AssignToken;
  let mut iter = i.iter();
  ctx.tok_id = iter.next_u32_le().unwrap();
  ctx.tok_byte_len = (ctx.input_ptr - ctx.sym_ptr) as u32;

  ctx.chkp_line_num = ctx.end_line_num;
  ctx.chkp_line_off = ctx.end_line_off;

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::PeekReset] operation
fn peek_reset<'a>(i: Instruction<'a>, ctx: &mut ParserContext) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::PeekReset;
  let offset = ctx.anchor_ptr;
  ctx.sym_ptr = offset;
  ctx.input_ptr = offset;
  ctx.tok_id = 0;
  ctx.recovery_tok_id = 0;
  ctx.tok_byte_len = 0;
  ctx.byte_len = 0;

  ctx.end_line_off = ctx.start_line_off;
  ctx.end_line_num = ctx.start_line_num;

  ctx.chkp_line_off = ctx.start_line_off;
  ctx.chkp_line_num = ctx.start_line_num;

  OpResult {
    action:    ParseAction::None,
    next:      i.next(),
    is_goto:   false,
    can_debug: true,
  }
}

/// Performs the [Opcode::ReadCodepoint] operation
fn read_codepoint<'a, 'debug>(
  i: Instruction<'a>,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  debug: &mut Option<&mut DebugFnNew>,
  is_scanner: bool,
) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::ReadCodepoint;
  emit_instruction_debug(debug, i, input, ParserStackTrackers::from(&*ctx), is_scanner);
  let (cp, is_nl) = get_input_value(MatchInputType::Codepoint, i, ctx, input, debug, is_scanner);
  if cp == 0 {
    OpResult {
      action:    ParseAction::FailState,
      next:      None,
      is_goto:   false,
      can_debug: false,
    }
  } else {
    if is_nl {
      ctx.chkp_line_num += 1;
      ctx.chkp_line_off = ctx.input_ptr as u32;
    }
    OpResult {
      action:    ParseAction::None,
      next:      i.next(),
      is_goto:   false,
      can_debug: false,
    }
  }
}

/// Performs the [Opcode::HashBranch] operation
fn hash_branch<'a, 'debug>(
  i: Instruction<'a>,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  debug: &mut Option<&mut DebugFnNew>,
  is_scanner: bool,
) -> OpResult<'a> {
  const __HINT__: Opcode = Opcode::HashBranch;

  // Decode data
  let TableHeaderData {
    input_type,
    table_meta,
    scan_block_instruction,
    default_block,
    table_start,
    ..
  } = i.into();

  let (input_value, is_nl) = get_input_value(input_type, scan_block_instruction, ctx, input, debug, is_scanner);
  #[cfg(any(debug_assertions, feature = "wasm-lab"))]
  {
    let tok_id = ctx.tok_id;
    ctx.tok_id = input_value;
    emit_instruction_debug(debug, i, input, ParserStackTrackers::from(&*ctx), is_scanner);
    ctx.tok_id = tok_id;
  }

  get_hash_result(input_value, table_meta, i, table_start, is_nl, ctx, default_block)
}

#[inline]
fn get_hash_result<'a>(
  input_value: u32,
  table_meta: u32,
  i: Instruction<'a>,
  table_start: usize,
  is_nl: bool,
  ctx: &mut ParserContext,
  default_block: Instruction<'a>,
) -> OpResult<'a> {
  let hash_mask = (1 << table_meta) - 1;
  loop {
    let mut hash_index = (input_value & hash_mask) as usize;
    loop {
      let mut iter: ByteCodeIterator = (i.bytecode(), table_start + hash_index * 4).into();
      let cell = iter.next_u32_le().unwrap();
      let value = cell & 0x7FF;
      let off = (cell >> 11) & 0x7FF;
      let next = ((cell >> 22) & 0x3FF) as i32 - 512;

      if value == input_value {
        if is_nl {
          ctx.end_line_num += 1;
          ctx.end_line_off = ctx.input_ptr as u32;
        }
        return OpResult {
          action:    ParseAction::None,
          next:      Some((i.bytecode(), i.address() + off as usize).into()),
          is_goto:   false,
          can_debug: false,
        };
      } else if next != 0 {
        hash_index = ((hash_index as i32) + next) as usize;
      } else {
        return OpResult {
          action:    ParseAction::None,
          next:      Some(default_block),
          is_goto:   false,
          can_debug: false,
        };
      }
    }
  }
}

pub fn get_success_branches<'a>(i: Instruction<'a>) -> Vec<SuccessorState> {
  // if yield success branch actions.

  let opcode = i.get_opcode();
  let mut addresses = Vec::new();

  match opcode {
    Opcode::HashBranch => {
      // Decode data
      let TableHeaderData { table_start, table_length, input_type, .. } = i.into();

      let mut iter: ByteCodeIterator = (i.bytecode(), table_start).into();

      for _ in 0..table_length {
        let cell = iter.next_u32_le().unwrap();
        let off = (cell >> 11) & 0x7FF;
        let value = cell & 0x7FF;
        let address = off as u32 + i.address() as u32;
        let next: Instruction<'_> = (i.bytecode(), address as usize).into();
        addresses.push(SuccessorState {
          edge_mode:  input_type,
          edge_value: value,
          skipped:    next.get_opcode() == Opcode::SkipToken,
          state_info: StateInfo {
            stack_address:  0,
            state_type:     StateType::Normal,
            is_state_entry: false,
            state_id:       address,
          },
        });
      }

      addresses
    }
    Opcode::VectorBranch => {
      let TableHeaderData { table_start, table_length, input_type, table_meta, .. } = i.into();

      let mut iter: ByteCodeIterator = (i.bytecode(), table_start).into();

      for val_offset in 0..table_length {
        let address_offset = iter.next_u32_le().unwrap();

        let address = address_offset as u32 + i.address() as u32;
        let next: Instruction<'_> = (i.bytecode(), address as usize).into();

        addresses.push(SuccessorState {
          edge_mode:  input_type,
          edge_value: table_meta + val_offset as u32,
          skipped:    next.get_opcode() == Opcode::SkipToken,
          state_info: StateInfo {
            stack_address:  0,
            state_type:     StateType::Normal,
            is_state_entry: false,
            state_id:       address,
          },
        });
      }

      addresses
    }
    _ => addresses,
  }
}

pub fn get_token_id<'a>(i: Instruction<'a>, ctx: &mut ParserContext, input: &impl ParserInput) -> (u32, bool) {
  let opcode = i.get_opcode();
  // if yield success branch actions.
  match opcode {
    Opcode::HashBranch | Opcode::VectorBranch => {
      let TableHeaderData {
        input_type,
        table_meta,
        scan_block_instruction,
        default_block,
        table_start,
        table_length,
        ..
      } = i.into();

      if input_type == MatchInputType::Token {
        let (input_value, is_nl) = get_input_value(input_type, scan_block_instruction, ctx, input, &mut None, false);

        let branch = match opcode {
          Opcode::HashBranch => get_hash_result(input_value, table_meta, i, table_start, is_nl, ctx, default_block),
          Opcode::VectorBranch => {
            vector_input_match(input_value, table_meta, table_length, is_nl, ctx, i, table_start, default_block)
          }
          _ => todo!(),
        };

        if branch.next.unwrap().get_opcode() == Opcode::SkipToken {
          (input_value, true)
        } else {
          (input_value, false)
        }
      } else {
        (0, false)
      }
    }
    _ => (0, false),
  }
}

pub fn is_token_state<'a>(i: Instruction<'a>) -> bool {
  let opcode = i.get_opcode();
  // if yield success branch actions.
  match opcode {
    Opcode::HashBranch | Opcode::VectorBranch => {
      let TableHeaderData { input_type, .. } = i.into();
      if input_type == MatchInputType::Token {
        true
      } else {
        false
      }
    }
    _ => false,
  }
}

fn vector_branch<'a, 'debug>(
  i: Instruction<'a>,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  debug: &mut Option<&'debug mut DebugFnNew>,
  is_scanner: bool,
) -> OpResult<'a> {
  // Decode data

  let TableHeaderData {
    input_type,
    table_meta,
    scan_block_instruction,
    default_block,
    table_start,
    table_length,
    ..
  } = i.into();

  let (input_value, is_nl) = get_input_value(input_type, scan_block_instruction, ctx, input, debug, is_scanner);

  vector_input_match(input_value, table_meta, table_length, is_nl, ctx, i, table_start, default_block)
}

fn vector_input_match<'a>(
  input_value: u32,
  table_meta: u32,
  table_length: u32,
  is_nl: bool,
  ctx: &mut ParserContext,
  i: Instruction<'a>,
  table_start: usize,
  default_block: Instruction<'a>,
) -> OpResult<'a> {
  loop {
    let value_index = (input_value as i32 - table_meta as i32) as usize;
    if value_index < table_length as usize {
      if is_nl {
        ctx.end_line_num += 1;
        ctx.end_line_off = ctx.input_ptr as u32;
      }
      let mut iter: ByteCodeIterator = (i.bytecode(), table_start + value_index * 4).into();
      let address_offset = iter.next_u32_le().unwrap();
      return OpResult {
        action:    ParseAction::None,
        next:      Some((i.bytecode(), i.address() + address_offset as usize).into()),
        is_goto:   false,
        can_debug: false,
      };
    } else {
      return OpResult {
        action:    ParseAction::None,
        next:      Some(default_block),
        is_goto:   false,
        can_debug: false,
      };
    }
  }
}

fn get_input_value<'a, 'debug>(
  input_type: MatchInputType,
  scan_index: Instruction<'a>,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  debug: &mut Option<&'debug mut DebugFnNew>,
  is_scanner: bool,
) -> (u32, bool) {
  let mut is_nl = false;
  let val = match input_type {
    MatchInputType::NonTerminal => ctx.nonterm as u32,
    MatchInputType::EndOfFile => (ctx.input_ptr >= input.len()) as u32,
    MatchInputType::Token => {
      if ctx.recovery_tok_id > 0 {
        ctx.tok_id = ctx.recovery_tok_id;
        ctx.tok_byte_len = 0;
        ctx.byte_len = 0;
      } else {
        debug_assert!(!is_scanner);
        token_scan(scan_index, ctx, input, debug);
      }
      ctx.tok_id as u32
    }
    MatchInputType::CSTNode => {
      /* Disabled for the time being. */
      u32::MAX
    }
    MatchInputType::Byte => {
      let byte = input.byte(ctx.input_ptr);

      if byte == 10 {
        is_nl = true;
      }

      if byte > 0 {
        ctx.byte_len = 1;
      } else {
        ctx.byte_len = 0;
      }
      byte as u32
    }
    MatchInputType::ByteScanless => {
      let byte = input.byte(ctx.input_ptr);

      if byte == 10 {
        is_nl = true;
      }

      if byte > 0 {
        ctx.tok_byte_len = 1;
      } else {
        ctx.tok_byte_len = 0;
      }
      byte as u32
    }
    input_type => {
      let cp: u32 = input.codepoint(ctx.input_ptr);

      let len = get_utf8_byte_length_from_code_point(cp);

      if cp == 10 {
        is_nl = true;
      }

      match input_type {
        MatchInputType::ClassScanless => {
          ctx.tok_byte_len = len;
          if cp > 0 {
            get_token_class_from_codepoint(cp)
          } else {
            0
          }
        }
        MatchInputType::Class => {
          ctx.byte_len = len;
          if cp > 0 {
            get_token_class_from_codepoint(cp)
          } else {
            0
          }
        }
        MatchInputType::CodepointScanless => {
          ctx.tok_byte_len = len;
          cp
        }
        MatchInputType::Codepoint => {
          ctx.byte_len = len as u32;
          cp
        }
        i_type => todo!("{}", i_type),
      }
    }
  };

  (val, is_nl)
}

fn token_scan<'a, 'debug>(
  scan_index: Instruction<'a>,
  ctx: &mut ParserContext,
  input: &impl ParserInput,
  debug: &mut Option<&'debug mut DebugFnNew>,
) {
  ctx.tok_id = 0;
  ctx.input_ptr = ctx.sym_ptr;

  // Initialize Scanner

  // We are done with the state reference, so we
  // invalidate variable by moving the reference to
  // an unused name to prevent confusion with the
  // `scan_state` variable.
  let mut stack = vec![0, 0, NORMAL_STATE_FLAG | STATE_HEADER, scan_index.address() as u32];
  let bc = scan_index.bytecode();

  match {
    let mut address = stack.pop().unwrap() as usize;
    let mut state = stack.pop().unwrap();

    loop {
      if state < 1 {
        break Some(());
      } else {
        #[cfg(any(debug_assertions, feature = "wasm-lab"))]
        emit_state_debug(debug, bc, address, ParserStackTrackers::from(&*ctx), input, true);

        match dispatch(ParserState { address: address as usize, info: Default::default() }, ctx, input, bc, debug, true) {
          (ParseAction::CompleteState, ..) => {}
          (ParseAction::FailState, ..) => {
            break Some(());
          }
          (_, next_block, ..) => {
            if let Some(next_block) = next_block {
              stack.push(NORMAL_STATE_FLAG);
              stack.push(next_block.address() as u32);
            }
          }
        };
      }
      address = stack.pop().unwrap() as usize;
      state = stack.pop().unwrap();
    }
  } {
    Some(()) => {
      ctx.input_ptr = ctx.sym_ptr;
    }
    _ => panic!("Unusable State"),
  }
}

pub struct ByteCodeParserNew {
  bc:                  Rc<dyn AsRef<[u8]>>,
  non_terminal_lookup: HashMap<u32, u32>,
  debugger:            Option<Box<DebugFnNew>>,
}

impl ByteCodeParserNew {
  pub fn new(bc: Rc<dyn AsRef<[u8]>>, non_terminal_lookup: HashMap<u32, u32>) -> Self {
    debug_assert!(bc.as_ref().as_ref().len() > 0, "Bytecode is empty!");
    ByteCodeParserNew { bc, non_terminal_lookup, debugger: None }
  }
}

impl ParserInitializer for ByteCodeParserNew {
  fn set_debugger(&mut self, debugger: Option<Box<DebugFnNew>>) {
    self.debugger = debugger;
  }

  fn get_debugger(&mut self) -> &mut Option<Box<DebugFnNew>> {
    &mut self.debugger
  }

  fn init_range(&mut self, nonterminal_goal_id: u32, start: usize, end: usize) -> Result<ParserContext, ParserError> {
    if let Some(address) = self.non_terminal_lookup.get(&nonterminal_goal_id) {
      if *address == 0 {
        Err(ParserError::InvalidNonTerminal)
      } else {
        let mut root = ParserContext::default();
        root.anchor_ptr = start;
        root.sym_ptr = start;
        root.end_ptr = end;
        root.stack = vec![ParserState::default(), ParserState::state_entry(*address as usize)];
        //self.stacks = vec![root];
        Ok(root)
      }
    } else {
      Err(ParserError::InvalidNonTerminal)
    }
  }

  fn init(&mut self, entry: EntryPoint) -> Result<ParserContext, ParserError> {
    if let Some(address) = self.non_terminal_lookup.get(&entry.nonterm_id) {
      if *address == 0 {
        Err(ParserError::InvalidNonTerminal)
      } else {
        let mut root = ParserContext::default();

        root.stack = vec![ParserState::default(), ParserState::state_entry(*address as usize)];
        Ok(root)
      }
    } else {
      Err(ParserError::InvalidNonTerminal)
    }
  }
}

impl<T: ParserInput> ParserIterator<T> for ByteCodeParserNew {
  fn get_success_states<'ctx>(&mut self, address: StateInfo) -> Vec<SuccessorState> {
    get_success_branches(Instruction::from((self.bc.as_ref().as_ref(), address.state_id as usize)))
  }

  fn get_token_id<'ctx>(&mut self, address: StateInfo, input: &mut T, ctx: &mut ParserContext) -> (u32, bool) {
    get_token_id(Instruction::from((self.bc.as_ref().as_ref(), address.state_id as usize)), ctx, input)
  }

  fn is_token_branch_state<'ctx>(&mut self, address: StateInfo) -> bool {
    is_token_state(Instruction::from((self.bc.as_ref().as_ref(), address.state_id as usize)))
  }

  fn next(&mut self, input: &mut T, ctx: &mut ParserContext) -> Option<ParseAction> {
    let Self { bc, debugger, .. } = self;

    if ctx.is_finished {
      return None;
    }

    let mut debugger = debugger.as_mut().map(|i| i.as_mut());

    let bc = bc.as_ref().as_ref();

    let mut state = ctx.pop_state();

    loop {
      if state.address < 1 {
        //Accept never encountered.
        ctx.is_finished = true;
        break Some(ParseAction::Error {
          last_nonterminal:  ctx.nonterm,
          last_state:        state,
          byte_offset:       ctx.sym_ptr as u32,
          byte_length:       ctx.tok_byte_len,
          token_line_count:  ctx.chkp_line_num,
          token_line_offset: ctx.chkp_line_off,
        });
      } else {
        #[cfg(any(debug_assertions, feature = "wasm-lab"))]
        if state.info.is_state_entry {
          emit_state_debug(&mut debugger, bc, state.address as usize, ParserStackTrackers::from(&*ctx), input, false);
        }
        match dispatch(state, ctx, input, bc, &mut debugger, false) {
          (ParseAction::CompleteState, ..) => {
            state = ctx.pop_state();
          }
          (ParseAction::FailState, _, fail_address) => {
            ctx.is_finished = true;
            break Some(ParseAction::Error {
              last_nonterminal:  ctx.nonterm,
              last_state:        ParserState::yield_entry(fail_address),
              byte_offset:       ctx.sym_ptr as u32,
              byte_length:       ctx.tok_byte_len,
              token_line_count:  ctx.chkp_line_num,
              token_line_offset: ctx.chkp_line_off,
            });
          }
          (action, next_state, ..) => {
            if let Some(next_state) = next_state {
              ctx.push_state(ParserState::yield_entry(next_state.address()));
            }
            break Some(action);
          }
        }
      }
    }
  }
}
