use crate::grammar::JSBytecodeParserDB;
use radlr_core::parser;
use radlr_rust_runtime::{
  parsers::{error_recovery::ErrorRecoveringDatabase, Parser},
  types::{bytecode::Opcode, *},
};
use serde::{Deserialize, Serialize};
use web_sys::console;

use std::{
  collections::VecDeque,
  hash::DefaultHasher,
  rc::Rc,
  sync::{LockResult, RwLock},
};

use wasm_bindgen::prelude::*;

/// Am iterable parser
#[wasm_bindgen]
pub struct JSByteCodeParser {
  running:         bool,
  db:              Rc<BytecodeParserDB>,
  reader:          StringInput,
  bytecode_parser: Box<dyn Parser<StringInput>>,
  ctx:             ParserContext,
  values:          Rc<RwLock<VecDeque<JSDebugPacket>>>,
}

#[wasm_bindgen]
impl JSByteCodeParser {
  pub fn new(input: String, bytecode: &JSBytecodeParserDB) -> Self {
    Self {
      running:         false,
      reader:          StringInput::from(input),
      bytecode_parser: bytecode.0.get_parser().unwrap(),
      db:              bytecode.0.clone(),
      ctx:             Default::default(),
      values:          Rc::new(RwLock::new(VecDeque::new())),
    }
  }

  pub fn best_error_recovery(&mut self, entry_name: String, input: String) -> String {
    
    let entry = self.db.get_entry_data_from_name(&entry_name).expect("Could not find entry point");
    let parser = &self.db;
    
    let result = parser.parse_with_recovery(&mut StringInput::from(input), entry, &Default::default());
    
    let Ok(result) = result else { return "ETF".to_string() };
    

    if let Some(best) = result.first() {
      let mut string = String::default();
      string += "result";
      for (_, sym) in &best.symbols {
        for alt in split_alternates(sym) {
          if let Some(first) = alt.first() {
            return Printer::new(first, true, parser.as_ref()).to_string();
          }
        }
      }
      string
    } else {
      "RTF".to_string()
    }
  }

  pub fn init(&mut self, entry_name: String, enable_debugger: bool) {
    let entry = self.db.get_entry_data_from_name(&entry_name).expect("Could not find entry point");

    self.ctx = self.bytecode_parser.init(entry).expect("Could not create context");

    let v = self.values.clone();

    if enable_debugger {
      let debugger: Option<Box<DebugFnNew>> = Some(Box::new(move |e, ctx, i| {
        if let LockResult::Ok(mut values) = v.write() {
          match e {
            DebugEventNew::ExecuteInstruction { instruction, is_scanner } => {
              values.push_back(JSDebugPacket {
                event: JSDebugEvent::ExecuteInstruction,
                ctx: JSCTXState::from(&ctx),
                instruction: instruction.address(),
                is_scanner: *is_scanner,
                complete: (instruction.get_opcode() == Opcode::Accept) as u32,
                ..Default::default()
              });
            }
            DebugEventNew::ExecuteState { base_instruction, is_scanner } => {
              values.push_back(JSDebugPacket {
                event: JSDebugEvent::ExecuteState,
                ctx: JSCTXState::from(&ctx),
                instruction: base_instruction.address(),
                is_scanner: *is_scanner,
                ..Default::default()
              });
            }
            _ => {}
          }
        }
      }));

      self.bytecode_parser.set_debugger(debugger);
    }

    self.running = true;
  }

  pub fn next(&mut self) -> JsValue {
    if let LockResult::Ok(mut values) = self.values.write() {
      if let Some(debug_event) = values.pop_front() {
        return debug_event.into();
      }
    };

    if self.running {
      self.next_internal();
      self.next()
    } else {
      JsValue::UNDEFINED
    }
  }

