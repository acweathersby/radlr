use super::{
  fastCC,
  input_block_indices::{
    InputBlockEnd,
    InputBlockPtr,
    InputBlockSize,
    InputBlockStart,
    InputBlockTruncated,
  },
  parser_functions::{
    build_tail_call_with_return,
    construct_dispatch_function,
    construct_next_function,
    construct_parse_function,
    construct_scan,
  },
  CTX_AGGREGATE_INDICES as CTX,
  FAIL_STATE_FLAG_LLVM,
};
use crate::{
  build,
  compile::BytecodeOutput,
  llvm::{LLVMParserModule, LLVMTypes, PublicFunctions, NORMAL_STATE_FLAG_LLVM},
  types::*,
};
use inkwell::{
  builder::Builder,
  context::Context,
  module::Linkage,
  types::IntType,
  values::{BasicMetadataValueEnum, CallSiteValue, CallableValue, IntValue, PointerValue},
};

pub(crate) fn construct_module<'a>(module_name: &str, ctx: &'a Context) -> LLVMParserModule<'a> {
  use inkwell::AddressSpace::*;
  let module = ctx.create_module(module_name);
  let builder = ctx.create_builder();

  let i8 = ctx.i8_type();
  let bool = ctx.bool_type();
  let i64 = ctx.i64_type();
  let i32 = ctx.i32_type();
  let CP_INFO = ctx.opaque_struct_type("s.CP_INFO");
  let READER = ctx.opaque_struct_type("s.READER");
  let ACTION = ctx.opaque_struct_type("s.ACTION");
  let CTX = ctx.opaque_struct_type("s.CTX");
  let GOTO = ctx.opaque_struct_type("s.Goto");
  let TOKEN = ctx.opaque_struct_type("s.Token");
  let INPUT_BLOCK = ctx.opaque_struct_type("s.InputBlock");
  let AST_OBJ = ctx.opaque_struct_type("s.AST_OBJ");
  let AST_SLOT_SLICE = ctx.struct_type(&[AST_OBJ.ptr_type(Generic).into(), i32.into()], false);
  let CTX_PTR = CTX.ptr_type(Generic);
  let ACTION_PTR = ACTION.ptr_type(Generic);
  let AST_PARSE_RESULT = ctx.opaque_struct_type("s.ParseResult");

  AST_PARSE_RESULT.set_body(&[i8.array_type((std::mem::size_of::<ParseResult<u32>>()) as u32).into()], false);

  let SHIFT_HANDLER_FUNCTION = ctx
    .void_type()
    .fn_type(&[CTX_PTR.into(), ACTION_PTR.into(), AST_SLOT_SLICE.ptr_type(Generic).into()], false);

  let RESULT_HANDLER_FUNCTION = AST_PARSE_RESULT
    .fn_type(&[CTX_PTR.into(), ACTION_PTR.into(), AST_SLOT_SLICE.ptr_type(Generic).into()], false);

  let TAIL_CALLABLE_PARSE_FUNCTION =
    ctx.void_type().fn_type(&[CTX_PTR.into(), ACTION_PTR.into()], false);

  AST_OBJ.set_body(&[i8.array_type(8 + (std::mem::size_of::<Token>() * 2) as u32).into()], false);

  ACTION.set_body(
    &[
      i32.into(),
      i32.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
    ],
    false,
  );

  let GOTO_FN = i32.fn_type(&[CTX_PTR.into(), ACTION_PTR.into()], false);

  GOTO.set_body(
    &[TAIL_CALLABLE_PARSE_FUNCTION.ptr_type(Generic).into(), i32.into(), i32.into()],
    false,
  );

  CP_INFO.set_body(&[i32.into(), i32.into()], false);

  TOKEN.set_body(&[i64.into(), i64.into(), i64.into(), i64.into()], false);

  INPUT_BLOCK.set_body(
    &[
      // Input pointer
      i8.ptr_type(Generic).into(),
      // start index
      i32.into(),
      // end index
      i32.into(),
      // readable bytes
      i32.into(),
      // is truncated
      ctx.bool_type().into(),
    ],
    false,
  );
  let get_input_block_type = ctx
    .void_type()
    .fn_type(&[READER.ptr_type(Generic).into(), INPUT_BLOCK.ptr_type(Generic).into()], false)
    .ptr_type(Generic);

  CTX.set_body(
    &[
      INPUT_BLOCK.into(),
      GOTO.ptr_type(Generic).into(),
      READER.ptr_type(Generic).into(),
      get_input_block_type.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i64.into(),
      i32.into(),
      i32.into(),
      i32.into(),
      i32.into(),
      bool.into(),
      bool.into(),
    ],
    false,
  );

  let internal_linkage = Some(Linkage::Private);
  let internal_linkage = None;

  let ast_function =
    ctx.void_type().fn_type(&[i32.into(), AST_SLOT_SLICE.ptr_type(Generic).into()], false);

  let fun = PublicFunctions {
    /// Public functions
    init: module.add_function(
      "init",
      ctx.void_type().fn_type(&[CTX_PTR.into(), READER.ptr_type(Generic).into()], false),
      Some(Linkage::External),
    ),
    ast_builder: module.add_function(
      "ast_builder",
      AST_PARSE_RESULT
        .fn_type(&[
          CTX_PTR.into(), 
          ast_function.ptr_type(Generic).ptr_type(Generic).into(),
          SHIFT_HANDLER_FUNCTION.ptr_type(Global).into(),
          RESULT_HANDLER_FUNCTION.ptr_type(Global).into()
          ], false),
      Some(Linkage::External),
    ),
    drop: module.add_function(
      "drop",
      ctx.void_type().fn_type(&[CTX_PTR.into()], false),
      Some(Linkage::External),
    ),
    prime: module.add_function(
      "prime",
      ctx.void_type().fn_type(&[CTX_PTR.into(), i32.into()], false),
      Some(Linkage::External),
    ),
    next: module.add_function("next", TAIL_CALLABLE_PARSE_FUNCTION, None),
    /// Provided by parser host -------------------------------------------------
    get_token_class_from_codepoint: module.add_function(
      "sherpa_get_token_class_from_codepoint",
      i32.fn_type(&[i32.into()], false),
      Some(Linkage::External),
    ),
    allocate_stack: module.add_function(
      "sherpa_allocate_stack",
      GOTO.ptr_type(Generic).fn_type(&[i64.into()], false),
      Some(Linkage::External),
    ),
    free_stack: module.add_function(
      "sherpa_free_stack",
      ctx.void_type().fn_type(&[GOTO.ptr_type(Generic).into(), i64.into()], false),
      Some(Linkage::External),
    ),
    /// ------------------------------------------------------------------------
    // These functions can be tail called, as they all use the same interface
    dispatch: module.add_function("dispatch", TAIL_CALLABLE_PARSE_FUNCTION, internal_linkage),
    emit_accept: module.add_function("emit_accept", TAIL_CALLABLE_PARSE_FUNCTION, internal_linkage),
    emit_error: module.add_function(
      "emit_error",
      TAIL_CALLABLE_PARSE_FUNCTION.clone(),
      internal_linkage,
    ),
    emit_eop: module.add_function("emit_eop", TAIL_CALLABLE_PARSE_FUNCTION, internal_linkage),
    /// ------------------------------------------------------------------------
    ///
    emit_reduce: module.add_function(
      "emit_reduce",
      ctx.void_type().fn_type(
        &[CTX.ptr_type(Generic).into(), ACTION_PTR.into(), i32.into(), i32.into(), i32.into()],
        false,
      ),
      internal_linkage,
    ),
    emit_shift: module.add_function(
      "emit_shift",
      ctx
        .void_type()
        .fn_type(&[ACTION_PTR.into(), i64.into(), i64.into(), i64.into(), i64.into()], false),
      internal_linkage,
    ),
    emit_eoi: module.add_function(
      "emit_eoi",
      ctx
        .void_type()
        .fn_type(&[CTX.ptr_type(Generic).into(), ACTION_PTR.into(), i32.into()], false),
      internal_linkage,
    ),
    /// ------------------------------------------------------------------------
    get_utf8_codepoint_info: module.add_function(
      "get_utf8_codepoint_info",
      CP_INFO.fn_type(&[i8.ptr_type(Generic).into()], false),
      internal_linkage,
    ),
    merge_utf8_part: module.add_function(
      "merge_utf8_part",
      i32.fn_type(&[i8.ptr_type(Generic).into(), i32.into(), i32.into()], false),
      internal_linkage,
    ),
    internal_free_stack: module.add_function(
      "internal_free_stack",
      ctx.void_type().fn_type(&[CTX.ptr_type(Generic).into()], false),
      internal_linkage,
    ),
    get_adjusted_input_block: module.add_function(
      "get_adjusted_input_block",
      INPUT_BLOCK.fn_type(&[CTX.ptr_type(Generic).into(), i32.into(), i32.into()], false),
      internal_linkage,
    ),
    scan: module.add_function(
      "scan",
      ctx.void_type().fn_type(
        &[
          CTX.ptr_type(Generic).into(),
          TAIL_CALLABLE_PARSE_FUNCTION.ptr_type(Generic).into(),
          i64.into(),
          i64.into(),
        ],
        false,
      ),
      internal_linkage,
    ),
    push_state: module.add_function(
      "push_state",
      ctx.void_type().fn_type(
        &[
          CTX.ptr_type(Generic).into(),
          i32.into(),
          TAIL_CALLABLE_PARSE_FUNCTION.ptr_type(Generic).into(),
        ],
        false,
      ),
      internal_linkage,
    ),
    pop_state: module.add_function(
      "pop_state",
      GOTO.fn_type(&[CTX.ptr_type(Generic).into()], false),
      internal_linkage,
    ),
    extend_stack_if_needed: module.add_function(
      "extend_stack_if_needed",
      i32.fn_type(&[CTX.ptr_type(Generic).into(), i32.into()], false),
      internal_linkage,
    ),
    /// LLVM intrinsics ------------------------------------------------------------
    memcpy: module.add_function(
      "llvm.memcpy.p0i8.p0i8.i32",
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
    memset: module.add_function(
      "llvm.memset.p0.i32",
      ctx.void_type().fn_type(
        &[i8.ptr_type(Generic).into(), i8.into(), i32.into(), ctx.bool_type().into()],
        false,
      ),
      None,
    ),
    max: module.add_function("llvm.umax.i32", i32.fn_type(&[i32.into(), i32.into()], false), None),
    min: module.add_function("llvm.umin.i32", i32.fn_type(&[i32.into(), i32.into()], false), None),
    ctlz_i8: module.add_function(
      "llvm.ctlz.i8",
      i8.fn_type(&[i8.into(), ctx.bool_type().into()], false),
      None,
    ),
  };

  // Set all functions that are not part of the public interface to fastCC.
  fun.dispatch.set_call_conventions(fastCC);
  fun.internal_free_stack.set_call_conventions(fastCC);
  fun.scan.set_call_conventions(fastCC);
  fun.emit_accept.set_call_conventions(fastCC);
  fun.emit_error.set_call_conventions(fastCC);
  fun.emit_eop.set_call_conventions(fastCC);
  fun.emit_reduce.set_call_conventions(fastCC);
  fun.emit_shift.set_call_conventions(fastCC);
  fun.emit_eoi.set_call_conventions(fastCC);
  fun.pop_state.set_call_conventions(fastCC);
  fun.push_state.set_call_conventions(fastCC);
  fun.get_utf8_codepoint_info.set_call_conventions(fastCC);
  fun.get_token_class_from_codepoint.set_call_conventions(fastCC);
  fun.get_adjusted_input_block.set_call_conventions(fastCC);
  fun.extend_stack_if_needed.set_call_conventions(fastCC);

  LLVMParserModule {
    builder,
    ctx,
    types: LLVMTypes {
      TAIL_CALLABLE_PARSE_FUNCTION,
      SHIFT_HANDLER_FUNCTION,
      RESULT_HANDLER_FUNCTION,
      stack_struct: AST_SLOT_SLICE,
      ast_slot: AST_OBJ,
      reader: READER,
      action: ACTION,
      token: TOKEN,
      parse_ctx: CTX,
      goto: GOTO,
      goto_fn: GOTO_FN,
      input_block: INPUT_BLOCK,
      cp_info: CP_INFO,
      parse_result: AST_PARSE_RESULT,
    },
    fun,
    module,
    exe_engine: None,
  }
}

