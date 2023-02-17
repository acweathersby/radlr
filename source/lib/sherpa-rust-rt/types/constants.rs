use std::fmt::Debug;

// Global Constants
pub const STATE_ADDRESS_MASK: u32 = (1 << 24) - 1;

/// The portion of a GOTO instruction that contains the state offset.
/// Alias of [STATE_INDEX_MASK]
pub const GOTO_STATE_ADDRESS_MASK: u32 = STATE_ADDRESS_MASK;

/// The portion of an instruction that stores inline data.
/// Masks out the instruction header.
pub const INSTRUCTION_CONTENT_MASK: u32 = 0xFFF_FFFF;

/// The portion of a instruction that contains the instruction's
/// type.
pub const INSTRUCTION_HEADER_MASK: u32 = 0xF000_0000;

pub const SKIPPED_SCAN_PROD: u16 = 9009;

/// Mask the part of the state metadata that stores the
/// production id.
pub const PRODUCTION_META_MASK: u64 = 0xFFFFF;

/// Mask the part of the state metadata that stores the
/// production id.
pub const PRODUCTION_META_MASK_INVERT: u64 = !PRODUCTION_META_MASK;

// Bit mask for bytecode states that are active during failure
// recovery mode
pub const FAIL_STATE_FLAG: u32 = 1 << 27;

/// Bit mask for bytecode states that are active during normal parse
/// mode
pub const NORMAL_STATE_FLAG: u32 = 1 << 26;

pub const STATE_MODE_MASK: u32 = FAIL_STATE_FLAG | NORMAL_STATE_FLAG;

pub const PEEK_MODE_FLAG: u32 = 1 << 28;

/// This is the standard location of a `fail` instruction that is
/// present in all bytecode blocks produced by Hydrocarbon.
pub const DEFAULT_FAIL_INSTRUCTION_ADDRESS: u32 = 2;

/// This is the standard location of a `pass-through` instruction that
/// is present in all bytecode blocks produced by Hydrocarbon.
pub const DEFAULT_PASS_THROUGH_INSTRUCTION_ADDRESS: u32 = 0;

/// This is the standard location of a `pass` instruction that is
/// present in all bytecode blocks produced by Hydrocarbon.
pub const DEFAULT_PASS_INSTRUCTION_ADDRESS: u32 = 1;

/// The offset of the first state within any HC bytecode buffer.
pub const FIRST_STATE_ADDRESS: u32 = 6;

pub const TOKEN_ASSIGN_FLAG: u32 = 0x04000000;

