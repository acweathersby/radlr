use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use hctk::bytecode::BytecodeOutput;
use hctk::grammar::get_exported_productions;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::types::StructType;
use inkwell::values::CallableValue;
use inkwell::values::FunctionValue;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;

use crate::builder::table::BranchTableData;
use crate::options::BuildOptions;
use hctk::types::*;

pub const FAIL_STATE_FLAG_LLVM: u32 = 2;
pub const NORMAL_STATE_FLAG_LLVM: u32 = 1;

#[derive(Debug)]
pub struct LLVMTypes<'a>
{
  pub reader:      StructType<'a>,
  pub parse_ctx:   StructType<'a>,
  pub token:       StructType<'a>,
  pub goto:        StructType<'a>,
  pub goto_fn:     FunctionType<'a>,
  pub action:      StructType<'a>,
  pub input_block: StructType<'a>,
}
#[derive(Debug)]
pub struct CTXGEPIndices
{
  pub goto_base:       u32,
  pub goto_stack:      u32,
  pub goto_stack_len:  u32,
  pub goto_top:        u32,
  pub tok_anchor:      u32,
  pub tok_assert:      u32,
  pub tok_peek:        u32,
  pub input_block:     u32,
  pub reader:          u32,
  pub state:           u32,
  pub production:      u32,
  pub peek_mode:       u32,
  pub get_input_block: u32,
}
#[derive(Debug)]
pub struct PublicFunctions<'a>
{
  pub(crate) next: FunctionValue<'a>,
  pub(crate) init: FunctionValue<'a>,
  pub(crate) pop_state: FunctionValue<'a>,
  pub(crate) push_state: FunctionValue<'a>,
  pub(crate) emit_accept: FunctionValue<'a>,
  pub(crate) emit_error: FunctionValue<'a>,
  pub(crate) emit_eoi: FunctionValue<'a>,
  pub(crate) emit_eop: FunctionValue<'a>,
  pub(crate) emit_shift: FunctionValue<'a>,
  pub(crate) emit_reduce: FunctionValue<'a>,
  pub(crate) prime: FunctionValue<'a>,
  pub(crate) scan: FunctionValue<'a>,
  pub(crate) memcpy: FunctionValue<'a>,
  pub(crate) min: FunctionValue<'a>,
  pub(crate) max: FunctionValue<'a>,
  pub(crate) get_adjusted_input_block: FunctionValue<'a>,
  pub(crate) extend_stack_if_needed: FunctionValue<'a>,
  pub(crate) allocate_stack: FunctionValue<'a>,
  pub(crate) free_stack: FunctionValue<'a>,
}

#[derive(Debug)]
pub struct LLVMParserModule<'a>
{
  pub(crate) ctx:         &'a Context,
  pub(crate) module:      Module<'a>,
  pub(crate) builder:     Builder<'a>,
  pub(crate) types:       LLVMTypes<'a>,
  pub(crate) ctx_indices: CTXGEPIndices,
  pub(crate) fun:         PublicFunctions<'a>,
  pub(crate) exe_engine:  Option<ExecutionEngine<'a>>,
}

pub(crate) fn construct_context<'a>(
  module_name: &str,
  ctx: &'a Context,
) -> LLVMParserModule<'a>
{
  use inkwell::AddressSpace::*;
  let module = ctx.create_module(module_name);
  let builder = ctx.create_builder();

  let i8 = ctx.i8_type();
  let i64 = ctx.i64_type();
  let i32 = ctx.i32_type();
  let READER = ctx.opaque_struct_type("s.READER");
  let ACTION = ctx.opaque_struct_type("s.ACTION");
  let CTX = ctx.opaque_struct_type("s.CTX");
  let GOTO = ctx.opaque_struct_type("s.Goto");
  let TOKEN = ctx.opaque_struct_type("s.Token");
  let INPUT_BLOCK = ctx.opaque_struct_type("s.InputBlock");
  let GOTO_FN =
    i32.fn_type(&[CTX.ptr_type(Generic).into(), ACTION.ptr_type(Generic).into()], false);

  ACTION.set_body(&[i32.into(), i32.into()], false);

  GOTO.set_body(&[GOTO_FN.ptr_type(Generic).into(), i32.into(), i32.into()], false);

  TOKEN.set_body(&[i64.into(), i64.into(), i64.into(), i64.into()], false);

  INPUT_BLOCK.set_body(
    &[
      i8.ptr_type(Generic).into(),
      i32.into(),
      i32.into(),
      ctx.bool_type().into(),
    ],
    false,
  );
  let get_input_block_type = ctx
    .void_type()
    .fn_type(
      &[READER.ptr_type(Generic).into(), INPUT_BLOCK.ptr_type(Generic).into()],
      false,
    )
    .ptr_type(Generic);

  CTX.set_body(
    &[
      GOTO.array_type(8).into(),
      TOKEN.into(),
      TOKEN.into(),
      TOKEN.into(),
      INPUT_BLOCK.into(),
      GOTO.ptr_type(Generic).into(),
      GOTO.ptr_type(Generic).into(),
      get_input_block_type.into(),
      READER.ptr_type(Generic).into(),
      i32.into(),
      i32.into(),
      i32.into(),
      i32.into(),
    ],
    false,
  );

  let emit_function_type =
    i32.fn_type(&[CTX.ptr_type(Generic).into(), ACTION.ptr_type(Generic).into()], false);

  LLVMParserModule {
    builder,
    ctx,
    types: LLVMTypes {
      reader:      READER,
      action:      ACTION,
      token:       TOKEN,
      parse_ctx:   CTX,
      goto:        GOTO,
      goto_fn:     GOTO_FN,
      input_block: INPUT_BLOCK,
    },
    ctx_indices: CTXGEPIndices {
      goto_stack:      0,  // 0
      tok_anchor:      1,  // 1
      tok_assert:      2,  // 2
      tok_peek:        3,  // 3
      input_block:     4,  // 4
      goto_base:       5,  // 5
      goto_top:        6,  // 6
      get_input_block: 7,  // 7
      reader:          8,  // 8
      goto_stack_len:  9,  // 9
      production:      10, // 10
      state:           11, // 11
      peek_mode:       12, // 12
    },
    fun: PublicFunctions {
      allocate_stack: module.add_function(
        "hctk_allocate_stack",
        GOTO.ptr_type(Generic).fn_type(&[i32.into()], false),
        Some(Linkage::External),
      ),
      free_stack: module.add_function(
        "hctk_free_stack",
        ctx.void_type().fn_type(&[GOTO.ptr_type(Generic).into(), i32.into()], false),
        Some(Linkage::External),
      ),
      get_adjusted_input_block: module.add_function(
        "get_adjusted_input_block",
        INPUT_BLOCK.fn_type(
          &[
            CTX.ptr_type(Generic).into(),
            TOKEN.ptr_type(Generic).into(),
            i32.into(),
          ],
          false,
        ),
        None,
      ),
      scan: module.add_function(
        "scan",
        TOKEN.fn_type(
          &[
            CTX.ptr_type(Generic).into(),
            GOTO_FN.ptr_type(Generic).into(),
            TOKEN.ptr_type(Generic).into(),
          ],
          false,
        ),
        None,
      ),
      next: module.add_function(
        "next",
        ctx.void_type().fn_type(
          &[CTX.ptr_type(Generic).into(), ACTION.ptr_type(Generic).into()],
          false,
        ),
        None,
      ),
      init: module.add_function(
        "init",
        ctx.void_type().fn_type(&[CTX.ptr_type(Generic).into()], false),
        None,
      ),
      push_state: module.add_function(
        "push_state",
        ctx.void_type().fn_type(
          &[
            CTX.ptr_type(Generic).into(),
            i32.into(),
            GOTO_FN.ptr_type(Generic).into(),
          ],
          false,
        ),
        None,
      ),
      pop_state: module.add_function(
        "pop_state",
        GOTO.fn_type(&[CTX.ptr_type(Generic).into()], false),
        None,
      ),
      emit_reduce: module.add_function(
        "emit_reduce",
        i32.fn_type(
          &[
            CTX.ptr_type(Generic).into(),
            ACTION.ptr_type(Generic).into(),
            i32.into(),
            i32.into(),
            i32.into(),
          ],
          false,
        ),
        None,
      ),
      emit_eop: module.add_function("emit_eop", emit_function_type, None),
      emit_shift: module.add_function("emit_shift", emit_function_type, None),
      emit_eoi: module.add_function(
        "emit_eoi",
        i32.fn_type(
          &[
            CTX.ptr_type(Generic).into(),
            ACTION.ptr_type(Generic).into(),
            i32.into(),
          ],
          false,
        ),
        None,
      ),
      emit_accept: module.add_function("emit_accept", emit_function_type, None),
      emit_error: module.add_function("emit_error", emit_function_type.clone(), None),
      prime: module.add_function(
        "prime",
        ctx.void_type().fn_type(&[CTX.ptr_type(Generic).into(), i32.into()], false),
        None,
      ),
      memcpy: module.add_function(
        "llvm.memcpy.p0.p0.i32",
        ctx.void_type().fn_type(
          &[
            i8.ptr_type(Generic).into(),
            i8.ptr_type(Generic).into(),
            i32.into(),
            ctx.bool_type().into(),
          ],
          false,
        ),
        None,
      ),
      max: module.add_function(
        "llvm.umax.i32",
        i32.fn_type(&[i32.into(), i32.into()], false),
        None,
      ),
      min: module.add_function(
        "llvm.umin.i32",
        i32.fn_type(&[i32.into(), i32.into()], false),
        None,
      ),
      extend_stack_if_needed: module.add_function(
        "extend_stack_if_needed",
        i32.fn_type(&[CTX.ptr_type(Generic).into(), i32.into()], false),
        None,
      ),
    },
    module,
    exe_engine: None,
  }
}