pub(crate) fn construct_emit_end_of_input(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, fun: funct, .. } = module;

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
  let eoi_struct = b.build_insert_value(eoi_struct, i32.const_int(9, false), 0, "").unwrap();
  let eoi_struct = b.build_insert_value(eoi_struct, current_offset, 2, "").unwrap();

  b.build_store(eoi, eoi_struct);

  b.build_return(None);

  if funct.emit_eoi.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not build emit_eoi function"))
  }
}

pub(crate) unsafe fn construct_emit_end_of_parse(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();
  let bool = ctx.bool_type();

  let fn_value = funct.emit_eop;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let success = ctx.append_basic_block(fn_value, "SuccessfulParse");
  let failure = ctx.append_basic_block(fn_value, "FailedParse");

  let parse_ctx = fn_value.get_nth_param(0)?.into_pointer_value();
  let basic_action = fn_value.get_nth_param(1)?.into_pointer_value();

  b.position_at_end(entry);

  // Update the active state to be inactive
  CTX::is_active.store(module, parse_ctx, bool.const_zero())?;

  let comparison = b.build_int_compare(
    inkwell::IntPredicate::NE,
    CTX::state.load(module, parse_ctx)?.into_int_value(),
    i32.const_int(FAIL_STATE_FLAG_LLVM as u64, false).into(),
    "",
  );
  b.build_conditional_branch(comparison, success, failure);

  b.position_at_end(success);

  build_tail_call_with_return(module, fn_value, funct.emit_accept);

  b.position_at_end(failure);

  build_tail_call_with_return(module, fn_value, funct.emit_error);

  if funct.emit_eop.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not build emit_eop function"))
  }
}

