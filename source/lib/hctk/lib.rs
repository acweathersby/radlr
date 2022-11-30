pub use hctk_compile::*;
pub use hctk_core::{types, *};

/// Compiles a parse module from a grammar string
/// #Example
/// ```
/// # use hctk::*;
///
/// mod parser {
///   compile_mod! {
///       "
/// @IGNORE g:sp
///
/// <> start > t:hello t:world  f:ast{ { t_HelloWorld } }
///
/// "
///   }
/// }
///
/// let input = "hello world";
/// let node = parser::Context::parse_default(&mut UTF8StringReader::new(&input));
///
/// dbg!(node);
/// ```
pub use hctk_proc::compile_mod as compile;

pub use hctk_core::types::{Token, UTF8StringReader as UTF8Reader, UTF8StringReader};
