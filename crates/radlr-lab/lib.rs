//!
//! # RADLR LAB
pub mod serialize;

#[cfg(feature = "host")]
use radlr_bytecode::compile_bytecode;

#[cfg(feature = "host")]
use radlr_core::{
  ParserClassification,
  ParserConfig,
  ParserStore,
  RadlrError,
  RadlrGrammar,
  RadlrGrammarDatabase,
  RadlrIRParser,
  RadlrResult,
};

#[cfg(feature = "host")]
use std::io;

#[cfg(feature = "host")]
use std::path::PathBuf;

#[cfg(feature = "host")]
use tungstenite::*;

#[cfg(feature = "host")]
use std::{net::TcpListener, thread::spawn};

#[cfg(feature = "client")]
use wasm_bindgen::prelude::*;

#[repr(u8)]
#[cfg_attr(feature = "client", wasm_bindgen)]
pub enum WSRequestCodes {
  Undefined    = 0,
  /**
   * Request to build grammar artifacts (parser, states, ast, etc)
   */
  BuildGrammar = 1,
  Ping         = 2,
}

#[repr(u8)]
#[cfg_attr(feature = "client", wasm_bindgen)]
pub enum WSResponseCodes {
  Undefined      = 0,
  Classification = 1,
  ByteCode       = 2,
  Pong           = 3,
}

#[cfg(feature = "host")]
const DEFAULT_PORT: u16 = 15421;