pub(crate) unsafe fn construct_get_adjusted_input_block_function(
  module: &LLVMParserModule,
) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();
  let bool = ctx.bool_type();

  let fn_value = funct.get_adjusted_input_block;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let can_extend_check = ctx.append_basic_block(fn_value, "CanExtendCheck");
  let attempt_extend = ctx.append_basic_block(fn_value, "Attempt_Extend");
  let valid_window = ctx.append_basic_block(fn_value, "Valid_Window");

  let parse_ctx = fn_value.get_nth_param(0)?.into_pointer_value();
  let byte_offset = fn_value.get_nth_param(1)?.into_int_value();
  let needed_size = fn_value.get_nth_param(2)?.into_int_value();

  b.position_at_end(entry);

  let ctx_input_block = CTX::input_block.get_ptr(module, parse_ctx)?;

  let requested_end_index = b.build_int_add(byte_offset, needed_size, "");

  // if the requested_end_cursor_position is > the blocks end position, and the
  // the block is truncated, then request a new block.

  let end_byte_index_ptr = b.build_struct_gep(ctx_input_block, InputBlockEnd, "end_ptr")?;
  let end_byte_index = b.build_load(end_byte_index_ptr, "end").into_int_value();

  let comparison =
    b.build_int_compare(inkwell::IntPredicate::UGE, end_byte_index, requested_end_index, "");

  b.build_conditional_branch(comparison, valid_window, can_extend_check);

  b.position_at_end(can_extend_check);

  let truncated_ptr = b.build_struct_gep(ctx_input_block, InputBlockTruncated, "truncated_ptr")?;
  let truncated = b.build_load(truncated_ptr, "end").into_int_value();

  let comparison =
    b.build_int_compare(inkwell::IntPredicate::EQ, truncated, bool.const_int(1, false), "");

  b.build_conditional_branch(comparison, attempt_extend, valid_window);

  // If the truncated value is `true`, then we can extend:

  b.position_at_end(attempt_extend);

  let block_start_ptr = b.build_struct_gep(ctx_input_block, InputBlockStart, "")?;

  b.build_store(block_start_ptr, byte_offset);

  b.build_call(
    CallableValue::try_from(CTX::get_input_block.load(module, parse_ctx)?.into_pointer_value())?,
    &[CTX::reader.load(module, parse_ctx)?.into(), ctx_input_block.into()],
    "",
  );

  b.build_unconditional_branch(valid_window);

  // otherwise we can only adjust the remaining value:

  b.position_at_end(valid_window);

  let block = b.build_load(ctx_input_block, "").into_struct_value();

  let ptr = b.build_extract_value(block, InputBlockPtr, "")?.into_pointer_value();
  let start = b.build_extract_value(block, InputBlockStart, "")?.into_int_value();
  let end = b.build_extract_value(block, InputBlockEnd, "")?.into_int_value();
  let diff = b.build_int_sub(byte_offset, start, "");
  // offset the pointer by the difference between the token_offset and
  // and the block offset
  let adjusted_size = b.build_int_sub(end, byte_offset, "");
  let adjusted_ptr = b.build_gep(ptr, &[diff.into()], "");
  let block = b.build_insert_value(block, adjusted_ptr, InputBlockPtr, "")?;
  let block = b.build_insert_value(block, adjusted_size, InputBlockSize, "")?;

  b.build_return(Some(&block));

  if funct.get_adjusted_input_block.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate get_adjusted_input_block"))
  }
}

pub(crate) fn construct_emit_reduce_function(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_reduce;

  let reduce_action =
    ctx.struct_type(&[i32.into(), i32.into(), i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();
  let production_id = fn_value.get_nth_param(2).unwrap().into_int_value();
  let rule_id = fn_value.get_nth_param(3).unwrap().into_int_value();
  let symbol_count = fn_value.get_nth_param(4).unwrap().into_int_value();

  b.position_at_end(entry);

  let reduce = b
    .build_bitcast(basic_action, reduce_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let reduce_struct = b.build_load(reduce, "").into_struct_value();
  let reduce_struct = b.build_insert_value(reduce_struct, i32.const_int(6, false), 0, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, production_id, 2, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, rule_id, 3, "").unwrap();
  let reduce_struct = b.build_insert_value(reduce_struct, symbol_count, 4, "").unwrap();

  b.build_store(reduce, reduce_struct);

  b.build_return(None);

  if funct.emit_reduce.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate emit_reduce"))
  }
}

pub(crate) unsafe fn construct_drop(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, fun: funct, .. } = module;

  let fn_value = funct.drop;

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();

  b.position_at_end(ctx.append_basic_block(fn_value, "Entry"));

  build_fast_call(module, funct.internal_free_stack, &[parse_ctx.into()]);

  b.build_return(None);

  if funct.drop.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate drop"))
  }
}