pub const END_OF_INPUT_TOKEN_ID: u32 = 0x1;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionType {
  Pass       = 0,
  ShiftToken = 1,
  Goto       = 2,
  SetProd    = 3,
  Reduce     = 4,
  Token      = 5,
  ForkTo     = 6,
  ResetPeek  = 7,
  Pop        = 8,
  VectorBranch = 9,
  HashBranch = 10,
  SetFailState = 11,
  Skip       = 12,
  ShiftScanner = 13,
  PeekToken  = 14,
  Fail       = 15,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
/// Bytecode instruction constants
pub struct Instruction(pub u32, usize);

impl Instruction {
  pub const I00_PASS: u32 = 0;
  pub const I01_SHIFT_TOKEN: u32 = 1 << 28;
  pub const I02_GOTO: u32 = 2 << 28;
  pub const I03_SET_PROD: u32 = 3 << 28;
  pub const I04_REDUCE: u32 = 4 << 28;
  pub const I05_TOKEN: u32 = 5 << 28;
  pub const I05_TOKEN_ASSIGN: u32 = Instruction::I05_TOKEN | TOKEN_ASSIGN_FLAG;
  pub const I05_TOKEN_ASSIGN_SHIFT: u32 = Instruction::I05_TOKEN | 0x09000000;
  pub const I05_TOKEN_LENGTH: u32 = Instruction::I05_TOKEN | 0x08000000;
  pub const I06_FORK_TO: u32 = 6 << 28;
  pub const I07_PEEK_RESET: u32 = 7 << 28;
  pub const I08_POP: u32 = 8 << 28;
  pub const I09_VECTOR_BRANCH: u32 = 9 << 28;
  pub const I10_HASH_BRANCH: u32 = 10 << 28;
  pub const I11_SET_CATCH_STATE: u32 = 11 << 28;
  pub const I12_SKIP: u32 = 12 << 28;
  pub const I13_SHIFT_SCANNER: u32 = 13 << 28;
  pub const I14_PEEK_TOKEN: u32 = 14 << 28;
  pub const I15_FAIL: u32 = 15 << 28;
  pub const I15_FALL_THROUGH: u32 = 15 << 28 | 1;

  pub fn pass() -> Instruction {
    Instruction(0, 1)
  }

  pub fn fail() -> Instruction {
    Instruction(0, 2)
  }

  pub fn is_valid(&self) -> bool {
    self.1 > 0
  }

  pub fn invalid() -> Self {
    Instruction(0, 0)
  }

  pub fn from(bc: &[u32], address: usize) -> Self {
    if address > bc.len() {
      Self::invalid()
    } else {
      Instruction(bc[address], address)
    }
  }

  pub fn next(&self, bc: &[u32]) -> Self {
    if self.1 >= bc.len() - 1 {
      Self::invalid()
    } else {
      Instruction(bc[self.1 + 1], self.1 + 1)
    }
  }

  pub fn get_token_value(&self) -> u32 {
    debug_assert!(self.is_token());

    self.get_contents() & 0x00FF_FFFF
  }

  /// If this instruction is a GOTO, returns the first instruction of the target state
  /// Otherwise returns an invalid instruction.
  pub fn goto(&self, bc: &[u32]) -> Self {
    match self.to_type() {
      InstructionType::Goto => Self::from(bc, (self.0 & GOTO_STATE_ADDRESS_MASK) as usize),
      _ => Self::invalid(),
    }
  }

  /// IF this is a branching instruction (HASH or VECTOR), then returns the first instruction
  /// located within the default block. Otherwise returns an invalid instruction
  pub fn branch_default(&self, bc: &[u32]) -> Self {
    if self.is_hash_branch() || self.is_vector_branch() {
      Self::from(&bc, (bc[self.get_address() + 3] as usize) + self.get_address())
    } else {
      Self::invalid()
    }
  }

  pub fn get_address(&self) -> usize {
    self.1
  }

  pub fn get_value(&self) -> u32 {
    self.0
  }

  pub fn is_pass(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I00_PASS
  }

  pub fn is_token_shift(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I01_SHIFT_TOKEN
  }

  pub fn is_goto(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I02_GOTO
  }

  pub fn is_set_prod(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I03_SET_PROD
  }

  pub fn is_reduce(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I04_REDUCE
  }

  pub fn is_token(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I05_TOKEN
  }

  pub fn is_fork(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I06_FORK_TO
  }

  pub fn is_reset_peek(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I07_PEEK_RESET
  }

  pub fn is_pop(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I08_POP
  }

  pub fn is_vector_branch(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I09_VECTOR_BRANCH
  }

  pub fn is_hash_branch(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I10_HASH_BRANCH
  }

  pub fn is_set_fail_state(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I11_SET_CATCH_STATE
  }

  pub fn is_skip(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I12_SKIP
  }

  pub fn is_scanner_shift(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I13_SHIFT_SCANNER
  }

  pub fn is_assert_shift(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I14_PEEK_TOKEN
  }

  pub fn is_fail(&self) -> bool {
    (self.0 & INSTRUCTION_HEADER_MASK) == Self::I15_FAIL
  }

  pub fn get_contents(&self) -> u32 {
    self.0 & INSTRUCTION_CONTENT_MASK
  }

  pub fn get_type(&self) -> u32 {
    self.0 & INSTRUCTION_HEADER_MASK
  }

  pub fn to_type(&self) -> InstructionType {
    match self.0 & INSTRUCTION_HEADER_MASK {
      Self::I00_PASS => InstructionType::Pass,
      Self::I01_SHIFT_TOKEN => InstructionType::ShiftToken,
      Self::I02_GOTO => InstructionType::Goto,
      Self::I03_SET_PROD => InstructionType::SetProd,
      Self::I04_REDUCE => InstructionType::Reduce,
      Self::I05_TOKEN => InstructionType::Token,
      Self::I06_FORK_TO => InstructionType::ForkTo,
      Self::I07_PEEK_RESET => InstructionType::ResetPeek,
      Self::I08_POP => InstructionType::Pop,
      Self::I09_VECTOR_BRANCH => InstructionType::VectorBranch,
      Self::I10_HASH_BRANCH => InstructionType::HashBranch,
      Self::I11_SET_CATCH_STATE => InstructionType::SetFailState,
      Self::I12_SKIP => InstructionType::Skip,
      Self::I13_SHIFT_SCANNER => InstructionType::ShiftScanner,
      Self::I14_PEEK_TOKEN => InstructionType::PeekToken,
      Self::I15_FAIL => InstructionType::Fail,
      _ => InstructionType::Pass,
    }
  }

  pub fn to_str(&self) -> &str {
    match self.0 & INSTRUCTION_HEADER_MASK {
      Self::I00_PASS => "I00_PASS",
      Self::I01_SHIFT_TOKEN => "I01_SHIFT",
      Self::I02_GOTO => "I02_GOTO",
      Self::I03_SET_PROD => "I03_SET_PROD",
      Self::I04_REDUCE => "I04_REDUCE",
      Self::I05_TOKEN => "I05_TOKEN",
      Self::I06_FORK_TO => "I06_FORK_TO",
      Self::I07_PEEK_RESET => "I07_SCAN",
      Self::I08_POP => "I08_POP",
      Self::I09_VECTOR_BRANCH => "I09_VECTOR_BRANCH",
      Self::I10_HASH_BRANCH => "I10_HASH_BRANCH",
      Self::I11_SET_CATCH_STATE => "I11_SET_FAIL_STATE",
      Self::I12_SKIP => "I12_REPEAT",
      Self::I13_SHIFT_SCANNER => "I13_SHIFT_SCANNER",
      Self::I14_PEEK_TOKEN => "I14_ASSERT_SHIFT",
      Self::I15_FAIL => "I15_FAIL",
      _ => "Undefined",
    }
  }
}

impl Debug for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug_struc = f.debug_struct(&format!("Sherpa Instruction [{}]", self.to_str()));
    debug_struc.finish()
  }
}

