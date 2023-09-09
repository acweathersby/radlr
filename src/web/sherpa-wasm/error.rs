use sherpa_core::SherpaError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct JSSherpaSourceError {
  pub line:         u32,
  pub col:          u32,
  pub len:          u32,
  pub start_offset: u32,
  pub end_offset:   u32,
  message:          String,
}

#[wasm_bindgen]
impl JSSherpaSourceError {
  #[wasm_bindgen(getter)]
  pub fn message(&mut self) -> String {
    self.message.clone()
  }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PositionedErrors {
  vec: Vec<JSSherpaSourceError>,
}

#[wasm_bindgen]
impl PositionedErrors {
  #[wasm_bindgen(getter)]
  pub fn length(&self) -> u32 {
    self.vec.len() as u32
  }

  pub fn get_error_at(&self, index: u32) -> Option<JSSherpaSourceError> {
    self.vec.get(index as usize).cloned()
  }
}

impl From<&Vec<SherpaError>> for PositionedErrors {
  fn from(errors: &Vec<SherpaError>) -> Self {
    let mut out = PositionedErrors { vec: vec![] };
    out.extend(errors);
    out
  }
}

impl From<&Vec<&SherpaError>> for PositionedErrors {
  fn from(errors: &Vec<&SherpaError>) -> Self {
    let mut out = PositionedErrors { vec: vec![] };
    out.extend_from_refs(errors);
    out
  }
}

impl PositionedErrors {
  pub fn extend_from_refs(&mut self, errors: &Vec<&SherpaError>) {
    self.vec.extend(errors.iter().map(|e| convert_error(*e)).flatten())
  }

  pub fn extend(&mut self, errors: &Vec<SherpaError>) {
    self.vec.extend(errors.iter().map(|e| convert_error(e)).flatten())
  }
}

fn convert_error(err: &SherpaError) -> Vec<JSSherpaSourceError> {
  match err {
    SherpaError::SourcesError { sources, msg: base_message, .. } => sources
      .iter()
      .map(|(loc, _, msg)| {
        let range = loc.get_range();
        JSSherpaSourceError {
          col:          range.start_column,
          line:         range.start_line,
          len:          loc.len() as u32,
          start_offset: loc.get_start() as u32,
          end_offset:   loc.get_end() as u32,
          message:      base_message.clone() + ":\n " + msg,
        }
      })
      .collect(),

    SherpaError::SourceError { loc, msg, .. } => {
      let range = loc.get_range();
      vec![JSSherpaSourceError {
        col:          range.start_column,
        line:         range.start_line,
        len:          loc.len() as u32,
        start_offset: loc.get_start() as u32,
        end_offset:   loc.get_end() as u32,
        message:      msg.clone(),
      }]
    }
    SherpaError::StaticText(text) => {
      vec![JSSherpaSourceError { message: text.to_string(), ..Default::default() }]
    }
    SherpaError::Text(text) => {
      vec![JSSherpaSourceError { message: text.to_string(), ..Default::default() }]
    }
    SherpaError::Multi(errors) => errors.iter().map(|e| convert_error(e)).flatten().collect(),
    SherpaError::PoisonError(..) => vec![JSSherpaSourceError { message: "Poison Error".into(), ..Default::default() }],
    SherpaError::IOError(..) => vec![JSSherpaSourceError { message: "Io Error".into(), ..Default::default() }],
    SherpaError::Error(err) => vec![JSSherpaSourceError { message: err.to_string(), ..Default::default() }],
  }
}
