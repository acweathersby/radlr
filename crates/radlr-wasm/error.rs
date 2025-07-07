use radlr_core::RadlrError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Default, Debug)]
pub enum ErrorOrigin {
  #[default]
  Grammar,
  ParserBuild,
  StatesCreation,
  Ir,
  BytecodeImport,
}

#[wasm_bindgen]
#[derive(Clone, Default, Debug)]
pub struct JSRadlrSourceError {
  pub line:         u32,
  pub col:          u32,
  pub len:          u32,
  pub start_offset: u32,
  pub end_offset:   u32,
  pub origin:       ErrorOrigin,
  message:          String,
}

#[wasm_bindgen]
impl JSRadlrSourceError {
  #[wasm_bindgen(getter)]
  pub fn message(&mut self) -> String {
    self.message.clone()
  }
}

#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct PositionedErrors {
  vec: Vec<JSRadlrSourceError>,
}

#[wasm_bindgen]
impl PositionedErrors {
  #[wasm_bindgen(getter)]
  pub fn length(&self) -> u32 {
    self.vec.len() as u32
  }

  pub fn get_error_at(&self, index: u32) -> Option<JSRadlrSourceError> {
    self.vec.get(index as usize).cloned()
  }
}

impl From<(&Vec<RadlrError>, ErrorOrigin)> for PositionedErrors {
  fn from((errors, origin): (&Vec<RadlrError>, ErrorOrigin)) -> Self {
    let mut out = PositionedErrors { vec: vec![] };
    out.extend(errors, origin);
    out
  }
}

impl From<(Vec<RadlrError>, ErrorOrigin)> for PositionedErrors {
  fn from((mut errors, origin): (Vec<RadlrError>, ErrorOrigin)) -> Self {
    let mut out = PositionedErrors { vec: vec![] };
    out.extend(&mut errors, origin);
    out
  }
}

impl From<(RadlrError, ErrorOrigin)> for PositionedErrors {
  fn from((err, origin): (RadlrError, ErrorOrigin)) -> Self {
    PositionedErrors { vec: convert_error(&err, origin) }
  }
}

impl From<(&Vec<&RadlrError>, ErrorOrigin)> for PositionedErrors {
  fn from((errors, origin): (&Vec<&RadlrError>, ErrorOrigin)) -> Self {
    let mut out = PositionedErrors { vec: vec![] };
    out.extend_from_refs(&errors.into_iter().map(|e| (*e).clone()).collect(), origin);
    out
  }
}

impl PositionedErrors {
  pub fn extend_from_refs(&mut self, errors: &Vec<RadlrError>, origin: ErrorOrigin) {
    self.vec.extend(errors.iter().map(|e| convert_error(e, origin)).flatten())
  }

  pub fn extend(&mut self, errors: &Vec<RadlrError>, origin: ErrorOrigin) {
    self.vec.extend(errors.iter().map(|e| convert_error(e, origin)).flatten())
  }
}

fn convert_error(err: &RadlrError, origin: ErrorOrigin) -> Vec<JSRadlrSourceError> {
  match err {
    RadlrError::SourcesError { sources, msg: base_message, .. } => sources
      .iter()
      .map(|(loc, _, msg)| {
        let range = loc.get_range();
        JSRadlrSourceError {
          col: range.start_column,
          line: range.start_line,
          len: loc.len() as u32,
          start_offset: loc.get_start() as u32,
          end_offset: loc.get_end() as u32,
          message: base_message.clone() + ":\n " + msg,
          origin,
        }
      })
      .collect(),

    RadlrError::SourceError { loc, msg, .. } => {
      let range = loc.get_range();
      vec![JSRadlrSourceError {
        col: range.start_column,
        line: range.start_line,
        len: loc.len() as u32,
        start_offset: loc.get_start() as u32,
        end_offset: loc.get_end() as u32,
        message: msg.clone(),
        origin,
      }]
    }
    RadlrError::StaticText(text) => {
      vec![JSRadlrSourceError { message: text.to_string(), ..Default::default() }]
    }
    RadlrError::Text(text) => {
      vec![JSRadlrSourceError { message: text.to_string(), ..Default::default() }]
    }
    RadlrError::Multi(errors) => errors.iter().map(|e| convert_error(e, origin)).flatten().collect(),
    RadlrError::PoisonError(..) => vec![JSRadlrSourceError { message: "Poison Error".into(), ..Default::default() }],
    RadlrError::IOError(..) => vec![JSRadlrSourceError { message: "Io Error".into(), ..Default::default() }],
    RadlrError::Error(err) => vec![JSRadlrSourceError { message: err.to_string(), ..Default::default() }],
    _ => unreachable!(),
  }
}