#[non_exhaustive]

pub struct InputType;

impl InputType {
  pub const T01_PRODUCTION: u32 = 0;
  pub const T02_TOKEN: u32 = 1;
  pub const T03_CLASS: u32 = 2;
  pub const T04_CODEPOINT: u32 = 3;
  pub const T05_BYTE: u32 = 4;

  pub fn to_string(val: u32) -> &'static str {
    match val {
      Self::T01_PRODUCTION => "PRODUCTION",
      Self::T02_TOKEN => "TOKEN",
      Self::T03_CLASS => "CLASS",
      Self::T04_CODEPOINT => "CODEPOINT",
      Self::T05_BYTE => "BYTE",
      _ => "",
    }
  }
}

#[non_exhaustive]
pub struct LexerType;
impl LexerType {
  pub const ASSERT: u32 = 1;
  pub const PEEK: u32 = 2;
}

pub enum BranchSelector {
  Hash,
  Vector,
}

/// values - The set of keys used to select a branch to jump to.
/// branches - An vector of branch bytecode vectors.
pub type GetBranchSelector =
  fn(values: &[u32], max_span: u32, branches: &[Vec<u32>]) -> BranchSelector;

pub fn default_get_branch_selector(
  values: &[u32],
  max_span: u32,
  branches: &[Vec<u32>],
) -> BranchSelector {
  // Hash table limitations:
  // Max supported item value: 2046 with skip set to 2048
  // Max number of values: 1024 (maximum jump span)
  // Max instruction offset from table header 2042

  let total_instruction_length = branches.iter().map(|b| b.len()).sum::<usize>();

  let has_unsupported_value = values.iter().cloned().any(|v| v > 2046);

  if (max_span < 2) || total_instruction_length > 2042 || has_unsupported_value {
    BranchSelector::Vector
  } else {
    BranchSelector::Hash
  }
}
