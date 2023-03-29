//!
//! Handles the integration of Grammars into a GrammarSoup

mod build_db;
mod build_grammar;
mod compile;
mod utils;

pub use build_db::build_compile_db;
pub use build_grammar::parse_grammar;
pub use compile::compile_grammars_from_path;