pub(crate) fn construct_emit_end_of_input(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_eoi;

  let eoi_action = ctx.struct_type(&[i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();
  let current_offset = fn_value.get_nth_param(2).unwrap();

  b.position_at_end(entry);

  let eoi = b
    .build_bitcast(basic_action, eoi_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let eoi_struct = b.build_load(eoi, "").into_struct_value();
  let eoi_struct =
    b.build_insert_value(eoi_struct, i32.const_int(9, false), 0, "").unwrap();
  let eoi_struct = b.build_insert_value(eoi_struct, current_offset, 2, "").unwrap();

  b.build_store(eoi, eoi_struct);

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_eoi.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_emit_end_of_parse(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_eop;

  let eoi_action = ctx.struct_type(&[i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let success = ctx.append_basic_block(fn_value, "SuccessfulParse");
  let failure = ctx.append_basic_block(fn_value, "FailedParse");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  b.position_at_end(entry);

  let state = b.build_struct_gep(parse_ctx, ci.state, "").unwrap();
  let state = b.build_load(state, "");

  let comparison = b.build_int_compare(
    inkwell::IntPredicate::NE,
    state.into_int_value(),
    i32.const_int(FAIL_STATE_FLAG_LLVM as u64, false).into(),
    "",
  );
  b.build_conditional_branch(comparison, success, failure);

  b.position_at_end(success);

  b.build_call(funct.emit_accept, &[parse_ctx.into(), basic_action.into()], "");

  b.build_return(Some(&i32.const_int(1, false)));

  b.position_at_end(failure);

  b.build_call(funct.emit_error, &[parse_ctx.into(), basic_action.into()], "");

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_eop.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_get_adjusted_input_block_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule { builder: b, types, ctx, ctx_indices: ci, fun: funct, .. } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.get_adjusted_input_block;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let attempt_extend = ctx.append_basic_block(fn_value, "Attempt_Extend");
  let valid_window = ctx.append_basic_block(fn_value, "Valid_Window");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let offset_token = fn_value.get_nth_param(1).unwrap().into_pointer_value();
  let requested_size = fn_value.get_nth_param(2).unwrap().into_int_value();

  b.position_at_end(entry);

  let ctx_input_block = b.build_struct_gep(parse_ctx, ci.input_block, "").unwrap();

  let block_offset_ptr = b.build_struct_gep(ctx_input_block, 1, "").unwrap();
  let block_offset = b.build_load(block_offset_ptr, "").into_int_value();

  let block_size_ptr = b.build_struct_gep(ctx_input_block, 2, "").unwrap();
  let block_size = b.build_load(block_size_ptr, "").into_int_value();

  let token_offset = b.build_struct_gep(offset_token, 0, "").unwrap();
  let token_offset = b.build_load(token_offset, "").into_int_value();
  let token_offset = b.build_int_truncate(token_offset, i32.into(), "");

  let needed_size = b.build_int_add(token_offset, requested_size, "");
  let needed_size = b.build_int_sub(needed_size, block_offset, "");

  let comparison =
    b.build_int_compare(inkwell::IntPredicate::UGE, block_size, needed_size, "");

  b.build_conditional_branch(comparison, valid_window, attempt_extend);

  b.position_at_end(attempt_extend);

  b.build_store(block_offset_ptr, token_offset);

  let reader = b.build_struct_gep(parse_ctx, ci.reader, "").unwrap();
  let reader = b.build_load(reader, "");
  let get_byte_block = b.build_struct_gep(parse_ctx, ci.get_input_block, "").unwrap();
  let get_byte_block = b.build_load(get_byte_block, "").into_pointer_value();
  let get_byte_block = CallableValue::try_from(get_byte_block).unwrap();

  b.build_call(get_byte_block, &[reader.into(), ctx_input_block.into()], "");

  b.build_unconditional_branch(valid_window);

  b.position_at_end(valid_window);

  let block = b.build_load(ctx_input_block, "").into_struct_value();

  let ptr = b.build_extract_value(block, 0, "").unwrap().into_pointer_value();
  let offset = b.build_extract_value(block, 1, "").unwrap().into_int_value();
  let size = b.build_extract_value(block, 2, "").unwrap().into_int_value();
  let diff = b.build_int_sub(token_offset, offset, "");
  // offset the pointer by the difference between the token_offset and
  // and the block offset
  let adjusted_size = b.build_int_sub(size, diff, "");
  let adjusted_ptr = b.build_gep(ptr, &[diff.into()], "");
  let block = b.build_insert_value(block, adjusted_ptr, 0, "").unwrap();
  let block = b.build_insert_value(block, adjusted_size, 2, "").unwrap();

  b.build_return(Some(&block));

  if funct.get_adjusted_input_block.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) fn construct_emit_reduce_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_reduce;

  let eoi_action =
    ctx.struct_type(&[i32.into(), i32.into(), i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();
  let production_id = fn_value.get_nth_param(2).unwrap().into_int_value();
  let body_id = fn_value.get_nth_param(3).unwrap().into_int_value();
  let symbol_count = fn_value.get_nth_param(4).unwrap().into_int_value();

  b.position_at_end(entry);

  let reduce = b
    .build_bitcast(basic_action, eoi_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let reduce_struct = b.build_load(reduce, "").into_struct_value();
  let reduce_struct =
    b.build_insert_value(reduce_struct, i32.const_int(6, false), 0, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, production_id, 2, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, body_id, 3, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, symbol_count, 4, "").unwrap();

  b.build_store(reduce, reduce_struct);

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_reduce.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_extend_stack_if_needed(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;
  let i32 = ctx.i32_type();

  let fn_value = funct.extend_stack_if_needed;

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let needed_slot_count = fn_value.get_nth_param(1).unwrap().into_int_value();

  let entry = ctx.append_basic_block(fn_value, "Entry");
  b.position_at_end(entry);

  // Get difference between current goto and the base goto.
  let goto_base_ptr_ptr = b.build_struct_gep(parse_ctx, ci.goto_base, "base").unwrap();
  let goto_base_ptr = b.build_load(goto_base_ptr_ptr, "base").into_pointer_value();
  let goto_top_ptr_ptr = b.build_struct_gep(parse_ctx, ci.goto_top, "top").unwrap();
  let goto_top_ptr = b.build_load(goto_top_ptr_ptr, "top").into_pointer_value();
  let goto_used_bytes = b.build_int_sub(
    b.build_ptr_to_int(goto_top_ptr, ctx.i64_type().into(), "top"),
    b.build_ptr_to_int(goto_base_ptr, ctx.i64_type().into(), "base"),
    "used",
  );
  let goto_used_bytes_i32 = b.build_int_truncate(goto_used_bytes, i32.into(), "");
  let goto_used_slots = b.build_right_shift(
    goto_used_bytes_i32,
    ctx.i32_type().const_int(4, false),
    false,
    "used",
  );

  let goto_size_ptr = b.build_struct_gep(parse_ctx, ci.goto_stack_len, "size").unwrap();
  let goto_slot_count = b.build_load(goto_size_ptr, "size").into_int_value();
  let goto_slots_remaining =
    b.build_int_sub(goto_slot_count, goto_used_slots, "remainder");

  // Compare to the stack size
  let comparison = b.build_int_compare(
    inkwell::IntPredicate::ULT,
    goto_slots_remaining,
    needed_slot_count,
    "",
  );

  let extend_block = ctx.append_basic_block(fn_value, "Extend");
  let free_block = ctx.append_basic_block(fn_value, "FreeStack");
  let update_block = ctx.append_basic_block(fn_value, "UpdateStack");
  let return_block = ctx.append_basic_block(fn_value, "Return");

  b.build_conditional_branch(comparison, extend_block, return_block);

  // If the difference is less than the amount requested:
  b.position_at_end(extend_block);
  // Create a new stack, copy data from old stack to new one
  // and, if the old stack was not the original stack,
  // delete the old stack.

  // create a size that is equal to the needed amount rounded up to the nearest 64bytes
  let new_slot_count = b.build_int_add(goto_used_slots, needed_slot_count, "new_size");
  let new_slot_count =
    b.build_left_shift(new_slot_count, i32.const_int(1, false), "new_size");

  let new_ptr = b
    .build_call(funct.allocate_stack, &[new_slot_count.into()], "")
    .try_as_basic_value()
    .unwrap_left()
    .into_pointer_value();

  b.build_call(
    funct.memcpy,
    &[
      b.build_bitcast(
        new_ptr,
        ctx.i8_type().ptr_type(inkwell::AddressSpace::Generic),
        "",
      )
      .into(),
      b.build_bitcast(
        goto_base_ptr,
        ctx.i8_type().ptr_type(inkwell::AddressSpace::Generic),
        "",
      )
      .into(),
      goto_used_bytes_i32.into(),
      ctx.bool_type().const_int(0, false).into(),
    ],
    "",
  );

  let comparison = b.build_int_compare(
    inkwell::IntPredicate::NE,
    goto_slot_count,
    i32.const_int(8, false).into(),
    "",
  );

  b.build_conditional_branch(comparison, free_block, update_block);

  b.position_at_end(free_block);

  b.build_call(funct.free_stack, &[goto_base_ptr.into(), goto_slot_count.into()], "");

  b.build_unconditional_branch(update_block);
  b.position_at_end(update_block);

  b.build_store(goto_base_ptr_ptr, new_ptr);

  let new_stack_top_ptr = b.build_ptr_to_int(new_ptr, ctx.i64_type(), "new_top");
  let new_stack_top_ptr = b.build_int_add(new_stack_top_ptr, goto_used_bytes, "new_top");
  let new_stack_top_ptr = b.build_int_to_ptr(
    new_stack_top_ptr,
    types.goto.ptr_type(inkwell::AddressSpace::Generic),
    "new_top",
  );

  b.build_store(goto_top_ptr_ptr, new_stack_top_ptr);
  b.build_store(goto_size_ptr, new_slot_count);

  b.build_unconditional_branch(return_block);

  b.position_at_end(return_block);
  b.build_return(Some(&i32.const_int(1, false)));

  if funct.scan.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_scan_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.scan;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let success = ctx.append_basic_block(fn_value, "Produce_Scan_Token");
  let failure = ctx.append_basic_block(fn_value, "Produce_Failed_Token");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let scanner_entry_goto = fn_value.get_nth_param(1).unwrap().into_pointer_value();
  let token_basis = fn_value.get_nth_param(2).unwrap().into_pointer_value();

  b.position_at_end(entry);

  let scan_ctx = b.build_alloca(types.parse_ctx, "");

  b.build_call(funct.init, &[scan_ctx.into()], "");

  let parse_ctx_reader = b.build_struct_gep(parse_ctx, ci.reader, "").unwrap();
  let scan_ctx_reader = b.build_struct_gep(scan_ctx, ci.reader, "").unwrap();

  let parse_input_block = b.build_struct_gep(parse_ctx, ci.input_block, "").unwrap();
  let scan_input_block = b.build_struct_gep(scan_ctx, ci.input_block, "").unwrap();

  let parse_get_input_block =
    b.build_struct_gep(parse_ctx, ci.get_input_block, "").unwrap();
  let scan_get_input_block =
    b.build_struct_gep(scan_ctx, ci.get_input_block, "").unwrap();

  b.build_store(scan_get_input_block, b.build_load(parse_get_input_block, ""));
  b.build_store(scan_input_block, b.build_load(parse_input_block, ""));
  b.build_store(scan_ctx_reader, b.build_load(parse_ctx_reader, ""));
  b.build_store(
    b.build_struct_gep(scan_ctx, ci.state, "").unwrap(),
    i32.const_int(NORMAL_STATE_FLAG_LLVM as u64, false),
  );

  let root_token = b.build_load(token_basis, "");
  let assert_token = b.build_struct_gep(scan_ctx, ci.tok_assert, "").unwrap();
  let anchor_token = b.build_struct_gep(scan_ctx, ci.tok_anchor, "").unwrap();

  b.build_store(assert_token, root_token);
  b.build_store(anchor_token, root_token);

  b.build_call(
    funct.push_state,
    &[
      scan_ctx.into(),
      i32.const_int((NORMAL_STATE_FLAG_LLVM | FAIL_STATE_FLAG_LLVM) as u64, false).into(),
      funct.emit_eop.as_global_value().as_pointer_value().into(),
    ],
    "",
  );

  b.build_call(
    funct.push_state,
    &[
      scan_ctx.into(),
      i32.const_int((NORMAL_STATE_FLAG_LLVM) as u64, false).into(),
      scanner_entry_goto.into(),
    ],
    "",
  );
  // reserve enough space on the stack for an Action enum

  let action = b.build_alloca(types.action.array_type(8), "");
  let action =
    b.build_bitcast(action, types.action.ptr_type(inkwell::AddressSpace::Generic), "");

  b.build_call(funct.next, &[scan_ctx.into(), action.into()], "");

  // copy the input data from the scan context to the parse context

  b.build_store(parse_input_block, b.build_load(scan_input_block, ""));

  // Produce either a failure token or a success token based on
  // outcome of the `next` call.

  let action_type = b.build_struct_gep(action.into_pointer_value(), 0, "").unwrap();

  let action_type = b.build_load(action_type, "");

  let comparison = b.build_int_compare(
    inkwell::IntPredicate::EQ,
    action_type.into_int_value(),
    i32.const_int(7, false),
    "",
  );
  b.build_conditional_branch(comparison, success, failure);

  b.position_at_end(success);
  let offset_min = b.build_struct_gep(assert_token, 0, "").unwrap();
  let offset_min = b.build_load(offset_min, "");
  let offset_max = b.build_struct_gep(anchor_token, 0, "").unwrap();
  let offset_max = b.build_load(offset_max, "");

  let offset_diff =
    b.build_int_sub(offset_max.into_int_value(), offset_min.into_int_value(), "");

  let length = b.build_struct_gep(anchor_token, 1, "").unwrap();

  b.build_store(length, offset_diff);

  let token = b.build_load(anchor_token, "");

  b.build_return(Some(&token));

  b.position_at_end(failure);
  let token = b.build_load(anchor_token, "");
  b.build_return(Some(&token));

  if funct.scan.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_emit_shift_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_shift;

  let eoi_action =
    ctx.struct_type(&[i32.into(), types.token.into(), types.token.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  b.position_at_end(entry);

  // load the anchor token to be used as the skipped symbols

  let anchor_token_ptr = b.build_struct_gep(parse_ctx, ci.tok_anchor, "").unwrap();
  let skip_token = b.build_load(anchor_token_ptr, "").into_struct_value();

  // load the anchor token to be used as the skipped symbols

  let assert_token_ptr = b.build_struct_gep(parse_ctx, ci.tok_assert, "").unwrap();
  let assert_token = b.build_load(assert_token_ptr, "").into_struct_value();

  // The length of the skip token is equal to the tokens offset minus the
  // assert token's offset

  let shift_offset = b.build_extract_value(assert_token, 0, "").unwrap().into_int_value();
  let skip_offset = b.build_extract_value(skip_token, 0, "").unwrap().into_int_value();

  let skip_length = b.build_int_sub(shift_offset, skip_offset, "");

  let skip_token = b.build_insert_value(skip_token, skip_length, 1, "").unwrap();

  let shift = b
    .build_bitcast(basic_action, eoi_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let shift_struct = b.build_load(shift, "").into_struct_value();
  let shift_struct =
    b.build_insert_value(shift_struct, i32.const_int(5, false), 0, "").unwrap();

  let shift_struct = b.build_insert_value(shift_struct, skip_token, 1, "").unwrap();

  let shift_struct = b.build_insert_value(shift_struct, assert_token, 2, "").unwrap();

  b.build_store(shift, shift_struct);

  let assert_length =
    b.build_extract_value(assert_token, 1, "").unwrap().into_int_value();

  let assert_offset = b.build_int_add(assert_length, shift_offset, "");

  let assert_token = b.build_insert_value(assert_token, assert_offset, 0, "").unwrap();
  let assert_token = b
    .build_insert_value(assert_token, ctx.i64_type().const_int(0, false), 3, "")
    .unwrap();

  b.build_store(anchor_token_ptr, assert_token);
  b.build_store(assert_token_ptr, assert_token);

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_shift.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_emit_accept_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_accept;

  let accept_action = ctx.struct_type(&[i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  b.position_at_end(entry);

  let production = b.build_struct_gep(parse_ctx, ci.production, "").unwrap();
  let production = b.build_load(production, "");
  let accept = b
    .build_bitcast(
      basic_action,
      accept_action.ptr_type(inkwell::AddressSpace::Generic),
      "",
    )
    .into_pointer_value();

  let accept_struct = b.build_load(accept, "").into_struct_value();
  let accept_struct =
    b.build_insert_value(accept_struct, i32.const_int(7, false), 0, "").unwrap();
  let accept_struct = b.build_insert_value(accept_struct, production, 2, "").unwrap();

  b.build_store(accept, accept_struct);

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_accept.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_emit_error_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_error;

  let error_action =
    ctx.struct_type(&[i32.into(), types.token.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  b.position_at_end(entry);

  // load the anchor token as the error token

  let error_token = b.build_struct_gep(parse_ctx, ci.tok_anchor, "").unwrap();
  let error_token = b.build_load(error_token, "");

  // load the last production value

  let production = b.build_struct_gep(parse_ctx, ci.production, "").unwrap();
  let production = b.build_load(production, "");

  // build the ParseAction::Error struct

  let error = b
    .build_bitcast(
      basic_action,
      error_action.ptr_type(inkwell::AddressSpace::Generic),
      "",
    )
    .into_pointer_value();

  let error_struct = b.build_load(error, "").into_struct_value();
  let error_struct =
    b.build_insert_value(error_struct, i32.const_int(8, false), 0, "").unwrap();
  let error_struct = b.build_insert_value(error_struct, error_token, 1, "").unwrap();
  let error_struct = b.build_insert_value(error_struct, production, 2, "").unwrap();

  b.build_store(error, error_struct);

  b.build_return(Some(&i32.const_int(1, false)));

  if funct.emit_error.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_init_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module, builder, types, ctx, ctx_indices: ci, fun: funct, ..
  } = ctx;

  let i32 = ctx.i32_type();
  let zero = i32.const_int(0, false);

  let fn_value = funct.init;

  let parse_ctx_ptr = fn_value.get_first_param().unwrap().into_pointer_value();

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "entry");

  builder.position_at_end(entry);

  let goto_stack = builder.build_struct_gep(parse_ctx_ptr, ci.goto_stack, "")?;
  let goto_start = builder.build_gep(goto_stack, &[zero, zero], "");
  let goto_base = builder.build_struct_gep(parse_ctx_ptr, ci.goto_base, "")?;
  let goto_top = builder.build_struct_gep(parse_ctx_ptr, ci.goto_top, "")?;
  let goto_len = builder.build_struct_gep(parse_ctx_ptr, ci.goto_stack_len, "")?;
  let state = builder.build_struct_gep(parse_ctx_ptr, ci.state, "")?;

  builder.build_store(goto_base, goto_start);
  builder.build_store(goto_top, goto_start);
  builder.build_store(goto_len, i32.const_int(8, false));
  builder.build_store(state, i32.const_int(NORMAL_STATE_FLAG_LLVM as u64, false));
  builder.build_return(None);

  if funct.init.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_push_state_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.push_state;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let goto_state = fn_value.get_nth_param(1).unwrap().into_int_value();
  let goto_fn = fn_value.get_nth_param(2).unwrap().into_pointer_value();

  b.position_at_end(entry);
  let new_goto = b.build_insert_value(types.goto.get_undef(), goto_state, 1, "").unwrap();
  let new_goto = b.build_insert_value(new_goto, goto_fn, 0, "").unwrap();

  let goto_top_ptr = b.build_struct_gep(parse_ctx, ci.goto_top, "")?;
  // let goto_top = b.build_in_bounds_gep(goto_top_ptr, &[zero], "");
  let goto_top = b.build_load(goto_top_ptr, "").into_pointer_value();

  b.build_store(goto_top, new_goto);

  let goto_top = b.build_gep(goto_top, &[i32.const_int(1, false)], "");

  b.build_store(goto_top_ptr, goto_top);

  b.build_return(None);

  if funct.push_state.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_pop_state_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  use inkwell::AddressSpace::*;

  let LLVMParserModule {
    module,
    builder: b,
    types,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();

  let fn_value = funct.pop_state;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();

  b.position_at_end(entry);

  let goto_top_ptr = b.build_struct_gep(parse_ctx, ci.goto_top, "")?;
  // let goto_top = b.build_in_bounds_gep(goto_top_ptr, &[zero], "");
  let goto_top = b.build_load(goto_top_ptr, "").into_pointer_value();
  let goto_top = b.build_gep(goto_top, &[i32.const_int(1, false).const_neg()], "");
  b.build_store(goto_top_ptr, goto_top);

  let old_goto = b.build_load(goto_top, "");

  b.build_return(Some(&old_goto));

  if funct.pop_state.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) unsafe fn construct_next_function(
  ctx: &LLVMParserModule,
) -> std::result::Result<(), ()>
{
  let LLVMParserModule {
    module: m,
    builder: b,
    types: t,
    ctx,
    ctx_indices: ci,
    fun: funct,
    ..
  } = ctx;

  let i32 = ctx.i32_type();
  let zero = i32.const_int(0, false);

  let fn_value = funct.next;

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  // Set the context's goto pointers to point to the goto block;
  let block_entry = ctx.append_basic_block(fn_value, "Entry");
  let block_dispatch = ctx.append_basic_block(fn_value, "Dispatch");
  let block_useful_state = ctx.append_basic_block(fn_value, "ModeAppropriateState");
  let block_emit = ctx.append_basic_block(fn_value, "Emit");

  b.position_at_end(block_entry);
  b.build_unconditional_branch(block_dispatch);

  b.position_at_end(block_dispatch);
  let state = b.build_load(b.build_struct_gep(parse_ctx, ci.state, "state")?, "state");
  let goto = b
    .build_call(funct.pop_state, &[parse_ctx.into()], "")
    .try_as_basic_value()
    .unwrap_left()
    .into_struct_value();
  let goto_state = b.build_extract_value(goto, 1, "").unwrap();
  let masked_state = b.build_and(state.into_int_value(), goto_state.into_int_value(), "");
  let condition = b.build_int_compare(inkwell::IntPredicate::NE, masked_state, zero, "");
  b.build_conditional_branch(condition, block_useful_state, block_dispatch);

  b.position_at_end(block_useful_state);
  let gt_fn = CallableValue::try_from(
    b.build_extract_value(goto, 0, "").unwrap().into_pointer_value(),
  )
  .unwrap();

  let should_emit = b.build_call(gt_fn, &[parse_ctx.into(), action.into()], "");

  // should_emit.set_call_convention(11);

  let should_emit_return =
    should_emit.try_as_basic_value().unwrap_left().into_int_value();

  let condition =
    b.build_int_compare(inkwell::IntPredicate::EQ, should_emit_return, zero, "");
  b.build_conditional_branch(condition, block_dispatch, block_emit);

  b.position_at_end(block_emit);
  b.build_return(None);

  if funct.next.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) fn construct_prime_function(
  ctx: &LLVMParserModule,
  sp: &Vec<(usize, u32, String)>,
) -> Result<(), ()>
{
  let i32 = ctx.ctx.i32_type();
  let b = &ctx.builder;
  let funct = &ctx.fun;

  let fn_value = funct.prime;

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let selector = fn_value.get_nth_param(1).unwrap().into_int_value(); // Set the context's goto pointers to point to the goto block;
  let block_entry = ctx.ctx.append_basic_block(fn_value, "Entry");
  b.position_at_end(block_entry);

  let blocks = sp
    .iter()
    .map(|(id, address, ..)| {
      (
        *id,
        ctx.ctx.append_basic_block(fn_value, &create_offset_label(*address as usize)),
        get_parse_function(*address as usize, ctx).as_global_value().as_pointer_value(),
      )
    })
    .collect::<Vec<_>>();

  b.build_call(
    funct.push_state,
    &[
      parse_ctx.into(),
      i32.const_int((NORMAL_STATE_FLAG_LLVM | FAIL_STATE_FLAG_LLVM) as u64, false).into(),
      funct.emit_eop.as_global_value().as_pointer_value().into(),
    ],
    "",
  );

  if (!blocks.is_empty()) {
    b.build_switch(
      selector,
      blocks[0].1.into(),
      &blocks.iter().map(|b| (i32.const_int(b.0 as u64, false), b.1)).collect::<Vec<_>>(),
    );

    for (_, block, fn_ptr) in &blocks {
      b.position_at_end(*block);
      b.build_call(
        funct.push_state,
        &[
          parse_ctx.into(),
          i32.const_int(NORMAL_STATE_FLAG_LLVM as u64, false).into(),
          (*fn_ptr).into(),
        ],
        "",
      );

      b.build_return(None);
    }
  } else {
    b.build_return(None);
  }

  if funct.prime.verify(true) {
    Ok(())
  } else {
    Err(())
  }
}

pub(crate) fn create_offset_label(offset: usize) -> String
{
  format!("off_{:X}", offset)
}

pub(crate) fn construct_parse_functions(
  ctx: &LLVMParserModule,
  output: &BytecodeOutput,
  build_options: &BuildOptions,
) -> Result<(), ()>
{
  // start points
  let start_points = get_exported_productions(output.grammar)
    .iter()
    .enumerate()
    .map(|(i, p)| {
      let address = *(output.state_name_to_offset.get(p.guid_name).unwrap());

      let name = create_offset_label(address as usize);
      (i, address, name)
    })
    .collect::<Vec<_>>();

  construct_prime_function(ctx, &start_points)?;

  let sp_lu =
    start_points.iter().map(|(_, address, _)| *address).collect::<BTreeSet<_>>();

  let mut addresses = output
    .ir_states
    .values()
    .filter(|s| !s.is_scanner())
    .map(|s| {
      let address = *output.state_name_to_offset.get(&s.get_name()).unwrap();
      let pushed_to_stack = sp_lu.contains(&address);
      (address, pushed_to_stack)
    })
    .collect::<VecDeque<_>>();

  let mut seen = BTreeSet::new();
  let mut goto_fn = BTreeSet::new();

  while let Some((address, pushed_to_stack)) = addresses.pop_front() {
    if pushed_to_stack {
      goto_fn.insert(address);
    }

    if seen.insert(address) {
      let mut referenced_addresses = Vec::new();

      let function = get_parse_function(address as usize, ctx);
      let block_entry = ctx.ctx.append_basic_block(function, "Entry");
      ctx.builder.position_at_end(block_entry);

      let pack = InstructionPack {
        fun:           &function,
        output:        output,
        build_options: build_options,
        address:       address as usize,
        is_scanner:    false,
      };

      construct_parse_function_statements(ctx, &pack, &mut referenced_addresses)?;

      for address in referenced_addresses {
        addresses.push_front(address);
      }
    }
  }

  Ok(())
}

pub(crate) struct InstructionPack<'a>
{
  fun:           &'a FunctionValue<'a>,
  output:        &'a BytecodeOutput<'a>,
  build_options: &'a BuildOptions,
  address:       usize,
  is_scanner:    bool,
}

fn construct_parse_function_statements(
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
) -> Result<(usize, String), ()>
{
  let InstructionPack { fun, output, address, is_scanner, .. } = pack;
  let BytecodeOutput { bytecode, offset_to_state_name, .. } = pack.output;

  let mut address = *address;
  let mut is_scanner = *is_scanner;
  let mut return_val = None;

  if address >= bytecode.len() {
    return Ok((bytecode.len(), "".to_string()));
  }
  let i32 = ctx.ctx.i32_type();
  let parse_cxt = fun.get_nth_param(0).unwrap().into_pointer_value();

  if let Some(ir_state_name) = offset_to_state_name.get(&(address as u32)) {
    if let Some(state) = output.ir_states.get(ir_state_name) {
      match state.get_type() {
        IRStateType::ProductionStart
        | IRStateType::ScannerStart
        | IRStateType::ProductionGoto
        | IRStateType::ScannerGoto => {
          if state.get_stack_depth() > 0 {
            ctx.builder.build_call(
              ctx.fun.extend_stack_if_needed,
              &[
                parse_cxt.into(),
                i32.const_int((state.get_stack_depth() + 2) as u64, false).into(),
              ],
              "",
            );
          }
        }
        _ => {}
      }

      is_scanner = state.is_scanner();
    }
  }

  while address < bytecode.len() {
    match bytecode[address] & INSTRUCTION_HEADER_MASK {
      INSTRUCTION::I01_CONSUME => {
        if is_scanner {
          address = construct_scanner_instruction_consume(address, ctx, pack);
        } else {
          construct_instruction_consume(address, ctx, pack, referenced);
          break;
        }
      }
      INSTRUCTION::I02_GOTO => {
        (address, return_val) =
          construct_instruction_goto(address, ctx, pack, referenced);
      }
      INSTRUCTION::I03_SET_PROD => {
        address = construct_instruction_prod(address, ctx, pack);
      }
      INSTRUCTION::I04_REDUCE => {
        construct_instruction_reduce(address, ctx, pack, referenced);
        break;
      }
      INSTRUCTION::I05_TOKEN => {
        address = construct_instruction_token(address, ctx, pack);
      }
      INSTRUCTION::I06_FORK_TO => {
        // TODO
        break;
      }
      INSTRUCTION::I07_SCAN => address += 1,
      INSTRUCTION::I08_NOOP => address += 1,
      INSTRUCTION::I09_VECTOR_BRANCH | INSTRUCTION::I10_HASH_BRANCH => {
        construct_instruction_branch(address, ctx, pack, referenced, is_scanner)?;
        break;
      }
      INSTRUCTION::I11_SET_FAIL_STATE => address += 1,
      INSTRUCTION::I12_REPEAT => address += 1,
      INSTRUCTION::I13_NOOP => address += 1,
      INSTRUCTION::I14_ASSERT_CONSUME => address += 1,
      INSTRUCTION::I15_FAIL => {
        construct_instruction_fail(ctx, pack);
        break;
      }
      INSTRUCTION::I00_PASS | _ => {
        construct_instruction_pass(ctx, pack, return_val);
        break;
      }
    }
  }

  Ok((address, String::default()))
}

fn write_emit_reentrance<'a>(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
)
{
  let bytecode = &pack.output.bytecode;

  let next_address = match bytecode[address] & INSTRUCTION_HEADER_MASK {
    INSTRUCTION::I00_PASS => 0,
    INSTRUCTION::I02_GOTO => {
      if bytecode[address + 1] & INSTRUCTION_HEADER_MASK == INSTRUCTION::I00_PASS {
        (bytecode[address] & GOTO_STATE_ADDRESS_MASK) as usize
      } else {
        address
      }
    }
    _ => address,
  };

  if next_address != 0 {
    ctx.builder.build_call(
      ctx.fun.push_state,
      &[
        pack.fun.get_first_param().unwrap().into_pointer_value().into(),
        ctx.ctx.i32_type().const_int(NORMAL_STATE_FLAG_LLVM as u64, false).into(),
        get_parse_function(next_address, ctx).as_global_value().as_pointer_value().into(),
      ],
      "",
    );
    referenced.push((next_address as u32, true));
  }
}

pub(crate) fn get_parse_function<'a>(
  address: usize,
  ctx: &'a LLVMParserModule,
) -> FunctionValue<'a>
{
  let name = format!("parse_fn_{:X}", address);
  match ctx.module.get_function(&name) {
    Some(function) => function,
    None => ctx.module.add_function(&name, ctx.types.goto_fn, None),
  }
}

pub(crate) fn write_get_input_ptr_lookup<'a>(
  ctx: &'a LLVMParserModule,
  pack: &'a InstructionPack,
  max_length: usize,
  address: usize,
  token_ptr: PointerValue<'a>,
) -> (PointerValue<'a>, IntValue<'a>, IntValue<'a>, IntValue<'a>)
{
  let i32 = ctx.ctx.i32_type();
  let b = &ctx.builder;
  let parse_ctx = pack.fun.get_first_param().unwrap().into_pointer_value();
  // let action_pointer = pack.fun.get_nth_param(1).unwrap().into_pointer_value();
  // let table_name = create_offset_label(address + 800000);

  let input_block = b
    .build_call(
      ctx.fun.get_adjusted_input_block,
      &[
        parse_ctx.into(),
        token_ptr.into(),
        i32.const_int(max_length as u64, false).into(),
      ],
      "",
    )
    .try_as_basic_value()
    .unwrap_left()
    .into_struct_value();

  let input_ptr = b.build_extract_value(input_block, 0, "").unwrap().into_pointer_value();
  let input_size = b.build_extract_value(input_block, 2, "").unwrap().into_int_value();
  let input_offset = b.build_extract_value(input_block, 1, "").unwrap().into_int_value();
  let input_truncated =
    b.build_extract_value(input_block, 3, "").unwrap().into_int_value();

  (input_ptr, input_size, input_offset, input_truncated)
}

pub(crate) fn construct_instruction_branch(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
  is_scanner: bool,
) -> Result<(), ()>
{
  if let Some(data) = BranchTableData::from_bytecode(address, pack.output) {
    let b = &ctx.builder;
    let i32 = ctx.ctx.i32_type();
    let i64 = ctx.ctx.i64_type();

    let parse_ctx = pack.fun.get_nth_param(0).unwrap().into_pointer_value();
    let action_pointer = pack.fun.get_nth_param(1).unwrap().into_pointer_value();

    // Convert the instruction data into table data.
    let table_name = create_offset_label(address + 800000);

    let table_block =
      ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_Table"));

    let TableHeaderData { input_type, lexer_type, scanner_address, .. } = data.data;

    let branches = &data.branches;
    let mut token_ptr = i32.ptr_type(inkwell::AddressSpace::Generic).const_null();
    let mut input_ptr = i32.ptr_type(inkwell::AddressSpace::Generic).const_null();
    let mut input_offset = i32.const_int(0, false);
    let mut input_size = i32.const_int(0, false);
    let mut input_truncated = ctx.ctx.bool_type().const_int(0, false);
    let mut value = i32.const_int(0, false);

    // Prepare the input token if we are working with
    // Token based branch types (TOKEN, BYTE, CODEPOINT, CLASS)
    match input_type {
      INPUT_TYPE::T01_PRODUCTION => {
        b.build_unconditional_branch(table_block);
        b.position_at_end(table_block);
      }
      _ => {
        let peek_mode_ptr =
          b.build_struct_gep(parse_ctx, ctx.ctx_indices.peek_mode, "").unwrap();

        if lexer_type == LEXER_TYPE::ASSERT {
          token_ptr =
            b.build_struct_gep(parse_ctx, ctx.ctx_indices.tok_assert, "").unwrap();

          if !is_scanner {
            let peek_mode_ptr =
              b.build_struct_gep(parse_ctx, ctx.ctx_indices.peek_mode, "").unwrap();
            b.build_store(peek_mode_ptr, i32.const_int(0, false));

            b.build_unconditional_branch(table_block);
            b.position_at_end(table_block);
          }
        } else {
          // Need to increment the peek token by either the previous length of the peek
          // token, or the current length of the assert token.

          let is_peeking_block =
            ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_Is_Peeking"));
          let not_peeking_block =
            ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_Not_Peeking"));
          let dispatch_block =
            ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_Dispatch"));

          token_ptr =
            b.build_struct_gep(parse_ctx, ctx.ctx_indices.tok_peek, "").unwrap();

          let peek_mode = b.build_load(peek_mode_ptr, "").into_int_value();
          let comparison = b.build_int_compare(
            inkwell::IntPredicate::EQ,
            peek_mode,
            i32.const_int(1, false),
            "",
          );

          let peek_token_off_ptr = b.build_struct_gep(token_ptr, 0, "").unwrap();

          b.build_conditional_branch(comparison, is_peeking_block, not_peeking_block);

          b.position_at_end(is_peeking_block);
          let prev_token_len_ptr = b.build_struct_gep(token_ptr, 1, "").unwrap();
          let prev_token_len = b.build_load(prev_token_len_ptr, "").into_int_value();
          let prev_token_off_ptr = b.build_struct_gep(token_ptr, 0, "").unwrap();
          let prev_token_off = b.build_load(prev_token_off_ptr, "").into_int_value();
          let new_off = b.build_int_add(prev_token_len, prev_token_off, "");
          b.build_store(peek_token_off_ptr, new_off);

          b.build_unconditional_branch(table_block);

          b.position_at_end(not_peeking_block);
          let assert_token_ptr =
            b.build_struct_gep(parse_ctx, ctx.ctx_indices.tok_assert, "").unwrap();
          let prev_token_len_ptr = b.build_struct_gep(assert_token_ptr, 1, "").unwrap();
          let prev_token_len = b.build_load(prev_token_len_ptr, "").into_int_value();
          let prev_token_off_ptr = b.build_struct_gep(assert_token_ptr, 0, "").unwrap();
          let prev_token_off = b.build_load(prev_token_off_ptr, "").into_int_value();
          let new_off = b.build_int_add(prev_token_len, prev_token_off, "");

          b.build_store(peek_token_off_ptr, new_off);

          b.build_unconditional_branch(table_block);
          b.position_at_end(table_block);
        };
      }
    }

    let mut build_switch = true;
    let mut build_truncated_input_block_check = true;

    // Creates blocks for each branch, skip, and default.

    let default_block =
      ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_Table_Default"));

    let mut blocks = BTreeMap::new();
    for branch in branches.values() {
      if branch.is_skipped {
        blocks.entry(usize::max_value()).or_insert_with(|| {
          (
            branch.value as u64,
            ctx.ctx.append_basic_block(*pack.fun, &(table_name.clone() + "_skip")),
          )
        });
      } else {
        blocks.entry(branch.address as usize).or_insert_with(|| {
          (
            branch.value as u64,
            ctx.ctx.append_basic_block(
              *pack.fun,
              &(table_name.clone() + "_" + &create_offset_label(branch.address)),
            ),
          )
        });
      }
    }

    match input_type {
      INPUT_TYPE::T02_TOKEN => {
        if data.has_trivial_comparisons() {
          build_switch = false;

          // Store branch data in tuples comprized of (branch address, branch data,  token string)
          let branches = data
            .branches
            .iter()
            .map(|(address, branch)| {
              let sym = data.get_branch_symbol(branch).unwrap();
              let string = match sym.guid {
                id if id.isDefinedSymbol() => {
                  vec![pack
                    .output
                    .grammar
                    .symbols_string_table
                    .get(&id)
                    .unwrap()
                    .as_str()]
                }
                SymbolID::GenericSpace => {
                  vec![" "]
                }
                _ => vec![""],
              };

              (address, branch, string)
            })
            .collect::<Vec<_>>();

          // Build a buffer to store the largest need register size, rounded to 4 bytes.

          let max_size = branches
            .iter()
            .flat_map(|(_, _, s)| s)
            .fold(0, |a, s| usize::max(a, s.len()));

          (input_ptr, input_size, input_offset, input_truncated) =
            write_get_input_ptr_lookup(ctx, pack, max_size, address, token_ptr);

          let buffer_pointer = b.build_array_alloca(
            ctx.ctx.i8_type(),
            i32.const_int(max_size as u64, false),
            "",
          );

          // Perform a memcpy between the input ptr and the buffer, using the smaller
          // of the two input block length and max_size as the amount of bytes to
          // copy.

          let min_size = b
            .build_call(
              ctx.fun.min,
              &[i32.const_int(max_size as u64, false).into(), input_size.into()],
              "",
            )
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();

          // Prepare a pointer to the token's type for later reuse in the switch block

          b.build_call(
            ctx.fun.memcpy,
            &[
              buffer_pointer.into(),
              input_ptr.into(),
              min_size.into(),
              ctx.ctx.bool_type().const_int(0, false).into(),
            ],
            "",
          );

          input_ptr = buffer_pointer;

          let token_type_ptr = b.build_struct_gep(token_ptr, 2, "").unwrap();
          fn string_to_byte_num_and_mask(string: &str, sym: &Symbol) -> (usize, usize)
          {
            string.as_bytes().iter().enumerate().fold((0, 0), |(val, mask), (i, v)| {
              let shift_amount = 8 * i;
              (val | ((*v as usize) << shift_amount), mask | (0xFF << shift_amount))
            })
          }

          let max_length = *branches
            .iter()
            .flat_map(|s| s.2.iter().map(|s| s.len()))
            .collect::<BTreeSet<_>>()
            .last()
            .unwrap();

          for (index, (address, branch, strings)) in branches.iter().enumerate() {
            let sym = data.get_branch_symbol(branch).unwrap();

            let mut comparison = ctx.ctx.i8_type().const_int(0, false);
            for string in strings {
              match sym.byte_length {
                len if len == 1 => {
                  let value = b.build_load(input_ptr, "").into_int_value();
                  comparison = b.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    value,
                    value.get_type().const_int(
                      string_to_byte_num_and_mask(string, sym).0 as u64,
                      false,
                    ),
                    "",
                  );
                }
                len if len <= 8 => {
                  let (byte_string, mask) = string_to_byte_num_and_mask(string, sym);
                  let adjusted_byte = b
                    .build_bitcast(
                      input_ptr,
                      i64.ptr_type(inkwell::AddressSpace::Generic),
                      "",
                    )
                    .into_pointer_value();
                  let value = b.build_load(adjusted_byte, "").into_int_value();
                  let masked_value = b.build_and(
                    value,
                    value.get_type().const_int(mask as u64, false),
                    "",
                  );
                  comparison = b.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    masked_value,
                    value.get_type().const_int(byte_string as u64, false),
                    "",
                  );
                }
                _ => {}
              }

              let this_block =
                ctx.ctx.append_basic_block(*pack.fun, &format!("this_{}", address));

              let next_block = if index == branches.len() - 1 {
                default_block
              } else {
                ctx.ctx.append_basic_block(*pack.fun, &format!("next_{}", address))
              };

              b.build_conditional_branch(comparison, this_block, next_block);
              b.position_at_end(this_block);

              let token_length_ptr = b.build_struct_gep(token_ptr, 1, "").unwrap();

              b.build_store(
                token_length_ptr,
                ctx.ctx.i64_type().const_int(
                  ((sym.byte_length as u64) | ((sym.code_point_length as u64) << 32)),
                  false,
                ),
              );

              if !branch.is_skipped {
                b.build_store(
                  token_type_ptr,
                  ctx.ctx.i64_type().const_int(branch.value as u64, false),
                );
              }

              b.build_unconditional_branch(if branch.is_skipped {
                blocks.get(&usize::max_value()).unwrap().1
              } else {
                blocks.get(&(&branch.address)).unwrap().1
              });
              b.position_at_end(next_block);
            }
          }
        } else {
          let fun = get_parse_function(scanner_address as usize, ctx)
            .as_global_value()
            .as_pointer_value()
            .into();

          referenced.push((scanner_address as u32, true));

          let scan_tok = b
            .build_call(ctx.fun.scan, &[parse_ctx.into(), fun, token_ptr.into()], "")
            .try_as_basic_value()
            .unwrap_left()
            .into_struct_value();

          b.build_store(token_ptr, scan_tok);

          let val_ptr = b.build_struct_gep(token_ptr, 2, "").unwrap();

          value = b.build_load(val_ptr, "").into_int_value();
        }
      }
      INPUT_TYPE::T01_PRODUCTION => {
        build_truncated_input_block_check = false;
        let production_ptr =
          b.build_struct_gep(parse_ctx, ctx.ctx_indices.production, "").unwrap();
        value = b.build_load(production_ptr, "").into_int_value();
      }
      _ => {
        build_truncated_input_block_check = false;
        match input_type {
          INPUT_TYPE::T05_BYTE => {
            (input_ptr, input_size, input_offset, input_truncated) =
              write_get_input_ptr_lookup(ctx, pack, 1, address, token_ptr);

            value = b.build_load(input_ptr, "").into_int_value();
          }

          INPUT_TYPE::T03_CLASS => {
            (input_ptr, input_size, input_offset, input_truncated) =
              write_get_input_ptr_lookup(ctx, pack, 4, address, token_ptr);

            // TODO: Use Class lookup function.
            value = i32.const_int(0, false);
          }

          INPUT_TYPE::T04_CODEPOINT => {
            (input_ptr, input_size, input_offset, input_truncated) =
              write_get_input_ptr_lookup(ctx, pack, 4, address, token_ptr);
            // TODO: Use Codepoint lookup function.
            value = i32.const_int(0, false);
          }
          _ => {}
        };
      }
    }

    if build_switch {
      // Create Switch statements.
      let value_type = value.get_type();
      let mut cases = vec![];

      for (address, (value, block)) in &blocks {
        if (*address == usize::max_value()) {
          cases.push((value_type.const_int(*value, false), *block));
        } else {
          cases.push((value_type.const_int(*value, false), *block));
        }
      }

      b.build_switch(value, default_block, &cases);
    }

    // Write branches, ending with the default branch.

    for (address, (_, block)) in &blocks {
      b.position_at_end(*block);
      if *address == usize::max_value() {
        create_skip_code(b, token_ptr, i64, table_block);
      } else {
        construct_parse_function_statements(
          ctx,
          &InstructionPack { address: *address, is_scanner, ..*pack },
          referenced,
        )?;
      }
    }

    b.position_at_end(default_block);

    if build_truncated_input_block_check {
      let good_size_block = ctx
        .ctx
        .append_basic_block(*pack.fun, &(table_name.clone() + "_have_sizable_block"));

      let truncated_block = ctx
        .ctx
        .append_basic_block(*pack.fun, &(table_name.clone() + "_block_is_truncated"));

      let comparison = b.build_int_compare(
        inkwell::IntPredicate::EQ,
        input_truncated,
        ctx.ctx.bool_type().const_int(1 as u64, false),
        "",
      );
      b.build_conditional_branch(comparison, truncated_block, good_size_block);

      b.position_at_end(truncated_block);
      b.build_call(
        ctx.fun.push_state,
        &[
          parse_ctx.into(),
          i32.const_int(NORMAL_STATE_FLAG_LLVM as u64, false).into(),
          pack.fun.as_global_value().as_pointer_value().into(),
        ],
        "",
      );

      b.build_call(
        ctx.fun.emit_eoi,
        &[parse_ctx.into(), action_pointer.into(), input_offset.into()],
        "",
      );
      b.build_return(Some(&i32.const_int(1, false)));
      b.position_at_end(good_size_block);
    }

    construct_parse_function_statements(
      ctx,
      &InstructionPack {
        address: (pack.output.bytecode[address + 3] as usize) + address,
        is_scanner,
        ..*pack
      },
      referenced,
    )?;
  }

  Ok(())
}

pub(crate) fn create_skip_code(
  b: &Builder,
  token_ptr: PointerValue,
  i64: inkwell::types::IntType,
  table_block: inkwell::basic_block::BasicBlock,
)
{
  let off_ptr = b.build_struct_gep(token_ptr, 0, "").unwrap();
  let len_ptr = b.build_struct_gep(token_ptr, 1, "").unwrap();
  let off = b.build_load(off_ptr, "offset").into_int_value();
  let len = b.build_load(len_ptr, "length").into_int_value();
  let new_off = b.build_int_add(off, len, "new_offset");
  b.build_store(off_ptr, new_off);
  b.build_store(len_ptr, i64.const_int(0, false));
  b.build_unconditional_branch(table_block);
}

pub(crate) fn construct_scanner_instruction_consume(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
) -> usize
{
  let b = &ctx.builder;

  let parse_ctx = pack.fun.get_first_param().unwrap().into_pointer_value();

  let assert_tok_ptr =
    b.build_struct_gep(parse_ctx, ctx.ctx_indices.tok_assert, "").unwrap();
  let assert_tok = b.build_load(assert_tok_ptr, "").into_struct_value();

  let assert_off = b.build_extract_value(assert_tok, 0, "").unwrap().into_int_value();
  let assert_len = b.build_extract_value(assert_tok, 1, "").unwrap().into_int_value();
  let assert_off = b.build_int_add(assert_len, assert_off, "");
  let assert_tok = b.build_insert_value(assert_tok, assert_off, 0, "").unwrap();
  let assert_tok = b
    .build_insert_value(assert_tok, ctx.ctx.i64_type().const_int(0, false), 2, "")
    .unwrap();

  b.build_store(assert_tok_ptr, assert_tok);

  address + 1
}

pub(crate) fn construct_instruction_consume(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
)
{
  let parse_ctx = pack.fun.get_first_param().unwrap().into_pointer_value();

  write_emit_reentrance(address + 1, ctx, pack, referenced);

  let b = &ctx.builder;
  let val = b
    .build_call(
      ctx.fun.emit_shift,
      &[
        parse_ctx.into(),
        pack.fun.get_nth_param(1).unwrap().into_pointer_value().into(),
      ],
      "",
    )
    .try_as_basic_value()
    .unwrap_left();

  b.build_return(Some(&val));
}

pub(crate) fn construct_instruction_reduce(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
)
{
  let parse_ctx = pack.fun.get_first_param().unwrap().into_pointer_value();
  let instruction = pack.output.bytecode[address];
  let symbol_count = instruction >> 16 & 0x0FFF;
  let body_id = instruction & 0xFFFF;

  write_emit_reentrance(address + 1, ctx, pack, referenced);

  let b = &ctx.builder;
  let prod = b.build_struct_gep(parse_ctx, ctx.ctx_indices.production, "").unwrap();
  let prod = b.build_load(prod, "").into_int_value();
  let val = b
    .build_call(
      ctx.fun.emit_reduce,
      &[
        parse_ctx.into(),
        pack.fun.get_nth_param(1).unwrap().into_pointer_value().into(),
        prod.into(),
        ctx.ctx.i32_type().const_int(body_id as u64, false).into(),
        ctx.ctx.i32_type().const_int(symbol_count as u64, false).into(),
      ],
      "",
    )
    .try_as_basic_value()
    .unwrap_left();

  b.build_return(Some(&val));
}

pub(crate) fn construct_instruction_goto<'a>(
  address: usize,
  ctx: &'a LLVMParserModule,
  pack: &'a InstructionPack,
  referenced: &mut Vec<(u32, bool)>,
) -> (usize, Option<IntValue<'a>>)
{
  let bytecode = &pack.output.bytecode;
  let goto_offset = bytecode[address] & GOTO_STATE_ADDRESS_MASK;
  let goto_function = get_parse_function(goto_offset as usize, ctx);
  let LLVMParserModule { ctx, builder, fun, .. } = ctx;

  if bytecode[address + 1] & INSTRUCTION_HEADER_MASK == INSTRUCTION::I00_PASS {
    // Call the function directly. This should end up as a tail call.
    let return_val = builder
      .build_call(
        goto_function,
        &[
          pack.fun.get_first_param().unwrap().into_pointer_value().into(),
          pack.fun.get_nth_param(1).unwrap().into_pointer_value().into(),
        ],
        "",
      )
      .try_as_basic_value()
      .unwrap_left()
      .into_int_value();
    (address + 1, Some(return_val))
  } else {
    builder.build_call(
      fun.push_state,
      &[
        pack.fun.get_first_param().unwrap().into_pointer_value().into(),
        ctx.i32_type().const_int(NORMAL_STATE_FLAG_LLVM as u64, false).into(),
        goto_function.as_global_value().as_pointer_value().into(),
      ],
      "",
    );

    referenced.push((goto_offset, true));

    (address + 1, None)
  }
}

pub(crate) fn construct_instruction_prod(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
) -> usize
{
  let production_id = pack.output.bytecode[address] & INSTRUCTION_CONTENT_MASK;
  let parse_ctx = pack.fun.get_nth_param(0).unwrap().into_pointer_value();
  let b = &ctx.builder;
  let production_ptr =
    b.build_struct_gep(parse_ctx, ctx.ctx_indices.production, "").unwrap();
  b.build_store(
    production_ptr,
    ctx.ctx.i32_type().const_int(production_id as u64, false),
  );
  address + 1
}

fn construct_instruction_token(
  address: usize,
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
) -> usize
{
  let token_value = pack.output.bytecode[address] & 0x00FF_FFFF;
  let parse_ctx = pack.fun.get_nth_param(0).unwrap().into_pointer_value();
  let b = &ctx.builder;
  let anchor_token =
    b.build_struct_gep(parse_ctx, ctx.ctx_indices.tok_anchor, "").unwrap();
  let anchor_type = b.build_struct_gep(anchor_token, 3, "").unwrap();
  b.build_store(anchor_type, ctx.ctx.i64_type().const_int(token_value as u64, false));
  address + 1
}

pub(crate) fn construct_instruction_pass(
  ctx: &LLVMParserModule,
  pack: &InstructionPack,
  return_val: Option<IntValue>,
)
{
  let parse_ctx = pack.fun.get_nth_param(0).unwrap().into_pointer_value();
  let b = &ctx.builder;
  let state_ptr = b.build_struct_gep(parse_ctx, ctx.ctx_indices.state, "").unwrap();
  b.build_store(
    state_ptr,
    ctx.ctx.i32_type().const_int(NORMAL_STATE_FLAG_LLVM as u64, false),
  );

  if let Some(return_val) = return_val {
    b.build_return(Some(&return_val));
  } else {
    b.build_return(Some(&ctx.ctx.i32_type().const_int(0, false)));
  }
}

pub(crate) fn construct_instruction_fail(ctx: &LLVMParserModule, pack: &InstructionPack)
{
  let parse_ctx = pack.fun.get_nth_param(0).unwrap().into_pointer_value();
  let b = &ctx.builder;
  let state_ptr = b.build_struct_gep(parse_ctx, ctx.ctx_indices.state, "").unwrap();
  b.build_store(
    state_ptr,
    ctx.ctx.i32_type().const_int(FAIL_STATE_FLAG_LLVM as u64, false),
  );
  b.build_return(Some(&ctx.ctx.i32_type().const_int(0, false)));
}

/// Compile a LLVM parser module from Hydrocarbon bytecode.
pub fn compile_from_bytecode<'a>(
  module_name: &str,
  llvm_context: &'a Context,
  build_options: &BuildOptions,
  output: &BytecodeOutput,
) -> core::result::Result<LLVMParserModule<'a>, ()>
{
  let mut parse_context = construct_context(module_name, &llvm_context);
  let ctx = &mut parse_context;

  unsafe {
    construct_emit_accept_function(ctx)?;
    construct_emit_end_of_input(ctx)?;
    construct_emit_end_of_parse(ctx)?;
    construct_emit_reduce_function(ctx)?;
    construct_emit_shift_function(ctx)?;
    construct_get_adjusted_input_block_function(ctx)?;
    construct_init_function(ctx)?;
    construct_next_function(ctx)?;
    construct_pop_state_function(ctx)?;
    construct_push_state_function(ctx)?;
    construct_scan_function(ctx)?;
    construct_emit_error_function(ctx)?;
    construct_extend_stack_if_needed(ctx)?;
  }

  construct_parse_functions(ctx, output, build_options)?;

  parse_context.fun.push_state.add_attribute(
    inkwell::attributes::AttributeLoc::Function,
    parse_context.ctx.create_string_attribute("alwaysinline", ""),
  );

  Ok(parse_context)
}
