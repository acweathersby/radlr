use crate::primitives::ast_node::HCObj;
use crate::primitives::ast_node::ReduceFunction;
use crate::primitives::token::Token;
use crate::runtime::buffer::ByteReader;
use crate::runtime::error::TokenError;
use crate::runtime::recognizer::iterator::ParseAction;
use crate::runtime::recognizer::iterator::ParseErrorCode;
use crate::runtime::recognizer::iterator::ParseIterator;
use std::fmt::Debug;
use std::str::FromStr;

pub fn complete<'b, I: ParseIterator<T>, T: 'b + ByteReader, Node: Debug>(
    iterator: &mut I,
    fns: &'static [ReduceFunction<Node>],
) -> Result<HCObj<Node>, ParseAction>
{
    let mut tokens: Vec<Token> = Vec::with_capacity(8);

    let mut nodes: Vec<HCObj<Node>> = Vec::with_capacity(8);

    let mut stack_pointer: usize = 0;

    let mut token_offset: usize = 0;

    let mut state: ParseAction = ParseAction::NONE {};

    let source = iterator.reader().get_source();

    let mut action_block = |action| match action {
        ParseAction::TOKEN { token: _ } => {
            state = action;
        }
        ParseAction::NONE {} => {}
        ParseAction::ERROR { .. } => {
            state = action;
        }
        ParseAction::FORK {} => {
            state = action;
        }
        ParseAction::ACCEPT {} => {
            state = action;
        }
        ParseAction::REDUCE {
            body,
            length,
            production: _,
        } => {
            let len = length as usize;

            let pos_a = &tokens[tokens.len() - len as usize];

            let pos_b = &tokens[tokens.len() - 1];

            let mut token = Token::from_range(pos_a, pos_b);

            token.set_source(source.clone());

            let root = tokens.len() - len;

            tokens[root] = token.clone();

            unsafe {
                tokens.set_len(root + 1);
            }

            fns[body as usize](&mut nodes, token.clone());

            stack_pointer = stack_pointer - len + 1;
        }
        ParseAction::SHIFT { token } => {
            let mut tok = Token::from_kernel_token(&token);

            tok.set_source(source.clone());

            let node = HCObj::TOKEN(tok.clone());

            nodes.push(node);

            tokens.push(tok.clone());

            token_offset += token.cp_length as usize;

            stack_pointer += 1;
        }
        ParseAction::SKIP {
            length,
            line: _,
            token_type: _,
        } => {
            token_offset += length as usize;
        }
    };

    let last_token = iterator.start(&mut action_block);

    match state {
        ParseAction::ACCEPT {} => Ok(nodes.remove(0)),
        ParseAction::ERROR { production, .. } => {
            let error = TokenError::new(
                production,
                last_token,
                Some(iterator.reader().get_source()),
            );

            println!("Last token: {} ", error.report());

            Err(state)
        }
        _ => Err(ParseAction::ERROR {
            error_code: ParseErrorCode::NORMAL,
            pointer:    0,
            production: 0,
        }),
    }
}
