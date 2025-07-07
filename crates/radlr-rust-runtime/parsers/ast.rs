//! A parser that produce the tokens, including skipped tokens, of an input.

use crate::types::*;
use std::fmt::Debug;

pub trait Tk: Clone + Default + std::hash::Hash + Debug + std::ops::Add<Output = Self> {
  fn to_string(&self) -> String;
  fn trim(&self, start: usize, end: usize) -> Self;
  fn from_range(start: usize, end: usize, line_number: u32, line_offset: u32, id: u32, source: SharedSymbolBuffer) -> Self;
  fn from_slice(slice: &[Self]) -> Self;
  fn len(&self) -> usize;
}

pub trait Node<Token: Tk>: Default {}

impl<Token: Tk, T: Default> Node<Token> for T {}

pub type Reducer<Token, N> = fn(*mut [N], &[Token], Token) -> N;

impl Tk for Token {
  fn from_range(start: usize, end: usize, line_number: u32, line_offset: u32, _id: u32, source: SharedSymbolBuffer) -> Self {
    Token {
      inner: TokenRange {
        len:      (end - start) as u32,
        off:      start as u32,
        line_num: line_number,
        line_off: line_offset,
      },
      input: Some(source),
    }
  }

  fn from_slice(slice: &[Self]) -> Self {
    debug_assert!(slice.len() > 0);
    if slice.len() == 1 {
      slice[0].clone()
    } else {
      let start = &slice[0];
      let end = &slice[slice.len() - 1];
      Token {
        inner: TokenRange { len: end.inner.off - start.inner.off + end.inner.len, ..start.inner },
        input: start.input.clone(),
      }
    }
  }

  fn len(&self) -> usize {
    self.inner.len as usize
  }

  fn to_string(&self) -> String {
    self.slice(0, self.inner.len as i32)
  }

  fn trim(&self, start: usize, end: usize) -> Self {
    Token::trim(&self, start as u32, end as u32)
  }
}

pub trait AstDatabase<I: ParserInput>: ParserProducer<I> + Sized {
  /// Parse while attempting to recover from any errors encountered in the
  /// input.
  ///
  /// This extends the fork parser by allowing recovery methods to be applied
  /// when the base parser encounters input that prevent it from continuing. As
  /// several error recovery strategies are employed, the resulting parse
  /// forest may include several trees containing error corrections of varying
  /// quality. In this case, the tree containing the least number of error
  /// correction assumptions will be ordered in front of trees that employ more
  /// guesswork to recover parsing.
  fn build_ast<Token: Tk + Debug, N: Node<Token> + Debug, R: AsRef<[Reducer<Token, N>]>>(
    &self,
    input: &mut I,
    entry: EntryPoint,
    rules: R,
  ) -> Result<N, ParserError> {
    build_ast(input, entry, self, rules)
  }
}

impl<I: ParserInput, T: ParserProducer<I> + Sized> AstDatabase<I> for T {}

