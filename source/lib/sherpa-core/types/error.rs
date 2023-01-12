use super::*;
use crate::journal::Journal;
use ::std::fmt::Display;
use std::{
  fmt::Debug,
  path::PathBuf,
  string::{FromUtf16Error, FromUtf8Error},
  sync::Arc,
};

pub(crate) mod severity {

  #[bitmask_enum::bitmask]
  /// Severity types of SherpaErrors
  pub enum SherpaErrorSeverity {
    Hint     = 0b100,
    Warning  = 0b10,
    Critical = 0b1,
    None     = 0b0,
  }
}

pub use severity::SherpaErrorSeverity;

/// Stores every error type that can be generated by a sherpa function. Also wraps common
/// error types.
#[derive(Clone, Debug)]
pub enum SherpaError {
  SourceError {
    loc:        Token,
    path:       PathBuf,
    id:         &'static str,
    msg:        String,
    inline_msg: String,
  },
  //---------------------------------------------------------------------------
  // ----------------- Grammar Load Errors ------------------------------------
  //---------------------------------------------------------------------------
  /// A path specified in one grammar file does map to a valid file
  load_err_invalid_grammar_path {
    path: PathBuf,
    reference_token: Option<Token>,
  },

  // An imported grammar path referenced in another grammar does not exist
  load_err_invalid_dependency {
    requestor: PathBuf,
    path:      PathBuf,
    tok:       Token,
    err:       Option<Box<SherpaError>>,
  },
  //---------------------------------------------------------------------------
  // ----------------- Grammar Compile Errors ---------------------------------Look
  //---------------------------------------------------------------------------
  grammar_err {
    message: String,
    inline_message: String,
    loc: Token,
    path: PathBuf,
  },

  grammar_err_multi_location {
    message:   String,
    locations: Vec<SherpaError>,
  },
  //---------------------------------------------------------------------------
  // ----------------- Runtime Errors -----------------------------------------
  //---------------------------------------------------------------------------
  rt_err {
    production: u32,
    tok:        Token,
    source:     Option<Arc<Vec<u8>>>,
    path:       PathBuf,
  },

  //---------------------------------------------------------------------------
  // ----------------- Ir Error Types -----------------------------------------
  //---------------------------------------------------------------------------
  ir_err_bad_parse,
  ir_warn_not_parsed,

  //---------------------------------------------------------------------------
  // ----------------- Generic Error Types ------------------------------------
  //---------------------------------------------------------------------------
  UNDEFINED,
  IOError(String),
  Error(std::fmt::Error),
  Text(String),
  Many {
    message: String,
    errors:  Vec<SherpaError>,
  },

  ExtendedError(Arc<dyn ExtendedError>),
}

use SherpaError::*;

pub trait ExtendedError: Debug + Send + Sync {
  fn report(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

  fn severity(&self) -> SherpaErrorSeverity;

  /// Create an error report with full access to Journal data.
  fn rich_report(&self, _: &Journal, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Empty Error Message")
  }

  /// A concise name for this error. This may include formatting, white-space, and special characters.
  fn friendly_name(&self) -> &str;
}

impl From<Arc<dyn ExtendedError>> for SherpaError {
  fn from(value: Arc<dyn ExtendedError>) -> Self {
    Self::ExtendedError(value)
  }
}

impl SherpaError {
  pub fn get_severity(&self) -> SherpaErrorSeverity {
    match self {
      ExtendedError(err) => err.severity(),
      _ => SherpaErrorSeverity::Critical,
    }
  }

  /// Compares the friendly name of an error with a string,
  /// returning `true` if the two match.
  pub fn is(&self, friendly_name: &str) -> bool {
    match self {
      ExtendedError(err) => err.friendly_name() == friendly_name,
      _ => false,
    }
  }

  /// Todo
  pub fn is_critical(&self) -> bool {
    matches!(self.get_severity(), _Critical)
  }

  /// Todo
  pub fn is_hint(&self) -> bool {
    matches!(self.get_severity(), _Hint)
  }

