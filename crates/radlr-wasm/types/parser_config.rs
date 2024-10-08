//! More or less a direct translation of 'radlre_core::{ParserClassification,
//! ParserConfig, ParserMetrics}'.

#![allow(non_snake_case, unused)]
use std::ptr::slice_from_raw_parts;

use js_sys::{ArrayBuffer, Uint8Array};
use radlr_core::{ParserClassification, ParserConfig, ParserMetrics};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::preserve;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct JSParserConfig {
  /// When enable, recursive descent style `Call` states will be generated
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
  /// likely to report terminal conflicts.
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

impl From<ParserConfig> for JSParserConfig {
  fn from(value: ParserConfig) -> Self {
    unsafe { std::mem::transmute_copy(&value) }
  }
}

impl From<JSParserConfig> for ParserConfig {
  fn from(value: JSParserConfig) -> Self {
    unsafe { std::mem::transmute_copy(&value) }
  }
}

pub struct OptimizeConfig {
  /// Enables using wide data types ( u16 | u32 | u64 | u128+ ) to recognize a
  /// sequence of bytes.
  pub ALLOW_BYTE_SEQUENCES: bool,
}

#[wasm_bindgen]
impl JSParserConfig {
  pub fn export(&self) -> JsValue {
    serde_wasm_bindgen::to_value(self).expect("Could not serialize JSParserConfig")
  }

  pub fn import(value: JsValue) -> JSParserConfig {
    serde_wasm_bindgen::from_value(value).expect("Could not deserialize JSParserConfig")
  }

  pub fn duplicate(&self) -> Self {
    *self
  }

  pub fn size() -> u32 {
    size_of::<ParserConfig>() as u32
  }

  pub fn serialize(&self) -> ArrayBuffer {
    let config = self.native_config();
    let buffer = vec![config];
    let array_buffer = ArrayBuffer::new(size_of::<ParserConfig>() as u32);
    let output = Uint8Array::new(&array_buffer);
    output.copy_from(unsafe { &*slice_from_raw_parts(buffer.as_ptr() as *const u8, size_of::<ParserConfig>()) });
    array_buffer
  }

  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    ParserConfig::new().into()
  }

  fn native_config(&self) -> ParserConfig {
    ParserConfig::from(*self)
  }

  pub fn to_classification(&self) -> JSParserClassification {
    self.native_config().to_classification().into()
  }

  pub fn lab_default(self) -> Self {
    let mut config = self.native_config();
    config.ALLOW_CALLS = false;
    config.ALLOW_LR = true;
    config.ALLOW_PEEKING = true;
    config.ALLOW_CONTEXT_SPLITTING = false;
    config.CONTEXT_FREE = false;
    config.AllOW_CST_MERGING = false;
    config.EXPORT_ALL_NONTERMS = false;
    config.ALLOW_CST_NONTERM_SHIFT = false;
    config.ALLOW_SCANNER_INLINING = true;
    config.ALLOW_ANONYMOUS_NONTERM_INLINING = false;
    config.ALLOW_BYTE_SEQUENCES = true;
    config.ALLOW_LOOKAHEAD_SCANNERS = false;
    config.max_k = 8;

    config.into()
  }

  pub fn hybrid(self) -> Self {
    self.native_config().hybrid().into()
  }

  pub fn g_hybrid(self) -> Self {
    self.native_config().g_hybrid().into()
  }

  pub fn cst_editor(self) -> Self {
    self.native_config().cst_editor().into()
  }

  pub fn g_recursive_descent_k(self) -> Self {
    self.recursive_descent_k(8).use_fork_states(true)
  }

  pub fn recursive_descent_k(mut self, k: u32) -> Self {
    self.native_config().recursive_descent_k(k).into()
  }

  pub fn glr(mut self) -> Self {
    self.native_config().glr().into()
  }

  pub fn gll(mut self) -> Self {
    self.native_config().gll().into()
  }

  pub fn lrk(mut self, k: u32) -> Self {
    self.native_config().lrk(k).into()
  }

  pub fn llk(mut self, k: u32) -> Self {
    self.native_config().llk(k).into()
  }

  pub fn ll1(mut self) -> Self {
    self.native_config().ll1().into()
  }

  pub fn set_k(mut self, k: u32) -> Self {
    self.native_config().set_k(k).into()
  }

  pub fn use_call_states(mut self, enable: bool) -> Self {
    self.native_config().use_call_states(enable).into()
  }

  pub fn use_fork_states(mut self, enable: bool) -> Self {
    self.native_config().use_fork_states(enable).into()
  }

  pub fn force_context_free(mut self, enable: bool) -> Self {
    self.native_config().force_context_free(enable).into()
  }

  /// Adds FOLLOW aware scanning behavior. May significantly increase the
  /// number of scanner states in more complex grammars.
  pub fn use_lookahead_scanners(mut self, enable: bool) -> Self {
    self.native_config().use_lookahead_scanners(enable).into()
  }
}

/// Used to track the type of parser that has been created by radlr.
#[derive(Default, Clone, Copy, Debug)]
#[wasm_bindgen]
pub struct JSParserClassification(ParserClassification);

impl From<ParserClassification> for JSParserClassification {
  fn from(value: ParserClassification) -> Self {
    unsafe { std::mem::transmute_copy(&value) }
  }
}

impl From<JSParserClassification> for ParserClassification {
  fn from(value: JSParserClassification) -> Self {
    unsafe { std::mem::transmute_copy(&value) }
  }
}

#[wasm_bindgen]
impl JSParserClassification {
  /// Returns the parser classification as an algorithm acronym string.
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
    self.0.to_string()
  }

  pub fn import(&self) -> String {
    ParserClassification::from(*self).to_string()
  }

  pub fn deserialize(buffer: Uint8Array) -> JSParserClassification {
    let buffer = buffer.to_vec();

    let mut classification = ParserClassification::default();

    unsafe {
      std::ptr::copy(buffer.as_ptr(), std::mem::transmute(&mut classification), size_of::<ParserClassification>());
    };

    JSParserClassification(classification)
  }
}
