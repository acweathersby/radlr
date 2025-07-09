import { JSDebugEvent } from "js/radlr/radlr_wasm";
import * as radlr from "js/radlr/radlr_wasm.js";
import { Eventable } from "./eventable";
import { NBEditorField } from "./notebook";
import { GrammarDBNode } from "./pipeline";
import { JSBytecodeParserDB } from "js/radlr/radlr_wasm";

export type ReduceStruct = { rule_id: number, symbols: number, non_terminal_id: number, db: radlr.JSBytecodeParserDB };
export type ShiftStruct = { token: string, byte_offset: number, byte_len: number, col: number, line: number, token_id: number, db: radlr.JSBytecodeParserDB }

export type ParserHandlerEvents = {
  shift: {
    interactive: boolean,
    byte_offset: number,
    byte_len: number,
    col: number,
    line: number,
    token: string,
    token_id: number,
    db: radlr.JSBytecodeParserDB
  },
  reduce: {
    interactive: boolean,
    non_terminal_id: number,
    rule_id: number,
    symbols: number, db: radlr.JSBytecodeParserDB
  },
  step: {
    interactive: boolean,
    data: radlr.JSDebugPacket,
    input: string
  },
  error: radlr.JSDebugPacket,
  complete: void,
  instruction: {
    interactive: boolean,
    data: radlr.JSDebugPacket
  },
  state: {
    interactive: boolean,
    data: radlr.JSDebugPacket
  },
  reset: {interactive: boolean},
  eof: void
}


export class Parser {
  input: string = ""
  parser: radlr.JSByteCodeParser | null = null
  db: radlr.JSBytecodeParserDB
  PARSING: boolean = false
  INITIALIZED: boolean = false
  can_play: boolean = true;

  on_state: ((dbg: ParserHandlerEvents["state"]) => { should_stop: boolean }) = _ => ({should_stop: false});
  on_instruction: ((dbg: ParserHandlerEvents["instruction"]) => { should_stop: boolean }) = _ => ({should_stop: false});
  on_shift: ((shift_data: ParserHandlerEvents["shift"]) => void) = _ => {};
  on_reduce: ((reduce_data: ParserHandlerEvents["reduce"]) => void) = _ => {};
  on_error: ((arg: ParserHandlerEvents["error"]) => void) = _ => {};
  on_step: ((arg: ParserHandlerEvents["step"]) => void) = _ => {};
  on_eof: ((arg: ParserHandlerEvents["eof"]) => void) = _ => {};
  on_complete: (() => void) = () => {};

  constructor(db: radlr.JSBytecodeParserDB, input: string) {
    this.db = db;
    this.input = input;
  }

  destroy() {
    if (this.parser)
      this.parser.free();
  }

  generate_error_corrected_output(input: string): string {
    const parser = radlr.JSByteCodeParser.new("", this.db);
    return parser.best_error_recovery("default", input) ?? ""
  }

  private next(interactive: boolean = false, key_frame: boolean = false, step_to_next_action: boolean = false): boolean {

    let parser = this.parser;
    let input = this.input;
    let db = this.db;

    if (!parser)
      return false;

    let step: radlr.JSDebugPacket | undefined = undefined;

    let i = 0;

    outer: while (this.can_play) {
      i++;
      step_to_next_action = false;

      if (step) {
        step.free();
      }

      step = parser.next();

      if (!step) {
        return false;
      }

      if (step.event == JSDebugEvent.ExecuteInstruction && step.complete) {
        key_frame = true;
      }

      switch (step.event) {

        case JSDebugEvent.ExecuteState: {
          if (this.on_state({interactive,data:step}).should_stop) {
            break outer
          }
          break
        };

        case JSDebugEvent.ExecuteInstruction: {

          if (this.on_instruction({interactive,data:step}).should_stop) {
            break outer;
          }

          break;
        };

        case JSDebugEvent.Complete: {
          this.on_complete();
          this.PARSING = false;
        } break outer;

        case JSDebugEvent.Error: {
          this.on_error(step);
          this.PARSING = false;
        } break outer;

        case JSDebugEvent.EndOfFile: {
          this.on_eof();
          this.PARSING = false;
        } break outer;
        case JSDebugEvent.Shift: {
          this.on_shift({
            interactive,
            byte_offset: step.offset_start,
            byte_len: step.offset_end - step.offset_start,
            col: 0,
            line: 0,
            token: this.input.slice(step.offset_start, step.offset_end),
            token_id: step.ctx.tok_id,
            db
          })
        } break
        case JSDebugEvent.Reduce: {
          this.on_reduce({
            interactive,
            non_terminal_id: step.nonterminal_id,
            rule_id: step.rule_id,
            symbols: step.symbol_count, db
          });
        } break
        case JSDebugEvent.Skip: break
        case JSDebugEvent.Undefined:
        default: break outer;
      }
    }

    if (step) {
      step.free();
    }

    return true;
  }