#[cfg(feature = "host")]
pub fn run_lab_server(port: Option<u16>) -> Result<(), RadlrError> {
  let port = port.unwrap_or(DEFAULT_PORT);
  let ip4 = "0.0.0.0";

  let server = TcpListener::bind(&format!("{ip4}:{port}"))?;

  println!("Listening on {ip4}:{port}");

  let mut sockets: Vec<std::thread::JoinHandle<Result<(), RadlrError>>> = Vec::new();

  for stream in server.incoming() {
    for socket_handle in sockets.drain(..).collect::<Vec<_>>() {
      if socket_handle.is_finished() {
        if let Err(err) = socket_handle.join() {
          eprintln!("{err:?}");
        }
      } else {
        sockets.push(socket_handle);
      }
    }

    match stream {
      Ok(stream) => {
        println!("Connected to {:?}", stream.peer_addr().expect("Could not read peer address. Somehow, this is fatal."));

        let socket_handle = spawn(move || -> Result<(), RadlrError> {
          match accept(stream) {
            Ok(mut websocket) => loop {
              match websocket.read() {
                Ok(msg) => match msg {
                  Message::Ping(ping) => {
                    println!("Ping received {ping:?}");
                  }
                  Message::Pong(pong) => {
                    println!("Pong received {pong:?}");
                  }
                  Message::Text(text_data) => {
                    println!("Text received {text_data:?}");
                  }
                  Message::Binary(binary_data) => {
                    if binary_data.len() == 1 {
                      if binary_data[0] == WSRequestCodes::Ping as u8 {
                        websocket.send(Message::Binary(vec![WSResponseCodes::Pong as u8])).unwrap();
                        websocket.flush().unwrap();
                        websocket.send(Message::Close(None)).unwrap();
                      } else {
                        websocket.send(Message::Close(None)).unwrap();
                      }
                    }

                    if binary_data.len() == 0 {
                      println!("Ignoring empty binary");
                      continue;
                    }

                    match unsafe { std::mem::transmute::<_, WSRequestCodes>(binary_data[0]) } {
                      WSRequestCodes::BuildGrammar => {
                        println!("Request to build grammar");

                        const CONFIG_SIZE: usize = size_of::<ParserConfig>();

                        if binary_data.len() < 8 {
                          eprintln!("Invalid minimum length {}; expected at least 8 bytes transmitted.", binary_data.len());
                        }

                        // SAFETY: assumes the target machine is little endian
                        let mut string_len: u32 = 0;

                        unsafe { binary_data.as_ptr().offset(4).copy_to((&mut string_len) as *mut _ as *mut u8, 4) };

                        println!("ParserConfig::size {}", CONFIG_SIZE);

                        let parser_config: ParserConfig =
                          unsafe { std::mem::transmute_copy(&*(binary_data.as_ptr().offset(8) as *const ParserConfig)) };

                        let string_off = CONFIG_SIZE + 8;

                        if (binary_data.len() - string_off) < string_len as usize {
                          eprintln!("Invalid string length {}; expected string length of {}", binary_data.len() - 8, string_len);
                          continue;
                        } else if string_len == 0 {
                          eprintln!("String section is empty");
                          continue;
                        }

                        println!("{parser_config:#?}");

                        match String::from_utf8(binary_data[string_off..string_off + string_len as usize].to_vec()) {
                          Ok(grammar_string) => {
                            println!("Grammar is:\n======\n{grammar_string}\n=====");

                            println!("Compiling Grammar!");

                            let grammar_path = PathBuf::from("/grammar.radlr".to_string());

                            match RadlrGrammar::new()
                              .add_source_from_string(&grammar_string, &grammar_path, false)?
                              .build_db(&grammar_path, parser_config)
                            {
                              Ok(db) => {
                                fn build_states(
                                  db: &RadlrGrammarDatabase,
                                  parser_config: ParserConfig,
                                  optimize: bool,
                                ) -> RadlrResult<RadlrIRParser> {
                                  let pool = radlr_core::worker_pool::StandardPool::new_with_max_workers().unwrap();
                                  let states = db.build_states(parser_config, &pool)?;
                                  let parser = states.build_ir_parser(optimize, false, &pool)?;

                                  println!("Parser Result \n{}", parser.report.to_string());
                                  Ok(parser)
                                }

                                let parser = match build_states(&db, parser_config, true) {
                                  Ok(parser) => parser,
                                  Err(err) => {
                                    eprintln!("{err}");
                                    continue;
                                  }
                                };

                                println!("Built parser: Type {:?}", parser.get_classification().to_string());
                                println!("Sending data back!");

                                let classification = parser.get_classification();
                                if let Err(err) = host_send_classification(classification, &mut websocket) {
                                  eprintln!("Error encountered sending parser classification data\n {err}");
                                }

                                if let Err(err) = host_send_bytecode_db(parser, &mut websocket) {
                                  eprintln!("Error encountered sending bytecode data\n {err}");
                                };

                                continue;
                              }

                              Err(err) => {
                                eprintln!("Errors encountered in grammar {err}");
                                continue;
                              }
                            }
                          }
                          Err(err) => {
                            eprintln!("Failed to decode utf8 data:\n{err}");
                          }
                        }
                      }
                      _ => {}
                    }
                  }
                  Message::Close(_) => {
                    println!("Close received");
                    break Ok(());
                  }
                  _ => {}
                },
                Err(err) => break Err(RadlrError::IOError(format!("{err:?}"))),
              }
            },
            Err(err) => Err(RadlrError::IOError(err.to_string())),
          }
        });
        sockets.push(socket_handle);
      }
      Err(err) => {
        eprintln!("{err} \n Exiting due to error");
        break;
      }
    }
  }

  Ok(())
}

#[cfg(feature = "host")]
fn host_send_bytecode_db(
  parser: RadlrIRParser,
  websocket: &mut WebSocket<std::net::TcpStream>,
) -> Result<(), tungstenite::Error> {
  let bytecode = match compile_bytecode(&parser, true) {
    Err(err) => return Err(tungstenite::Error::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))),
    Ok(data) => data,
  };

  let mut data = serialize::bytecode_db::export_bytecode_db(&bytecode);
  let mut export_data = Vec::new();
  export_data.push(WSResponseCodes::ByteCode as u8);
  export_data.append(&mut data);
  websocket.write(Message::Binary(export_data))?;
  websocket.flush()?;
  Ok(())
}

#[cfg(feature = "host")]
/// Emit a message with containing a parser classification structure
fn host_send_classification(
  classification: radlr_core::ParserClassification,
  websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
) -> Result<(), tungstenite::Error> {
  let mut data = [0u8; 1 + size_of::<ParserClassification>()];

  data[0] = WSResponseCodes::Classification as u8;

  unsafe {
    std::ptr::copy(std::mem::transmute(&classification), data.as_mut_ptr().offset(1), size_of::<ParserClassification>());
  };

  websocket.write(Message::Binary(data.to_vec()))?;
  websocket.flush()?;

  Ok(())
}
