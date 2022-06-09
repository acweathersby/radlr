//Global Constants
const STATE_INDEX_MASK: u32 = (1 << 24) - 1;
const _FAIL_STATE_MASK: u32 = 1 << 27;
const _NORMAL_STATE_MASK: u32 = 1 << 26;
const _GOTO_STATE_MASK: u32 = 1 << 25;
const _ALPHA_INCREMENT_STACK_POINTER_MASK: u32 = 1 << 0;
const _ALPHA_HAVE_DEFAULT_ACTION_MASK: u32 = 1 << 1;
const _PRODUCTION_SCOPE_POP_POINTER: u32 = 2;
const INSTRUCTION_POINTER_MASK: u32 = 0xFFFFFF;
const skipped_scan_prod: u16 = 9009;

const DEFAULT_PASS_INSTRUCTION: usize = 1;

const NORMAL_STATE_MASK: u32 = 1 << 26;