pub(crate) unsafe fn construct_internal_free_stack(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx, types, fun: funct, .. } = module;
  let i32 = ctx.i32_type();

  let fn_value = funct.internal_free_stack;

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();

  b.position_at_end(ctx.append_basic_block(fn_value, "Entry"));
  let empty_stack = ctx.append_basic_block(fn_value, "empty_stack");
  let free_stack = ctx.append_basic_block(fn_value, "free_stack");

  let goto_slot_count = CTX::goto_stack_size.load(module, parse_ctx)?.into_int_value();

  let c = b.build_int_compare(inkwell::IntPredicate::NE, goto_slot_count, i32.const_zero(), "");

  b.build_conditional_branch(c, free_stack, empty_stack);
  b.position_at_end(free_stack);

  let goto_byte_size =
    b.build_left_shift(goto_slot_count, ctx.i32_type().const_int(4, false), "goto_byte_size");
  let goto_total_bytes_64 = b.build_int_cast(goto_byte_size, ctx.i64_type(), "");

  let goto_remaining = CTX::goto_remaining.load(module, parse_ctx)?.into_int_value();
  let goto_remaining_bytes =
    b.build_left_shift(goto_remaining, ctx.i32_type().const_int(4, false), "remaining_bytes");

  let goto_used_bytes = b.build_int_sub(goto_byte_size, goto_remaining_bytes, "goto_used_bytes");
  let goto_used_bytes_64 = b.build_int_cast(goto_used_bytes, ctx.i64_type(), "");

  let goto_top_ptr = CTX::goto_stack_ptr.load(module, parse_ctx)?.into_pointer_value();
  let goto_base_ptr_int = b.build_int_sub(
    b.build_ptr_to_int(goto_top_ptr, ctx.i64_type().into(), ""),
    goto_used_bytes_64,
    "goto_base",
  );

  let goto_base_ptr = b.build_int_to_ptr(
    goto_base_ptr_int,
    types.goto.ptr_type(inkwell::AddressSpace::Generic),
    "goto_base",
  );

  b.build_call(funct.free_stack, &[goto_base_ptr.into(), goto_total_bytes_64.into()], "");

  CTX::goto_stack_size.store(module, parse_ctx, i32.const_zero());

  b.build_unconditional_branch(empty_stack);

  b.position_at_end(empty_stack);

  b.build_return(None);

  if funct.internal_free_stack.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate internal_free_stack"))
  }
}

pub(crate) unsafe fn construct_extend_stack_if_needed(
  module: &LLVMParserModule,
) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, types, ctx, fun: funct, .. } = module;
  let i32 = ctx.i32_type();

  let fn_value = funct.extend_stack_if_needed;
  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let needed_slot_count = fn_value.get_nth_param(1).unwrap().into_int_value();

  b.position_at_end(ctx.append_basic_block(fn_value, "Entry"));

  // Compare the number of needed slots with the number of available slots
  let goto_remaining = CTX::goto_remaining.load(module, parse_ctx)?.into_int_value();

  let comparison =
    b.build_int_compare(inkwell::IntPredicate::ULT, goto_remaining, needed_slot_count, "");

  let extend_block = ctx.append_basic_block(fn_value, "Extend");
  let update_block = ctx.append_basic_block(fn_value, "UpdateStack");
  let return_block = ctx.append_basic_block(fn_value, "Return");

  b.build_conditional_branch(comparison, extend_block, return_block);

  // If the difference is less than the amount requested:
  b.position_at_end(extend_block);
  // Create a new stack, copy data from old stack to new one
  // and, if the old stack was not the original stack,
  // delete the old stack.

  let goto_slot_count = CTX::goto_stack_size.load(module, parse_ctx)?.into_int_value();

  let goto_byte_size =
    b.build_left_shift(goto_slot_count, ctx.i32_type().const_int(4, false), "goto_byte_size");
  let goto_remaining_bytes =
    b.build_left_shift(goto_remaining, ctx.i32_type().const_int(4, false), "remaining_bytes");

  let goto_used_bytes = b.build_int_sub(goto_byte_size, goto_remaining_bytes, "goto_used_bytes");
  let goto_used_bytes_64 = b.build_int_cast(goto_used_bytes, ctx.i64_type(), "");
  let goto_top_ptr = CTX::goto_stack_ptr.load(module, parse_ctx)?.into_pointer_value();

  let goto_base_ptr_int = b.build_int_sub(
    b.build_ptr_to_int(goto_top_ptr, ctx.i64_type().into(), ""),
    goto_used_bytes_64,
    "goto_base",
  );
  let goto_base_ptr = b.build_int_to_ptr(
    goto_base_ptr_int,
    types.goto.ptr_type(inkwell::AddressSpace::Generic),
    "goto_base",
  );

  // create a size that is equal to the needed amount rounded up to the nearest 64bytes
  let new_slot_count = b.build_int_add(goto_slot_count, needed_slot_count, "new_size");
  let new_slot_count = b.build_left_shift(new_slot_count, i32.const_int(2, false), "new_size");
  let new_slot_byte_size =
    b.build_left_shift(new_slot_count, ctx.i32_type().const_int(4, false), "total_bytes");
  let new_slot_byte_size_64 = b.build_int_cast(new_slot_byte_size, ctx.i64_type(), "");

  let new_ptr = b
    .build_call(funct.allocate_stack, &[new_slot_byte_size_64.into()], "")
    .try_as_basic_value()
    .unwrap_left()
    .into_pointer_value();

  b.build_call(
    funct.memcpy,
    &[
      b.build_bitcast(new_ptr, ctx.i8_type().ptr_type(inkwell::AddressSpace::Generic), "").into(),
      b.build_bitcast(goto_base_ptr, ctx.i8_type().ptr_type(inkwell::AddressSpace::Generic), "")
        .into(),
      goto_used_bytes.into(),
      ctx.bool_type().const_int(0, false).into(),
    ],
    "",
  );

  build_fast_call(module, funct.internal_free_stack, &[parse_ctx.into()]);

  b.build_unconditional_branch(update_block);

  b.position_at_end(update_block);

  let new_stack_top_ptr = b.build_ptr_to_int(new_ptr, ctx.i64_type(), "new_top");
  let new_stack_top_ptr = b.build_int_add(new_stack_top_ptr, goto_used_bytes_64, "new_top");
  let new_stack_top_ptr = b.build_int_to_ptr(
    new_stack_top_ptr,
    types.goto.ptr_type(inkwell::AddressSpace::Generic),
    "new_top",
  );

  CTX::goto_stack_ptr.store(module, parse_ctx, new_stack_top_ptr)?;
  CTX::goto_stack_size.store(module, parse_ctx, new_slot_count)?;

  let slot_diff = b.build_int_sub(new_slot_count, goto_slot_count, "slot_diff");
  let new_remaining_count = b.build_int_add(slot_diff, goto_remaining, "remaining");
  CTX::goto_remaining.store(module, parse_ctx, new_remaining_count);

  b.build_unconditional_branch(return_block);

  b.position_at_end(return_block);
  b.build_return(Some(&i32.const_int(1, false)));

  if funct.extend_stack_if_needed.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate extend_stack_if_needed"))
  }
}

