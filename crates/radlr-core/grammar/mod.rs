//!
//! Handles the integration of Grammars into a GrammarSoup

mod build_database;
mod build_grammar;
mod build_grammar_2;
mod compile;
pub(crate) mod utils;

pub(crate) use build_database::build_compile_db;
pub use build_grammar::remove_grammar_mut;
pub use compile::{compile_grammar_from_str, load_grammar};