  public init(parser_entry_name: string = "default", input: string = this.input) {
    this.INITIALIZED = false;
    this.input = input;

    let selected_entry = "default";

    for (const [name, entry] of this.db.entry_points) {
      if (name == parser_entry_name) {
        selected_entry = name;
        break;
      }
    }

    if (this.parser) {
      this.parser.free();
    }

    try {
      this.parser = radlr.JSByteCodeParser.new(this.input, this.db);
      this.parser.init(selected_entry, true);
      this.INITIALIZED = true;
      this.PARSING = true;
    } catch (e) {
      console.error(e);
    }

  } 

  public stop() {
    this.can_play = false;
  }

  public play(interactive = false) {
    if (this.INITIALIZED && this.PARSING) {
      this.can_play = true;
      this.next(interactive, false, false);
    }
  }
}


export class ParserPlayer extends Eventable<ParserHandlerEvents> { 
  parser: Parser | null = null;
  input_string: string = "";
  db: JSBytecodeParserDB | null = null;

  update_time_out= -1;

  stop_on_instruction: boolean = false;
  stop_on_state: boolean = false;

  step_state() {
    this.stop_on_state = true;
    this.stop_on_instruction = false;
    if(this.parser) 
      this.parser.play(true);
  }

  step_instruction() {
    this.stop_on_state = false;
    this.stop_on_instruction = true;
    if(this.parser)
      this.parser.play(true);
  }

  play_till_complete(rate: number = 0) {
    
    this.stop_on_state = false;
    this.stop_on_instruction = false;
    if(this.parser)
      this.parser.play(false);
  }

  stop() {
    if(this.parser)
      this.parser.stop();
  }

  restart(interactive: boolean = false) {
    
    if(this.db && this.parser) {
      
      this.parser.init("default", this.input_string);
      this.emit("reset", {interactive});
    }
  }

  get input() : string {
    return this.input_string
  }

  private init_parser(parser: Parser) {
    this.parser = parser;
    this.parser.on_complete = () => this.emit("complete", void 0);
    this.parser.on_error = err => this.emit("error", err);
    this.parser.on_reduce = red => this.emit("reduce", red);
    this.parser.on_shift = shft => this.emit("shift", shft);
    this.parser.on_step = step => this.emit("step", step);
    this.parser.on_eof = eof => this.emit("eof", eof);
    this.parser.on_instruction = instr => (this.emit("instruction", instr), {should_stop:this.stop_on_instruction});
    this.parser.on_state = state => {
      this.emit("state", state);
      return {should_stop:this.stop_on_state}
    };
  }

  initPipeline(
    parser_input_panel: NBEditorField,
    grammar_pipeline: GrammarDBNode,
  ) {

  

    parser_input_panel.addListener("text_changed", field => {
      

      this.input_string = field.get_text();

      if(this.update_time_out) {
        clearTimeout(this.update_time_out);
        this.update_time_out = 0;
      }

      this.update_time_out = setTimeout(() => {
        if(this.parser) {
          this.restart(false);
          this.play_till_complete();
        }
      }, 300)
    })

    grammar_pipeline.addListener("bytecode_db", new_db => {
      this.db = new_db;

      if(this.parser) {
        this.parser.destroy();
      }
      
      this.init_parser(new Parser(this.db, this.input_string));
      this.restart(true);
      this.play_till_complete();
    });
  }
}


/**
 * Controls the "Global Parser", yielding parse information to interested
 * listeners to, allowing of an orchestration of UI updates based on the global
 * parser state.
 */
export const GlobalParser = new ParserPlayer;