pub(crate) unsafe fn construct_emit_shift(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, types, ctx, fun: funct, .. } = module;

  let i64 = ctx.i64_type();
  let i32 = ctx.i32_type();

  let fn_value = funct.emit_shift;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let basic_action = fn_value.get_nth_param(0)?.into_pointer_value();
  let anchor_offset = fn_value.get_nth_param(1)?.into_int_value();
  let token_offset = fn_value.get_nth_param(2)?.into_int_value();
  let token_length = fn_value.get_nth_param(3)?.into_int_value();
  let token_line_info = fn_value.get_nth_param(4)?.into_int_value();

  let token_action = ctx
    .struct_type(&[i32.into(), i32.into(), i64.into(), i64.into(), i64.into(), i64.into()], false);

  b.position_at_end(entry);

  let shift = b
    .build_bitcast(basic_action, token_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();
  let shift_struct = b.build_load(shift, "").into_struct_value();
  let shift_struct = b.build_insert_value(shift_struct, i32.const_int(ParseAction::des_Shift, false), 0, "")?;
  let shift_struct = b.build_insert_value(shift_struct, anchor_offset, 2, "")?;
  let shift_struct = b.build_insert_value(shift_struct, token_offset, 3, "")?;
  let shift_struct = b.build_insert_value(shift_struct, token_length, 4, "")?;
  let shift_struct = b.build_insert_value(shift_struct, token_line_info, 5, "")?;
  b.build_store(shift, shift_struct);

  // load the anchor token to be used as the skipped symbols
  b.build_return(None);

  if funct.emit_shift.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate emit_shift"))
  }
}

pub(crate) unsafe fn construct_emit_accept(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, ctx: c, fun: funct, .. } = module;

  let i32 = c.i32_type();

  let fn_value = funct.emit_accept;

  let accept_action = c.struct_type(&[i32.into(), i32.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = c.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0)?.into_pointer_value();
  let basic_action = fn_value.get_nth_param(1)?.into_pointer_value();

  b.position_at_end(entry);

  let production = CTX::production.get_ptr(module, parse_ctx)?;
  let production = b.build_load(production, "");
  let accept = b
    .build_bitcast(basic_action, accept_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let accept_struct = b.build_load(accept, "").into_struct_value();
  let accept_struct = b.build_insert_value(accept_struct, i32.const_int(7, false), 0, "")?;
  let accept_struct = b.build_insert_value(accept_struct, production, 2, "")?;

  b.build_store(accept, accept_struct);

  b.build_return(None);

  if funct.emit_accept.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not build emit_accept function"))
  }
}

pub(crate) unsafe fn construct_emit_error(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, types, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();

  let fn_value = funct.emit_error;

  let error_action = ctx.struct_type(&[i32.into(), types.token.into(), i32.into()], false);

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");
  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let basic_action = fn_value.get_nth_param(1).unwrap().into_pointer_value();

  b.position_at_end(entry);

  // load the anchor token as the error token

  //let error_token = b.build_struct_gep(parse_ctx, CTX_tok_anchor, "").unwrap();
  let error_token = types.token.get_undef();

  // load the last production value

  let production = CTX::production.get_ptr(module, parse_ctx)?;
  let production = b.build_load(production, "");

  // build the ParseAction::Error struct

  let error = b
    .build_bitcast(basic_action, error_action.ptr_type(inkwell::AddressSpace::Generic), "")
    .into_pointer_value();

  let error_struct = b.build_load(error, "").into_struct_value();
  let error_struct = b.build_insert_value(error_struct, i32.const_int(8, false), 0, "").unwrap();
  let error_struct = b.build_insert_value(error_struct, error_token, 1, "").unwrap();
  let error_struct = b.build_insert_value(error_struct, production, 2, "").unwrap();

  b.build_store(error, error_struct);

  b.build_return(None);

  if funct.emit_error.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not build emit_error function"))
  }
}

pub(crate) unsafe fn construct_init(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();

  let fn_value = funct.init;

  let parse_ctx = fn_value.get_first_param().unwrap().into_pointer_value();
  let reader_ptr = fn_value.get_last_param().unwrap().into_pointer_value();

  builder.position_at_end(ctx.append_basic_block(fn_value, "Entry"));

  CTX::reader.store(module, parse_ctx, reader_ptr)?;

  CTX::goto_stack_size.store(module, parse_ctx, i32.const_int(0, false))?;

  CTX::goto_remaining.store(module, parse_ctx, i32.const_int(0, false))?;

  CTX::state.store(module, parse_ctx, i32.const_int(NORMAL_STATE_FLAG_LLVM as u64, false))?;

  builder.build_return(None);

  if funct.init.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate init"))
  }
}

pub(crate) fn create_offset_label(offset: usize) -> String {
  format!("off_{:X}", offset)
}

pub(crate) unsafe fn construct_push_state_function(module: &LLVMParserModule) -> SherpaResult<()> {
  let LLVMParserModule { builder: b, types, ctx, fun: funct, .. } = module;

  let i32 = ctx.i32_type();

  let fn_value = funct.push_state;

  // Set the context's goto pointers to point to the goto block;
  let entry = ctx.append_basic_block(fn_value, "Entry");

  let parse_ctx = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let goto_state = fn_value.get_nth_param(1).unwrap().into_int_value();
  let goto_pointer = fn_value.get_nth_param(2).unwrap().into_pointer_value();

  b.position_at_end(entry);
  let new_goto = b.build_insert_value(types.goto.get_undef(), goto_state, 1, "").unwrap();
  let new_goto = b.build_insert_value(new_goto, goto_pointer, 0, "").unwrap();

  let goto_top_ptr = CTX::goto_stack_ptr.get_ptr(module, parse_ctx)?;
  let goto_top = b.build_load(goto_top_ptr, "").into_pointer_value();
  b.build_store(goto_top, new_goto);

  let goto_top = b.build_gep(goto_top, &[i32.const_int(1, false)], "");
  b.build_store(goto_top_ptr, goto_top);

  let goto_remaining_ptr = CTX::goto_remaining.get_ptr(module, parse_ctx)?;
  let goto_remaining = b.build_load(goto_remaining_ptr, "").into_int_value();
  let goto_remaining = b.build_int_sub(goto_remaining, i32.const_int(1, false), "");
  b.build_store(goto_remaining_ptr, goto_remaining);

  b.build_return(None);

  if funct.push_state.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate push_state"))
  }
}