  fn next_internal(&mut self) {
    let values = self.values.clone();

    {
      let Self { ctx, reader, bytecode_parser, .. } = self;
      match bytecode_parser.next(reader, ctx) {
        Some(ParseAction::Accept { nonterminal_id, .. }) => {
          self.running = false;
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::Complete,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              nonterminal_id,
              ..Default::default()
            });
          }
        }
        Some(ParseAction::EndOfInput { .. }) => {
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::EndOfFile,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              ..Default::default()
            });
          }
        }
        Some(ParseAction::Shift { byte_offset: token_byte_offset, byte_length: token_byte_length, .. }) => {
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::Shift,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              offset_end: (token_byte_offset + token_byte_length) as usize,
              offset_start: token_byte_offset as usize,
              ..Default::default()
            });
          }
        }
        Some(ParseAction::Skip { byte_length: token_byte_length, byte_offset: token_byte_offset, .. }) => {
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::Skip,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              offset_end: (token_byte_offset + token_byte_length) as usize,
              offset_start: token_byte_offset as usize,
              ..Default::default()
            });
          }
        }
        Some(ParseAction::Reduce { nonterminal_id, rule_id, symbol_count, .. }) => {
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::Reduce,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              nonterminal_id,
              rule_id,
              symbol_count,
              ..Default::default()
            });
          }
        }
        Some(ParseAction::Error { .. }) => {
          self.running = false;
          if let LockResult::Ok(mut values) = values.write() {
            values.push_back(JSDebugPacket {
              event: JSDebugEvent::Error,
              ctx: JSCTXState::from(&ParserStackTrackers::from(&*ctx)),
              ..Default::default()
            });
          }
        }
        _ => {
          panic!("Unexpected Action!")
        }
      }
    };
  }
}

fn console_log_string(str: String) {
  unsafe { console::debug_1(&JsValue::from(&str)) };
}

#[wasm_bindgen]
pub fn get_nonterminal_names() -> JsValue {
  serde_wasm_bindgen::to_value(&ProdNames(parser::meta::nonterm_names.iter().map(|i| (*i).to_string()).collect())).unwrap()
}

#[derive(Serialize, Deserialize)]
struct ParseTree(Vec<u32>);

#[derive(Serialize, Deserialize)]
struct ProdNames(Vec<String>);

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonParseAction {
  Accept,
  EndOfInput,
  Shift { len: u32 },
  Skip { len: u32 },
  Reduce { len: u32, nterm: u32 },
  Error,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[wasm_bindgen]
pub struct JSCTXState {
  pub is_scanner: bool,
  pub end_ptr:    usize,
  pub input_ptr:  usize,
  pub sym_ptr:    usize,
  pub begin_ptr:  usize,
  pub anchor_ptr: usize,
  pub tok_len:    usize,
  pub tok_id:     u32,
  pub sym_len:    u32,
}

impl From<&ParserStackTrackers> for JSCTXState {
  fn from(ctx: &ParserStackTrackers) -> Self {
    Self {
      anchor_ptr: ctx.anchor_ptr,
      begin_ptr:  ctx.begin_ptr,
      end_ptr:    ctx.end_ptr,
      input_ptr:  ctx.input_ptr,
      is_scanner: false,
      sym_ptr:    ctx.sym_ptr,
      sym_len:    ctx.tok_byte_len,
      tok_id:     ctx.tok_id,
      tok_len:    ctx.byte_len as usize,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[wasm_bindgen]
pub enum JSDebugEvent {
  ExecuteState,
  ExecuteInstruction,
  Skip,
  Shift,
  Reduce,
  Complete,
  Error,
  EndOfFile,
  #[default]
  Undefined,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[wasm_bindgen]
pub struct JSDebugPacket {
  pub event:          JSDebugEvent,
  pub ctx:            JSCTXState,
  pub instruction:    usize,
  pub offset_start:   usize,
  pub offset_end:     usize,
  pub nonterminal_id: u32,
  pub rule_id:        u32,
  pub symbol_count:   u32,
  pub complete:       u32,
  pub is_scanner:     bool,
}
