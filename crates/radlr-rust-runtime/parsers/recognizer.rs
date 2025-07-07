//! A parser that produce the tokens, including skipped tokens, of an input.
use crate::types::*;

/// A simple reference parser that determines whether a givin input is a valid
/// string or not.
pub trait Recognizer<T: ParserInput>: ParserIterator<T> + ParserInitializer {
  /// Attempts to recognize the given input in its entirerty
  fn recognize(&mut self, input: &mut T, entry: EntryPoint) -> Result<(), ParserError> {
    let mut ctx = self.init(entry)?;

    while let Some(action) = self.next(input, &mut ctx) {
      match action {
        ParseAction::Accept { nonterminal_id, final_offset, .. } => {
          return if final_offset != input.len() {
            Err(ParserError::InputError {
              inline_message:   format!(
                "\nFailed to read entire input \"{}\" \n     end pos: {} \n     expected end pos: {}",
                input.string_range(0..input.len()),
                final_offset,
                input.len(),
              ),
              last_nonterminal: nonterminal_id,
              loc:              Default::default(),
              message:          "Failed to read entire input".to_string(),
            })
          } else if nonterminal_id != entry.nonterm_id {
            Err(ParserError::InputError {
              inline_message:   "Top symbol did not match the target nonterminal".to_string(),
              last_nonterminal: nonterminal_id,
              loc:              Default::default(),
              message:          "CST is incorrect".to_string(),
            })
          } else {
            Ok(())
          };
        }
        #[allow(unused)]
        ParseAction::Shift {
          byte_length: token_byte_length,
          byte_offset: token_byte_offset,
          token_id,
          ..
        } =>
        {
          #[cfg(debug_assertions)]
          if let Some(debug) = self.get_debugger() {
            let offset_start = token_byte_offset as usize;
            let offset_end = (token_byte_offset + token_byte_length) as usize;
            debug(&DebugEventNew::ActionShift { offset_start, offset_end, token_id }, ParserStackTrackers::from(&ctx), input);
          }
        }

        ParseAction::Reduce { rule_id: _rule_id, .. } =>
        {
          #[cfg(debug_assertions)]
          if let Some(debug) = self.get_debugger() {
            debug(&DebugEventNew::ActionReduce { rule_id: _rule_id }, ParserStackTrackers::from(&ctx), input);
          }
        }

        ParseAction::Error { last_nonterminal, .. } => {
          let last_input = TokenRange {
            len:      ctx.tok_byte_len as u32,
            off:      ctx.sym_ptr as u32,
            line_num: 0,
            line_off: 0,
          };
          let token: Token = last_input.to_token_from_ref(input.get_owned_ref());
          return Err(ParserError::InputError {
            message: "Could not recognize the following input:".to_string(),
            inline_message: "".to_string(),
            loc: token,
            last_nonterminal,
          });
        }
        _ => {}
      }
    }

    Err(ParserError::Unexpected)
  }
}

impl<T: ParserIterator<I> + ParserInitializer, I: ParserInput> Recognizer<I> for T {}