pub(crate) unsafe fn construct_ast_builder(module: &LLVMParserModule) -> SherpaResult<()> {
  use inkwell::AddressSpace::*;
  let LLVMParserModule { ctx, types, builder: b, .. } = module;
  let LLVMTypes { parse_ctx, action, .. } = types;

  let i32 = ctx.i32_type();
  let i64 = ctx.i64_type();

  let ast_builder = module.fun.ast_builder;

  let REDUCE_STRUCT = ctx
    .struct_type(&[i32.into(), i32.into(), i32.into(), i32.into(), i32.into(), i32.into()], false);

  let parse_context = ast_builder.get_nth_param(0)?.into_pointer_value();
  let reducers = ast_builder.get_nth_param(1)?.into_pointer_value();
  let shift_handler = ast_builder.get_nth_param(2)?.into_pointer_value();
  let result_handler = ast_builder.get_nth_param(3)?.into_pointer_value();

  b.position_at_end(ctx.append_basic_block(ast_builder, "Preamble"));

  let parse_loop = ctx.append_basic_block(ast_builder, "ParseLoop");
  let shift = ctx.append_basic_block(ast_builder, "Shift");
  let shift_assign_base_pointer = ctx.append_basic_block(ast_builder, "AssignStackPointer");
  let shift_add_slot = ctx.append_basic_block(ast_builder, "AddSlot");
  let shift_new_object = ctx.append_basic_block(ast_builder, "ShiftNewObject");
  let reduce = ctx.append_basic_block(ast_builder, "Reduce");
  let default = ctx.append_basic_block(ast_builder, "Default");

  let stack_capacity_ptr = b.build_alloca(i32, "stack_capacity");
  b.build_store(stack_capacity_ptr, i32.const_zero());

  let stack_top_ptr = b.build_alloca(i32, "stack_top");
  b.build_store(stack_top_ptr, i32.const_zero());

  let action = b.build_alloca(*action, "action");
  let discriminant_ptr = b.build_struct_gep(action, 0, "discriminant")?;

  let ast_slot_slice_ptr = b.build_alloca(types.stack_struct, "slot_lookup_ptr"); // Stores the stack lookup structure
  let slot_ptr_ptr = b.build_alloca(types.ast_slot.ptr_type(Generic), "slot_ptr_ptr"); // Store the pointer to the bottom of the AST stack

  b.build_store(slot_ptr_ptr, types.ast_slot.ptr_type(Generic).const_null());

  b.build_unconditional_branch(parse_loop);

  // Parse Loop --------------------------------------------------------

  b.position_at_end(parse_loop);
  // Begin by calling the dispatch function.

  build_fast_call(module, module.fun.dispatch, &[parse_context.into(), action.into()])?;

  // Load the discriminant from the action.
  let discriminant = b.build_load(discriminant_ptr, "").into_int_value();

  b.build_switch(discriminant, default, &[
    (i32.const_int(ParseAction::des_Shift, false), shift),
    (i32.const_int(ParseAction::des_Reduce, false), reduce),
  ]);

  // SHIFT --------------------------------------------------------
  b.position_at_end(shift);

  let top = b.build_load(stack_top_ptr, "top").into_int_value();
  let capacity = b.build_load(stack_capacity_ptr, "capacity").into_int_value();

  let c = b.build_int_compare(inkwell::IntPredicate::UGE, top, capacity, "");
  b.build_conditional_branch(c, shift_add_slot, shift_new_object);
  // ADD SLOT --------------------------------------------------------
  b.position_at_end(shift_add_slot);

  // Need bottom and top pointer for the internally maintained stack.
  let slot_ptr = b.build_alloca(types.ast_slot, "stack");
  let capacity = b.build_int_add(capacity, i32.const_int(1, false), "");
  b.build_store(stack_capacity_ptr, capacity);

  // If the stack pointer is zero, assign the first slot address to this pointer.
  let stack_ptr = b.build_load(slot_ptr_ptr, "").into_pointer_value();
  let stack_ptr_val = b.build_ptr_to_int(stack_ptr, i64, "");
  let zero_ptr = types.ast_slot.ptr_type(Generic).const_null();
  let zero_ptr_val = b.build_ptr_to_int(zero_ptr, i64, "");

  let c = b.build_int_compare(inkwell::IntPredicate::EQ, stack_ptr_val, zero_ptr_val, "");
  b.build_conditional_branch(c, shift_assign_base_pointer, shift_new_object);

  b.position_at_end(shift_assign_base_pointer);

  b.build_store(slot_ptr_ptr, slot_ptr);

  b.build_unconditional_branch(shift_new_object);

  // SHIFT OBJECT --------------------------------------------------------
  b.position_at_end(shift_new_object);

  // Increment the top to look into the next slot.
  let top = b.build_int_add(top, i32.const_int(1, false), "");
  b.build_store(stack_top_ptr, top);

  // Call shift handler
  // Calculate the position of the empty object's first field
  let slot_ptr = build_stack_offset_ptr(
    module,
    b.build_load(slot_ptr_ptr, "slot").into_pointer_value(),
    b.build_load(stack_top_ptr, "top").into_int_value(),
  );
  // Store slot and symbol info in lookup structure
  let slot_lookup_entry_ptr = b.build_struct_gep(ast_slot_slice_ptr, 0, "")?;
  b.build_store(slot_lookup_entry_ptr, slot_ptr);
  let slot_lookup_size_ptr = b.build_struct_gep(ast_slot_slice_ptr, 1, "")?;
  b.build_store(slot_lookup_size_ptr, i32.const_int(1, false));
  
  b.build_call(
    CallableValue::try_from(shift_handler)?,
    &[parse_context.into(), action.into(), ast_slot_slice_ptr.into()],
    "",
  );
  b.build_unconditional_branch(parse_loop);
  // REDUCE --------------------------------------------------------
  b.position_at_end(reduce);
  // Get slice size
  let reduce_action = b
    .build_bitcast(action, REDUCE_STRUCT.ptr_type(Generic), "reduce_action_ptr")
    .into_pointer_value();

  let production_id_ptr = b.build_struct_gep(reduce_action, 2, "")?;
  let rule_id_ptr = b.build_struct_gep(reduce_action, 3, "")?;
  let symbol_count_ptr = b.build_struct_gep(reduce_action, 4, "")?;
  let symbol_count_original = b.build_load(symbol_count_ptr, "").into_int_value();

  // Calculate the position of the first element and the last element.
  let top = b.build_load(stack_top_ptr, "top").into_int_value();

  let symbol_count = b.build_int_sub(symbol_count_original, i32.const_int(1, false), "");
  let top = b.build_int_sub(top, symbol_count, "");
  b.build_store(stack_top_ptr, top);

  let bottom_slot_ptr = build_stack_offset_ptr(
    module,
    b.build_load(slot_ptr_ptr, "slot").into_pointer_value(),
    b.build_load(stack_top_ptr, "top").into_int_value(),
  );

  let rule_index = b.build_load(rule_id_ptr, "").into_int_value();

  // Load the parse function and pass the stack into it.
  let reducer = b.build_gep(reducers, &[rule_index.into()], "");
  let reducer = b.build_load(reducer, "").into_pointer_value();

  // Store slot and symbol info in lookup structure
  let slot_lookup_entry_ptr = b.build_struct_gep(ast_slot_slice_ptr, 0, "")?;
  b.build_store(slot_lookup_entry_ptr, bottom_slot_ptr);
  let slot_lookup_size_ptr = b.build_struct_gep(ast_slot_slice_ptr, 1, "")?;
  b.build_store(slot_lookup_size_ptr, symbol_count_original);

  b.build_call(
    CallableValue::try_from(reducer)?,
    &[symbol_count_original.into(), ast_slot_slice_ptr.into()],
    "",
  );

  b.build_unconditional_branch(parse_loop);

  // DEFAULT --------------------------------------------------------
  b.position_at_end(default);
  let top = b.build_load(stack_top_ptr, "top").into_int_value();
  
  // Store slot and symbol info in lookup structure
  let slot_lookup_entry_ptr = b.build_struct_gep(ast_slot_slice_ptr, 0, "")?;
  b.build_store(slot_lookup_entry_ptr, b.build_load(slot_ptr_ptr, "slot").into_pointer_value());
  let slot_lookup_size_ptr = b.build_struct_gep(ast_slot_slice_ptr, 1, "")?;
  b.build_store(slot_lookup_size_ptr, top);
  
  let return_value = b.build_call(
    CallableValue::try_from(result_handler)?,
    &[parse_context.into(), action.into(), ast_slot_slice_ptr.into()],
    "",
  );

  b.build_return(Some(&return_value.try_as_basic_value().unwrap_left().into_struct_value()));

  if ast_builder.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate ast_builder"))
  }
}

