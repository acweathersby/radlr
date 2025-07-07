#![allow(non_snake_case, unused)]
#[derive(Clone, Copy)]
/// Settings for configuring the type of parser Radlr will generate.
#[derive(Debug)]
#[repr(C)]
pub struct ParserConfig {
  /// When enable, recursive descent style s`Call` states will be generated
  pub ALLOW_CALLS: bool,
  /// When enable, LR style states may be produced. In general, this
  /// allows more advanced grammar constructs to be parsed, such
  /// as left recursive rules.
  ///
  /// When disabled, grammars with rules that require LR style parse states
  /// will be rejected, and relevant errors will be reported.
  pub ALLOW_LR: bool,
  /// When enabled, unrestricted lookahead states states will be generated
  ///
  /// When disabled, grammars with rules that require a lookahead that is
  ///  `k>1` will be rejected, and relevant errors will be reported.
  pub ALLOW_PEEKING: bool,
  /// Allow the parser to split its context to handle ambiguity. This
  /// may lead to a CSF (Concrete Syntax Forest) or a CSDAG (Concrete Syntax
  /// DAG) being returned by the parser instead of a CST
  pub ALLOW_CONTEXT_SPLITTING: bool,
  /// Creates a single scanner instead of multiple contextual scanners. More
  /// likely to report lexer conflicts.
  pub CONTEXT_FREE: bool,
  /// Creates states that directly handle transitions on terminals, allowing the
  /// creation of parsers that can patch existing CST data structures.
  pub AllOW_CST_MERGING: bool,
  /// Export all non-terminals as entry points in to the parser. This implies
  /// an RD or RAD parser.
  pub EXPORT_ALL_NONTERMS: bool,
  /// Allow the parser to shift on CST non-term nodes.
  pub ALLOW_CST_NONTERM_SHIFT: bool,
  /// Allow inlining of scanners that yield single codepoint tokens.
  ///
  /// Parsers created with this type of optimization tend to perform poorly when
  /// used for error correction.
  pub ALLOW_SCANNER_INLINING: bool,
  /// An anonymous non-terminal, aka grouped rules `e.g ( symA symB | symC | ..
  /// )`, may be inlined into the body of its host rule if none of the grouped
  /// rules contain semantic actions, such as `:ast` definitions.  
  ///
  /// Parsers created with this type of optimization tend to perform poorly when
  /// used for error correcting.
  pub ALLOW_ANONYMOUS_NONTERM_INLINING: bool,
  /// Enables using wide data types ( u16 | u32 | u64 | u128 ) to recognize a
  /// sequence of characters.
  pub ALLOW_BYTE_SEQUENCES: bool,
  /// Enables FOLLOW context sensitive scanners, which will consider the tokens
  /// that _follow_ the states which the scanner is constructing tokens
  /// for.
  ///
  /// May significantly increase the number scanner states.
  pub ALLOW_LOOKAHEAD_SCANNERS: bool,
  /// The maximum number of lookead symbols allowed before parser construction
  /// is aborted or a different disambiguating strategy is employed.
  pub max_k: u32,
}

pub struct OptimizeConfig {
  /// Enables using wide data types ( u16 | u32 | u64 | u128+ ) to recognize a
  /// sequence of bytes.
  pub ALLOW_BYTE_SEQUENCES: bool,
}

impl Default for ParserConfig {
  fn default() -> Self {
    Self {
      ALLOW_CALLS: true,
      ALLOW_LR: true,
      ALLOW_PEEKING: true,
      ALLOW_CONTEXT_SPLITTING: false,
      AllOW_CST_MERGING: false,
      CONTEXT_FREE: false,
      EXPORT_ALL_NONTERMS: false,
      ALLOW_CST_NONTERM_SHIFT: false,
      ALLOW_SCANNER_INLINING: true,
      ALLOW_ANONYMOUS_NONTERM_INLINING: true,
      ALLOW_BYTE_SEQUENCES: false,
      ALLOW_LOOKAHEAD_SCANNERS: false,
      max_k: u32::MAX,
    }
  }
}

impl ParserConfig {
  pub fn new() -> Self {
    Self::default().set_k(8).use_fork_states(false)
  }

  pub fn to_classification(&self) -> ParserClassification {
    ParserClassification {
      max_k:         self.max_k as u16,
      bottom_up:     self.ALLOW_LR,
      gotos_present: self.ALLOW_LR,
      calls_present: self.ALLOW_CALLS,
      peeks_present: self.ALLOW_PEEKING,
      forks_present: self.ALLOW_CONTEXT_SPLITTING,
    }
  }

  pub fn hybrid(self) -> Self {
    Self::new()
  }

  pub fn g_hybrid(self) -> Self {
    self.hybrid().set_k(8).use_fork_states(true)
  }

  pub fn cst_editor(mut self) -> Self {
    self = self.g_hybrid();
    self.EXPORT_ALL_NONTERMS = true;
    self.ALLOW_ANONYMOUS_NONTERM_INLINING = false;
    self.ALLOW_SCANNER_INLINING = false;
    //self.ALLOW_CST_NONTERM_SHIFT = true;
    self
  }

  pub fn g_recursive_descent_k(self) -> Self {
    self.recursive_descent_k(8).use_fork_states(true)
  }