  /// Todo
  pub fn is_warning(&self) -> bool {
    matches!(self.get_severity(), _Warning)
  }
}

impl From<std::io::Error> for SherpaError {
  fn from(err: std::io::Error) -> Self {
    IOError(err.to_string())
  }
}

impl From<std::fmt::Error> for SherpaError {
  fn from(err: std::fmt::Error) -> Self {
    Self::Error(err)
  }
}

impl From<()> for SherpaError {
  fn from(_: ()) -> Self {
    UNDEFINED
  }
}

impl From<&str> for SherpaError {
  fn from(err: &str) -> Self {
    Text(err.to_string())
  }
}

impl From<String> for SherpaError {
  fn from(err: String) -> Self {
    Text(err)
  }
}

impl From<FromUtf8Error> for SherpaError {
  fn from(err: FromUtf8Error) -> Self {
    Text(err.to_string())
  }
}

impl From<FromUtf16Error> for SherpaError {
  fn from(err: FromUtf16Error) -> Self {
    Text(err.to_string())
  }
}

impl Display for SherpaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SourceError { msg, id, inline_msg, loc, path } => f.write_fmt(format_args!(
        "\n{} Error [{}:{}]\n   {}\n{}",
        id,
        path.to_str().unwrap(),
        loc.loc_stub(),
        msg,
        loc.blame(1, 1, &inline_msg, BlameColor::RED),
      )),
      UNDEFINED => f.write_str("\nAn unknown error has occurred "),
      load_err_invalid_grammar_path { .. } => f.write_str("\nLoad_InvalidGrammarPath Error"),
      load_err_invalid_dependency { path, requestor, tok, err } => f.write_fmt(format_args!(
        "\n[{}:{}]\nThe import grammar path [{}] does not exist: \n{}",
        requestor.to_str().unwrap_or(""),
        tok.loc_stub(),
        path.to_str().unwrap_or(""),
        tok.blame(
          1,
          1,
          &err.as_ref().unwrap_or(&Box::new(SherpaError::UNDEFINED)).to_string().trim(),
          BlameColor::RED
        )
      )),
      ExtendedError(error) => error.report(f),
      IOError(err_string) => f.write_fmt(format_args!("\nIO Error: {}", err_string)),
      Text(err_string) => f.write_str(&err_string),
      Self::Error(error) => Display::fmt(error, f),
      ir_warn_not_parsed => f.write_str("\nIRNode has not been parsed"),
      ir_err_bad_parse => f.write_str("\nErrors occurred during while parsing IRNode code"),
      grammar_err { message, inline_message, loc, path } => f.write_fmt(format_args!(
        "\n[{}:{}]\n   {}\n{}",
        path.to_str().unwrap(),
        loc.loc_stub(),
        message,
        loc.blame(1, 1, &inline_message, BlameColor::RED),
      )),

      grammar_err_multi_location { message, locations } => f.write_fmt(format_args!(
        "\n{}\n{}",
        message,
        locations.iter().map(|s| format!("{}", s)).collect::<Vec<_>>().join("\n"),
      )),

      rt_err { path, tok, .. } => match path.to_str() {
        Some(path) => f.write_fmt(format_args!(
          "\n[{}:{}]\nUnexpected token [{}]\n{}",
          path,
          tok.loc_stub(),
          tok.to_string().replace("\n", ":nl").replace(" ", ":sp"),
          tok.blame(1, 1, "", BlameColor::RED)
        )),
        None => f.write_fmt(format_args!(
          "\nUnexpected token [{}]\n{}",
          tok.to_string(),
          tok.blame(1, 1, "", BlameColor::RED)
        )),
      },
      Many { message, errors } => f.write_fmt(format_args!(
        "\n{} \n-------------------\n {}",
        message,
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
      )),
    }
  }
}
#[derive(Default, Debug)]
pub struct ErrorGroups {
  pub hints:    Vec<SherpaError>,
  pub warnings: Vec<SherpaError>,
  pub critical: Vec<SherpaError>,
}

pub trait SherpaErrorContainer {
  /// Returns a sorted set of SherpaErrors represented by ErrorGroups
  fn get_errors_types(&self) -> ErrorGroups;

  fn get_critical(&self) -> Vec<SherpaError> {
    self.get_errors_types().critical
  }

  fn get_warnings(&self) -> Vec<SherpaError> {
    self.get_errors_types().warnings
  }

  fn get_hints(&self) -> Vec<SherpaError> {
    self.get_errors_types().hints
  }

  fn have_errors(&self) -> bool;

  fn have_critical(&self) -> bool;

  fn have_warnings(&self) -> bool;

  fn have_hints(&self) -> bool;
}

impl SherpaErrorContainer for Vec<SherpaError> {
  fn get_errors_types(&self) -> ErrorGroups {
    let mut groups = ErrorGroups { ..Default::default() };
    for error in self {
      match error.get_severity() {
        _Critical => groups.critical.push(error.clone()),
        _Warning => groups.warnings.push(error.clone()),
        _Hint => groups.critical.push(error.clone()),
      }
    }

    groups
  }

  fn have_errors(&self) -> bool {
    !self.is_empty()
  }

  fn have_critical(&self) -> bool {
    self.iter().any(|e| e.is_critical())
  }

  fn have_hints(&self) -> bool {
    self.iter().any(|e| e.is_hint())
  }

  fn have_warnings(&self) -> bool {
    self.iter().any(|e| e.is_warning())
  }
}

impl From<&Vec<SherpaError>> for ErrorGroups {
  fn from(vec: &Vec<SherpaError>) -> Self {
    vec.get_errors_types()
  }
}

pub trait SherpaErrorPrint {
  /// Prints errors to io::stdout stream
  fn debug_print(&self);
  /// Prints errors to io::stderr stream
  fn stderr_print(&self);
}

impl SherpaErrorPrint for Vec<SherpaError> {
  fn debug_print(&self) {
    for error in self {
      eprintln!("{}", error.to_string());
    }
  }

  fn stderr_print(&self) {
    for error in self {
      eprintln!("{}", error.to_string());
    }
  }
}