fn build_stack_offset_ptr<'a>(
  module: &'a LLVMParserModule,
  ast_stack_ptr: PointerValue<'a>,
  top: IntValue<'a>,
) -> PointerValue<'a> {
  let b = &module.builder;
  let i64 = module.ctx.i64_type();
  let i32 = module.ctx.i32_type();
  let ast_slot = module.types.ast_slot.size_of().unwrap();
  let top = b.build_int_sub(top, i32.const_int(1, false), "");
  let data_pointer_int = b.build_ptr_to_int(ast_stack_ptr, i64.into(), "");
  let top_64 = b.build_int_z_extend(top, i64, "");
  let top_64 = b.build_int_mul(top_64, ast_slot, "");
  let data_pointer_int = b.build_int_sub(data_pointer_int, top_64, "");
  let data_pointer = b.build_int_to_ptr(data_pointer_int, ast_stack_ptr.get_type(), "");
  data_pointer
}

pub(crate) fn construct_utf8_lookup_function(module: &LLVMParserModule) -> SherpaResult<()> {
  let i32 = module.ctx.i32_type();
  let i8 = module.ctx.i8_type();
  let zero = i32.const_int(0, false);
  let bool = module.ctx.bool_type();
  let b = &module.builder;
  let funct = &module.fun;
  let fn_value = funct.get_utf8_codepoint_info;

  let input_ptr = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let block_entry = module.ctx.append_basic_block(fn_value, "Entry");
  let block_return_ascii = module.ctx.append_basic_block(fn_value, "return_ascii");
  let block_build_code_point = module.ctx.append_basic_block(fn_value, "build_code_point");
  let block_4bytes = module.ctx.append_basic_block(fn_value, "_4bytes");
  let block_3bytes = module.ctx.append_basic_block(fn_value, "_3bytes");
  let block_2bytes = module.ctx.append_basic_block(fn_value, "_2bytes");
  let block_invalid = module.ctx.append_basic_block(fn_value, "invalid");
  b.position_at_end(block_entry);

  let codepoint_info =
    b.build_insert_value(module.types.cp_info.get_undef(), zero, 0, "cp_info").unwrap();
  let codepoint_info_base = b.build_insert_value(codepoint_info, zero, 1, "cp_info").unwrap();

  // Determine number of leading bits set

  let byte = b.build_load(input_ptr, "header_byte").into_int_value();
  let invert = b.build_xor(byte, i8.const_int(255, false), "inverted_header_byte");
  let bit_count = b
    .build_call(
      funct.ctlz_i8,
      &[invert.into(), bool.const_int(0, false).into()],
      "header_bit_count",
    )
    .try_as_basic_value()
    .unwrap_left()
    .into_int_value();

  let comparison =
    b.build_int_compare(inkwell::IntPredicate::EQ, bit_count, i8.const_int(0, false), "");

  b.build_conditional_branch(comparison, block_return_ascii, block_build_code_point);

  // --- Build ASCII Block
  b.position_at_end(block_return_ascii);
  // Insert the codepoint into the CP_INFO struct
  let codepoint_info =
    b.build_insert_value(codepoint_info_base, b.build_int_z_extend(byte, i32, ""), 0, "").unwrap();

  // Insert the codepoint byte length into the CP_INFO struct
  let codepoint_info =
    b.build_insert_value(codepoint_info, i32.const_int(1, false), 1, "").unwrap();

  b.build_return(Some(&codepoint_info));

  // --- Build CodePoint Block
  b.position_at_end(block_build_code_point);
  let off_ptr = b.build_alloca(i32, "offset_ptr");
  let val_ptr = b.build_alloca(i32, "val_ptr");

  let mask = b.build_right_shift(i8.const_int(255, false), bit_count, false, "mask");
  let base_val = b.build_and(byte, mask, "base_val");
  let val = b.build_int_z_extend(base_val, i32, "val_A");
  b.build_store(val_ptr, val);
  b.build_store(off_ptr, i32.const_int(1, false));
  b.build_switch(bit_count, block_invalid, &[
    (i8.const_int(2, false), block_2bytes),
    (i8.const_int(3, false), block_3bytes),
    (i8.const_int(4, false), block_4bytes),
  ]);

  // --- Build 4byte CP  block
  b.position_at_end(block_4bytes);
  let off = b.build_load(off_ptr, "off").into_int_value();
  let val = b.build_load(val_ptr, "val").into_int_value();
  let val = b.build_left_shift(val, i32.const_int(6, false), "val");
  let val = b
    .build_call(funct.merge_utf8_part, &[input_ptr.into(), val.into(), off.into()], "val")
    .try_as_basic_value()
    .unwrap_left()
    .into_int_value();
  let off = b.build_int_add(off, i32.const_int(1, false), "off");
  b.build_store(val_ptr, val);
  b.build_store(off_ptr, off);
  b.build_unconditional_branch(block_3bytes);

  // --- Build 3byte CP  block
  b.position_at_end(block_3bytes);
  let off = b.build_load(off_ptr, "off").into_int_value();
  let val = b.build_load(val_ptr, "val").into_int_value();
  let val = b.build_left_shift(val, i32.const_int(6, false), "val");
  let val = b
    .build_call(funct.merge_utf8_part, &[input_ptr.into(), val.into(), off.into()], "val")
    .try_as_basic_value()
    .unwrap_left()
    .into_int_value();
  let off = b.build_int_add(off, i32.const_int(1, false), "off");
  b.build_store(val_ptr, val);
  b.build_store(off_ptr, off);
  b.build_unconditional_branch(block_2bytes);

  // --- Build 2byte CP  block
  b.position_at_end(block_2bytes);
  let off = b.build_load(off_ptr, "off").into_int_value();
  let val = b.build_load(val_ptr, "val").into_int_value();
  let val = b.build_left_shift(val, i32.const_int(6, false), "val");
  let val = b
    .build_call(funct.merge_utf8_part, &[input_ptr.into(), val.into(), off.into()], "val")
    .try_as_basic_value()
    .unwrap_left()
    .into_int_value();

  let byte_length = b.build_int_z_extend(bit_count, i32, "");
  let codepoint_info = b.build_insert_value(codepoint_info_base, val, 0, "").unwrap();
  let codepoint_info = b.build_insert_value(codepoint_info, byte_length, 1, "").unwrap();

  b.build_return(Some(&codepoint_info));

  // --- Build Invalid Block
  b.position_at_end(block_invalid);

  b.build_return(Some(&codepoint_info_base));

  if funct.get_utf8_codepoint_info.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Err(SherpaError::from("\n\nCould not validate get_utf8_codepoint_info"))
  }
}