  pub fn recursive_descent_k(mut self, k: u32) -> Self {
    self.ALLOW_CALLS = true;
    self.ALLOW_LR = false;
    self.set_k(k)
  }

  pub fn glr(mut self) -> Self {
    self = self.lrk(8);
    self.ALLOW_CONTEXT_SPLITTING = true;
    self
  }

  pub fn gll(mut self) -> Self {
    self = self.llk(8);
    self.ALLOW_CONTEXT_SPLITTING = true;
    self
  }

  pub fn lrk(mut self, k: u32) -> Self {
    self.ALLOW_CALLS = false;
    self.ALLOW_CONTEXT_SPLITTING = false;
    self.ALLOW_LR = true;
    self.set_k(k)
  }

  pub fn llk(mut self, k: u32) -> Self {
    self.ALLOW_CALLS = false;
    self.ALLOW_LR = false;
    self.ALLOW_CONTEXT_SPLITTING = false;
    self.set_k(k)
  }

  pub fn ll1(mut self) -> Self {
    self.ALLOW_CALLS = false;
    self.ALLOW_LR = false;
    self.ALLOW_PEEKING = false;
    self
  }

  pub fn set_k(mut self, k: u32) -> Self {
    self.ALLOW_PEEKING = k > 1;
    self.max_k = k;
    self
  }

  pub fn use_call_states(mut self, enable: bool) -> Self {
    self.ALLOW_CALLS = enable;
    self
  }

  pub fn use_fork_states(mut self, enable: bool) -> Self {
    self.ALLOW_CONTEXT_SPLITTING = enable;
    self
  }

  pub fn force_context_free(mut self, enable: bool) -> Self {
    self.CONTEXT_FREE = enable;
    self
  }

  /// Adds FOLLOW aware scanning behavior. May significantly increase the
  /// number of scanner states in more complex grammars.
  pub fn use_lookahead_scanners(mut self, enable: bool) -> Self {
    self.ALLOW_LOOKAHEAD_SCANNERS = enable;
    self
  }
}

/// Used to track the type of parser that has been created by radlr.
#[derive(Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct ParserClassification {
  /// Maximum peek level used to disambiguate conflicting phrases. If this is
  /// equal to `u16::MAX`, then peeking failed or a fork was used in its place.
  pub max_k:         u16,
  ///
  pub bottom_up:     bool,
  /// If set to true then the parser has at least one state that transitions on
  /// non-terminals as well terminals.
  pub gotos_present: bool,
  /// If set to true, then the parser has at least one state that jumps to the
  /// head state of a specific non-terminal
  pub calls_present: bool,
  /// If set to true, the parser has at least one state that performs k>1
  /// lookaheads before selecting an appropriate alternative action.
  pub peeks_present: bool,
  /// If set to true, the parser has at least one state that forks the parse
  /// tree, and performs parsing on separate alternatives in parallel
  pub forks_present: bool,
}

impl std::ops::BitOr for ParserClassification {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    Self {
      max_k:         self.max_k.max(rhs.max_k),
      bottom_up:     self.bottom_up | rhs.bottom_up,
      gotos_present: self.gotos_present | rhs.gotos_present,
      calls_present: self.calls_present | rhs.calls_present,
      peeks_present: self.peeks_present | rhs.peeks_present,
      forks_present: self.forks_present | rhs.forks_present,
    }
  }
}

impl std::ops::BitOr for &ParserClassification {
  type Output = ParserClassification;

  fn bitor(self, rhs: Self) -> Self::Output {
    *self | *rhs
  }
}

impl std::ops::BitOrAssign for ParserClassification {
  fn bitor_assign(&mut self, rhs: Self) {
    *self = *self | rhs
  }
}

impl std::ops::Add for ParserClassification {
  type Output = ParserClassification;

  fn add(self, rhs: Self) -> Self::Output {
    self | rhs
  }
}

impl std::ops::Add for &ParserClassification {
  type Output = ParserClassification;

  fn add(self, rhs: Self) -> Self::Output {
    *self | *rhs
  }
}

impl ParserClassification {
  /// Returns the classification as algorithm acronym string.
  ///
  /// This can be one of `LL | LR | RD | RAD | GLL | GLR | GRD | GRAD`.
  ///
  /// The string may also be postfixed with the maximum level of token
  /// lookahead, k, required to parse an input.
  ///
  /// # Example
  ///
  /// `RAD(2)` - Recursive Ascent & Descent with 2 levels of look ahead.
  pub fn to_string(&self) -> String {
    let base = if self.calls_present {
      if self.bottom_up || self.gotos_present {
        "RAD"
      } else {
        "RD"
      }
    } else {
      if self.gotos_present || self.bottom_up {
        "LR"
      } else {
        "LL"
      }
    };

    let g = if self.forks_present { "G" } else { "" };

    let k = if !self.forks_present {
      if self.max_k > 64 {
        "(*)".to_string()
      } else {
        "(".to_string() + &self.max_k.to_string() + ")"
      }
    } else {
      Default::default()
    };

    g.to_string() + base + &k
  }
}

#[derive(Default, Clone, Copy)]
pub struct ParserMetrics {
  pub classification: ParserClassification,
  pub num_of_states:  usize,
  pub optimized:      bool,
}
