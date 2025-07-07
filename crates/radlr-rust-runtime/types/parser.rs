use bytecode::MatchInputType;

use super::*;
use crate::parsers::Parser;

pub trait ParserInitializer {
  /// Creates a new parse context configured to recognize and produce parse
  /// actions for the given non-terminal id
  fn init(&mut self, entry: EntryPoint) -> Result<ParserContext, ParserError>;

  /// Resets the parser and prepares it to accept in input that should yield the
  /// given non-terminal. A parser should be initialized before attempting to
  /// iterate.
  ///
  /// An error is returned if `nonterminal_goal_id` is an invalid entry point
  /// of the parser.
  fn init_range(&mut self, nonterminal_goal_id: u32, start: usize, end: usize) -> Result<ParserContext, ParserError>;

  fn get_debugger(&mut self) -> &mut Option<Box<DebugFnNew>>;

  fn set_debugger(&mut self, debugger: Option<Box<DebugFnNew>>);
}

#[derive(Clone, Copy)]
pub struct SuccessorState {
  pub edge_mode:  MatchInputType,
  pub edge_value: u32,
  pub skipped:    bool,
  pub state_info: StateInfo,
}

pub trait ParserIterator<T: ParserInput> {
  /// Given a parse context, returns the next ParseAction for that context,
  /// mutating the context as needed.
  ///
  /// Returns None if the context has already entered a finished state
  fn next<'ctx>(&mut self, input: &mut T, context: &'ctx mut ParserContext) -> Option<ParseAction>;

  fn get_success_states<'ctx>(&mut self, _address: StateInfo) -> Vec<SuccessorState> {
    vec![]
  }

  fn get_token_id<'ctx>(&mut self, _address: StateInfo, _input: &mut T, _ctx: &mut ParserContext) -> (u32, bool) {
    (0, false)
  }

  fn is_token_branch_state<'ctx>(&mut self, _address: StateInfo) -> bool {
    false
  }
}

pub type TokenData = u32;

/// Provides information on artifacts consumed and produced by a specific parser
pub trait RuntimeDatabase {
  fn get_entry_data_from_name(&self, entry_name: &str) -> Result<EntryPoint, ParserError>;

  /// Return the normal token ids (those that are not skipped) expected at the
  /// given state.
  #[allow(unused)]
  fn get_expected_tok_ids_at_state(&self, state_id: u32) -> Option<&[TokenData]> {
    None
  }

  #[allow(unused)]
  fn get_nonterminal_name_from_id(&self, nt_id: u32) -> Option<String> {
    None
  }

  /// Returns a human friendly string representation of the given token id.
  fn token_id_to_str<'str>(&'str self, _id: u32) -> Option<&'str str> {
    None
  }

  ///// Returns a human friendly string representation of the given token id.
  //fn state_id_to_ptr<'str>(&'str self, id: u32) -> Option<&'str str>;

  /// Returns the parser configuration.
  fn get_config(&self) -> usize {
    0
  }

  /// The main entry point for the root grammar. This is typically the
  /// Nonterminal declared in the first `EXPORT` statement of the root
  /// grammar, or the first Nonterminal declared in the root grammar.
  fn default_entrypoint(&self) -> EntryPoint;

  /// Returns a list of enterable nonterminals and their respective entry point.
  fn entrypoints(&self) -> Vec<(String, u32)>;
}

/// An object capable of instantiating a parser
pub trait ParserProducer<T: ParserInput>: RuntimeDatabase {
  /// Creates a new parser or `Err` if there is an issue creating the parser.
  fn get_parser(&self) -> Result<Box<dyn Parser<T>>, ParserError> {
    Err(ParserError::NoData)
  }
}

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum StateType {
  #[default]
  Normal,
  ErrorRecovery,
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct UniqueParseState {
  pub state_id: u16,
  pub address:  u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StateInfo {
  pub stack_address:  u16,
  pub state_type:     StateType,
  /// True if the address is the first entry in a state block.
  pub is_state_entry: bool,
  pub state_id:       u32,
}

/// An identifier for the first instruction of a particular state.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ParserState {
  pub address: usize,
  pub info:    StateInfo,
}

impl ParserState {
  /// This entry is created when a parser is first initialized with a start
  /// state, or when a GOTO state is pushed to the state stack.
  pub fn state_entry(address: usize) -> Self {
    Self {
      address,
      info: StateInfo {
        state_id:       address as u32,
        stack_address:  0,
        state_type:     StateType::Normal,
        is_state_entry: true,
      },
    }
  }

  /// This entry is created when an action is emitted within a given state,
  /// forcing the parser to pause in-order to allow the parser host to handle
  /// the action.
  ///
  /// Such actions include SHIFTS, REDUCES, FORKS, AND ERRORS.
  pub fn yield_entry(address: usize) -> Self {
    Self {
      address,
      info: StateInfo {
        state_id:       address as u32,
        stack_address:  0,
        state_type:     StateType::Normal,
        is_state_entry: false,
      },
    }
  }

  pub fn get_address(&self) -> usize {
    self.address as usize
  }

  pub fn get_info(&self) -> StateInfo {
    self.info
  }
}