pub(crate) unsafe fn construct_merge_utf8_part_function(
  module: &LLVMParserModule,
) -> SherpaResult<()> {
  let i32 = module.ctx.i32_type();

  let b = &module.builder;
  let funct = &module.fun;
  let fn_value = funct.merge_utf8_part;

  let input_ptr = fn_value.get_nth_param(0).unwrap().into_pointer_value();
  let val = fn_value.get_nth_param(1).unwrap().into_int_value();
  let off = fn_value.get_nth_param(2).unwrap().into_int_value();

  let block_entry = module.ctx.append_basic_block(fn_value, "Entry");
  b.position_at_end(block_entry);

  let byte_ptr = b.build_gep(input_ptr, &[off], "byte_ptr");
  let byte = b.build_load(byte_ptr, "byte").into_int_value();

  let dword = b.build_int_z_extend(byte, i32, "dword");
  let dword = b.build_and(dword, i32.const_int(63, false), "dword");

  let cp = b.build_or(val, dword, "codepoint");

  b.build_return(Some(&cp));

  if fn_value.verify(true) {
    SherpaResult::Ok(())
  } else {
    SherpaResult::Ok(()) //SherpaResult::Err(SherpaError::from("\n\nCould not validate merge_utf8_part"))
  }
}

pub fn build_fast_call<'a, T>(
  module: &'a LLVMParserModule,
  callee_fun: T,
  args: &[BasicMetadataValueEnum<'a>],
) -> SherpaResult<CallSiteValue<'a>>
where
  T: Into<CallableValue<'a>>,
{
  let call_site = module.builder.build_call(callee_fun, args, "FAST_CALL_SITE");
  call_site.set_call_convention(fastCC);

  SherpaResult::Ok(call_site)
}

/// Compile a LLVM parser module from Hydrocarbon bytecode.
pub fn compile_from_bytecode<'a>(
  module_name: &str,
  g: &GrammarStore,
  llvm_context: &'a Context,
  output: &BytecodeOutput,
) -> SherpaResult<LLVMParserModule<'a>> {
  let mut llvm_module = construct_module(module_name, &llvm_context);
  let module = &mut llvm_module;

  unsafe {
    construct_init(module)?;
    construct_emit_accept(module)?;
    construct_emit_end_of_input(module)?;
    construct_emit_end_of_parse(module)?;
    construct_emit_reduce_function(module)?;
    construct_emit_shift(module)?;
    construct_dispatch_function(module)?;
    construct_get_adjusted_input_block_function(module)?;
    construct_push_state_function(module)?;
    construct_emit_error(module)?;
    construct_extend_stack_if_needed(module)?;
    construct_merge_utf8_part_function(module)?;
    construct_utf8_lookup_function(module)?;
    construct_scan(module)?;
    construct_next_function(module)?;
    construct_internal_free_stack(module)?;
    construct_drop(module)?;
    construct_parse_function(g, module, output)?;
  }

  llvm_module.fun.push_state.add_attribute(
    inkwell::attributes::AttributeLoc::Function,
    llvm_module.ctx.create_string_attribute("alwaysinline", ""),
  );

  SherpaResult::Ok(llvm_module)
}