fn build_ast<I: ParserInput, DB: ParserProducer<I>, Token: Tk + Debug, N: Node<Token> + Debug, R: AsRef<[Reducer<Token, N>]>>(
  input: &mut I,
  entry: EntryPoint,
  db: &DB,
  rules: R,
) -> Result<N, ParserError> {
  use super::super::types::Token as TK;

  let mut tokens = vec![];
  let mut nodes = vec![];

  let mut parser = db.get_parser()?;

  let mut ctx = parser.init(entry)?;

  ctx.end_ptr = input.len();

  let input_data = input.get_owned_ref();

  while let Some(action) = parser.next(input, &mut ctx) {
    match action {
      ParseAction::Accept { nonterminal_id, final_offset, token_line_count, token_line_offset } => {
        return if final_offset != input.len() {
          let mut token = TK::from_vals(1, final_offset as u32, token_line_count, token_line_offset);

          token.set_source(input_data.clone());

          Err(ParserError::InputError {
            inline_message:   format!("Failed to read entire input {} {}", input.len(), final_offset),
            last_nonterminal: nonterminal_id,
            loc:              token,
            message:          "Failed to read entire input".to_string(),
          })
        } else if nonterminal_id != entry.nonterm_id {
          let mut token = TK::from_vals(1, final_offset as u32, token_line_count, token_line_offset);

          token.set_source(input_data.clone());

          Err(ParserError::InputError {
            inline_message:   "Top symbol did not match the target nonterminal".to_string(),
            last_nonterminal: nonterminal_id,
            loc:              token,
            message:          "CST is incorrect".to_string(),
          })
        } else {
          return Ok(nodes.pop().unwrap());
        };
      }
      ParseAction::Error {
        last_state,
        last_nonterminal,
        byte_length,
        byte_offset,
        token_line_count,
        token_line_offset,
      } => {
        println!("{byte_length} {byte_offset}");
        let mut token = TK::from_vals(byte_length, byte_offset, token_line_count, token_line_offset);
        token.set_source(input_data);

        if let Some(expected_tokens) = db.get_expected_tok_ids_at_state(last_state.address as u32) {
          let token_strings = expected_tokens
            .iter()
            .filter_map(|id| db.token_id_to_str(*id))
            .to_owned()
            .map(|d| format!("\"{}\"", d.replace("\"", "\\\"")))
            .collect::<Vec<_>>()
            .join(" | ");

          return Err(ParserError::InputError {
            inline_message:   if expected_tokens.len() > 1 {
              format!(
                "Expected one of  [ {token_strings} ] got [ \"{}\" ] instead",
                char::from_u32(input.codepoint(byte_offset as usize)).unwrap_or_default()
              )
            } else {
              format!(
                "Expected [ {token_strings} ] got [ \"{}\" ] instead",
                char::from_u32(input.codepoint(byte_offset as usize)).unwrap_or_default()
              )
            },
            last_nonterminal: last_nonterminal,
            loc:              token,
            message:          "Encountered an unexpected character".to_string(),
          });
        } else {
          return Err(ParserError::InputError {
            inline_message:   format!(
              "Did not expect to encounter character [ \"{}\" ] at this point",
              char::from_u32(input.codepoint(byte_offset as usize)).unwrap_or_default()
            ),
            last_nonterminal: last_nonterminal,
            loc:              token,
            message:          "Encountered an unexpected character".to_string(),
          });
        }
      }
      ParseAction::Fork { .. } => {
        panic!("No implementation of fork resolution is available")
      }
      ParseAction::Skip { .. } => {}
      ParseAction::Shift {
        byte_length: token_byte_length,
        byte_offset: token_byte_offset,
        token_id,
        token_line_count,
        token_line_offset,
        ..
      } => {
        let offset_start = token_byte_offset as usize;
        let offset_end = (token_byte_offset + token_byte_length) as usize;

        let token =
          Token::from_range(offset_start, offset_end, token_line_count, token_line_offset, token_id, input_data.clone());

        tokens.push(token.clone());
        nodes.push(N::default());
      }
      ParseAction::Reduce { rule_id: _rule_id, symbol_count, .. } => {
        let rule = &rules.as_ref()[_rule_id as usize];

        let nodes_start = nodes.len() - symbol_count as usize;
        let tok_start = tokens.len() - symbol_count as usize;

        let node_slice = nodes[nodes_start..].as_mut();
        let token_slice = tokens[tok_start..].as_ref();

        let non_term_token = Token::from_slice(token_slice);

        let new_node = rule(node_slice as *mut [N], token_slice, non_term_token.clone());

        nodes.drain(nodes_start..);
        nodes.push(new_node);
        tokens.drain(tok_start..);
        tokens.push(non_term_token);
      }
      _ => panic!("Unexpected Action!"),
    }
  }
  return Err(ParserError::Unexpected);
}
