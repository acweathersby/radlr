use super::*;
use crate::types::Token;
use ::std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

/// Stores every error type that can be generated by a HCTK function. Also wraps common
/// error types.
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum HCError {
  //---------------------------------------------------------------------------
  // ----------------- Transition Errors --------------------------------------
  //---------------------------------------------------------------------------
  /// Error occurs when a scanner parse path cannot be made
  /// unambiguous due two Generic symbol types.
  Transition_InvalidGenerics {
    /// The root Symbols that whose combination lead to this error
    root_symbols: Vec<SymbolID>,
    /// The item chain, from the root item to
    /// the leaf, for each branch
    chains:       Vec<Vec<Item>>,
  },

  //---------------------------------------------------------------------------
  // ----------------- Grammar Load Errors ------------------------------------
  //---------------------------------------------------------------------------
  /// A path specified in one grammar file does map to a valid file
  Load_InvalidGrammarPath {
    path: PathBuf,
    reference_token: Option<Token>,
  },

  // An imported grammar path referenced in another grammar does not exist
  Load_InvalidDependency {
    requestor: PathBuf,
    path:      PathBuf,
    tok:       Token,
    err:       Option<Box<HCError>>,
  },
  //---------------------------------------------------------------------------
  // ----------------- Grammar Compile Errors -------------------------------------------
  //---------------------------------------------------------------------------
  GrammarCompile_Location {
    message: String,
    inline_message: String,
    loc: Token,
  },

  GrammarCompile_MultiLocation {
    message:   String,
    locations: Vec<HCError>,
  },
  //---------------------------------------------------------------------------
  // ----------------- Runtime Errors -------------------------------------------
  //---------------------------------------------------------------------------
  Runtime_ParseError {
    production: u32,
    tok:        Token,
    source:     Option<Arc<Vec<u8>>>,
  },
  //---------------------------------------------------------------------------
  // ----------------- Ir Error Types -----------------------------------------
  //---------------------------------------------------------------------------
  IRError_BadParse,
  IRError_NotParsed,

  //---------------------------------------------------------------------------
  // ----------------- Generic Error Types ------------------------------------
  //---------------------------------------------------------------------------
  UNDEFINED,
  IOError(String),
  Error(std::fmt::Error),
  Text(String),
  Many {
    message: String,
    errors:  Vec<HCError>,
  },
}

use HCError::*;

impl From<std::io::Error> for HCError {
  fn from(err: std::io::Error) -> Self {
    IOError(err.to_string())
  }
}

impl From<std::fmt::Error> for HCError {
  fn from(err: std::fmt::Error) -> Self {
    Self::Error(err)
  }
}

impl From<()> for HCError {
  fn from(err: ()) -> Self {
    UNDEFINED
  }
}

impl From<&str> for HCError {
  fn from(err: &str) -> Self {
    Text(err.to_string())
  }
}

impl From<String> for HCError {
  fn from(err: String) -> Self {
    Text(err)
  }
}

impl Display for HCError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UNDEFINED => f.write_str("An unknown error has occurred "),
      Transition_InvalidGenerics { .. } => f.write_str("Transition_InvalidGenerics Error"),
      Load_InvalidGrammarPath { .. } => f.write_str("Load_InvalidGrammarPath Error"),
      Load_InvalidDependency { path, requestor, tok, err } => f.write_fmt(format_args!(
        "The import grammar path [{}], referenced in [{}:{}], does not exist: \n{}",
        path.to_str().unwrap_or(""),
        requestor.to_str().unwrap_or(""),
        tok.clone().get_line() + 1,
        tok.blame(
          1,
          1,
          &err.as_ref().unwrap_or(&Box::new(HCError::UNDEFINED)).to_string(),
          BlameColor::Red
        )
      )),
      IOError(err_string) => f.write_fmt(format_args!("IO Error: {}", err_string)),
      Text(err_string) => f.write_str(&err_string),
      Self::Error(err) => err.fmt(f),
      IRError_NotParsed => f.write_str("IRNode has not been parsed"),
      IRError_BadParse => f.write_str("Errors occurred during while parsing IRNode code"),
      GrammarCompile_Location { message, inline_message, loc } => {
        f.write_fmt(format_args!("{}\n{}", message, loc.blame(1, 1, &inline_message, None),))
      }
      GrammarCompile_MultiLocation { message, locations } => f.write_fmt(format_args!(
        "{}\n{}",
        message,
        locations.iter().map(|s| format!("{}", s)).collect::<Vec<_>>().join("\n"),
      )),
      Runtime_ParseError { production, tok, source } => {
        let mut tok = tok.clone();
        if tok.is_empty() {
          tok = tok.to_length(1);
        }
        f.write_str(&tok.blame(0, 0, "Unexpected Token", None))
      }
      Many { message, errors } => f.write_fmt(format_args!(
        "{} \n-------------------\n {}",
        message,
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
      )),
    }
  }
}
