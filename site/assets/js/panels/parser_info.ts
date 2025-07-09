import * as radlr from "js/radlr/radlr_wasm.js";
import * as pipeline from "../lab/pipeline";
import { NBContentField, NBEditorField } from "../lab/notebook";
import {  ParserPlayer } from "js/lab/parser";

export function init(
  parser_info_field: NBContentField,
  parser_input_field: NBEditorField,
  grammar_pipeline_node: pipeline.GrammarDBNode,
  parser_player: ParserPlayer
) {
  parser_info_field.set_content_hidden(false);
  let db: radlr.JSBytecodeParserDB | null = null;
  let bytecode: StatefullDV | null = null;

  let instruction_view = new ContextView();
  let bytecode_view = new BytecodeView();
  let last_offset = 0;


  parser_info_field.body.appendChild(instruction_view.ele);
  parser_info_field.body.appendChild(bytecode_view.ele);
  parser_info_field.body.id = "parser-info"

  grammar_pipeline_node.addListener("loading", _ => {
    parser_info_field.set_loading(true);
    instruction_view.reset();
    bytecode_view.clear();
    last_offset = 0;
  })

  grammar_pipeline_node.addListener("bytecode_db", async new_db => {
    parser_info_field.set_loading(false);
    db = new_db;

    bytecode = new StatefullDV(new DataView(db.bytecode.buffer), 0);
    bytecode_view.create_debug_info(bytecode.to());
  })

  parser_player.addListener("state", ({interactive, data: debug_info}) => {
    if(interactive) {
      instruction_view.handle_debug_info(debug_info, parser_player.input);
      bytecode_view.handle_state(debug_info);
    }
  })

  parser_player.addListener("instruction",  ({interactive, data: debug_info})  => {
    if(interactive) {
      if (bytecode) {
        bytecode_view.handle_instruction(bytecode.to(debug_info.instruction), debug_info, parser_player.input);
      }
      instruction_view.handle_debug_info(debug_info, parser_player.input);
    }
  })

  parser_player.addListener("instruction",  ({interactive, data: debug_info})  => { 
    if(interactive) {
      let ctx = debug_info.ctx;
      
      parser_input_field.remove_character_classes();
      parser_input_field.add_character_class(ctx.input_ptr, ctx.input_ptr + 1, "dbg-input-pos");
      parser_input_field.add_character_class(ctx.anchor_ptr, ctx.anchor_ptr + 1, "dbg-anchor-pos");
      parser_input_field.add_character_class(ctx.begin_ptr, ctx.begin_ptr + 1, "dbg-begin-pos");
      parser_input_field.add_character_class(ctx.end_ptr, ctx.end_ptr + 1, "dbg-end-pos");
      
      if (debug_info.is_scanner) {
        parser_input_field.add_character_class(ctx.sym_ptr, ctx.sym_ptr + 1, "dbg-sym-pos");
        parser_input_field.add_character_class(ctx.sym_ptr, ctx.input_ptr, "dbg-sym");
      } else if (ctx.sym_len > 0) {
        parser_input_field.add_character_class(ctx.sym_ptr, ctx.sym_ptr + ctx.sym_len, "dbg-sym");
      }
    }
  });

/*   parser_player.addListener("reset", () => {
    parser_input_field.remove_character_classes();
    parser_player_node.reset();
    bytecode_view.reset();
  }) */
}

type TableInfo = {
  table_type: "hash" | "vec";
  table_base_address: number;
  input_type: number;
  scan_address: number;
  table_meta: number;
  table_length: number;
  table_start_iter: StatefullDV;
  default_address: number;
  parse_block_address: number;
}

function getLUTableData(dv: StatefullDV): TableInfo {
  let table_base_address = dv.off - 1;
  let table_type = <"hash" | "vec">((dv.dv.getInt8(table_base_address) == radlr.Opcode.HashBranch) ? "hash" : "vec");
  let input_type = dv.u8();
  let default_delta = dv.u32();
  let scan_address = dv.u32();
  let table_length = dv.u32();
  let table_meta = dv.u32();
  let table_start = table_base_address + 18;
  let table_start_iter = dv.to(table_start);
  let default_address = table_base_address + default_delta;
  let parse_block_address = table_start + table_length * 4
  return { parse_block_address, table_type, table_meta, table_base_address, input_type, scan_address, table_length, table_start_iter, default_address };
}


type StatesLU = Map<number, { pseudo_code: string }>;

class ContextView {
  ele: HTMLDivElement
  fields: {
    type: "dbg" | "ctx" | "other", val: string, input: HTMLDivElement
  }[] = []

  constructor() {
    let ele = document.createElement("div");
    ele.append((<HTMLTemplateElement>document.querySelector("#context-view-template")).content.cloneNode(true));
    this.ele = <HTMLDivElement>ele.firstElementChild!;

    for (const ele of <HTMLDetailsElement[]>Array.from(this.ele.querySelectorAll(".context-view-field"))) {
      if (ele.dataset.ctx) {
        this.fields.push({
          type: "ctx",
          val: ele.dataset.ctx,
          input: <HTMLDivElement>ele.querySelector(".context-view-field-value")
        })
      } else if (ele.dataset.dbg) {
        this.fields.push({
          type: "dbg",
          val: ele.dataset.dbg,
          input: <HTMLDivElement>ele.querySelector(".context-view-field-value")
        })
      } else if (ele.dataset.other) {
        this.fields.push({
          type: "other",
          val: ele.dataset.other,
          input: <HTMLDivElement>ele.querySelector(".context-view-field-value")
        })
      }
    }
  }

  reset() {
    for (const field of this.fields) {
      if (field.type == "ctx") {
        //@ts-ignore
        field.input.innerHTML = "";
      } else if (field.type == "dbg") {
        //@ts-ignore
        field.input.innerHTML = "";
      } else if (field.type == "other") {
        field.input.innerHTML = "";
      }
    }
  }

  handle_debug_info(debug_info: radlr.JSDebugPacket, input: string) {
    let ctx = debug_info.ctx;
    for (const field of this.fields) {
      if (field.type == "ctx") {
        //@ts-ignore
        field.input.innerHTML = <string>ctx[field.val];
      } else if (field.type == "dbg") {
        //@ts-ignore
        field.input.innerHTML = <string>debug_info[field.val];
      } else if (field.type == "other") {
        switch (field.val) {
          case ("tok"): {
            let start = ctx.sym_ptr;
            let end = ctx.input_ptr;
            let str = input.slice(start, end);
            field.input.innerHTML = str;
          } break;
        }
      }
    }
  }
}

class BytecodeView {
  ele: HTMLDivElement
  instruction_area: HTMLDivElement
  parse_state_stack: HTMLDivElement
  scan_state_stack: HTMLDivElement
  active_parse_state: HTMLDivElement
  active_scan_state: HTMLDivElement
  active_parse_state_val: number = 0
  active_scan_state_val: number = 0
  active_instruction: number = -1;
  inst_map: Map<number, number> = new Map
  instructions: { state_head: boolean, address: number, ele: HTMLDivElement }[] = []
  state_headers: Map<number, boolean> = new Map
  parse_states: number[] = [];
  scan_states: number[] = [];


  constructor() {
    let ele = document.createElement("div");
    ele.append((<HTMLTemplateElement>document.querySelector("#bytecode-view-template")).content.cloneNode(true));
    this.ele = <HTMLDivElement>ele.firstElementChild!;
    this.instruction_area = <HTMLDivElement>this.ele.querySelector(".bytecode-instructions")

    this.parse_state_stack = <HTMLDivElement>this.ele.querySelector(".parser-states .state-stack")
    this.scan_state_stack = <HTMLDivElement>this.ele.querySelector(".scanner-states .state-stack")
    this.active_parse_state = <HTMLDivElement>this.ele.querySelector(".parser-states .active-state")
    this.active_scan_state = <HTMLDivElement>this.ele.querySelector(".scanner-states .active-state")
  }

  instr_ele(address: number, mnemonic: string): HTMLDivElement {
    let ele = document.createElement("div");
    ele.classList.add("bytecode-view-instruction")
    ele.innerHTML = `
    <div class=address>${address.toString(16)}</div>
    <div class=mnemonic>${mnemonic}</div>
    `;
    this.instructions.push({ address, ele, state_head: false })
    this.inst_map.set(address, this.instructions.length - 1);
    return ele
  }


  clear() {
    this.instruction_area.innerHTML = "";
    this.instructions.length = 0;
    this.inst_map.clear();
    this.state_headers.clear();
    this.state_headers.set(8, false);
    this.reset();
  }


  reset() {
    this.parse_states.length = 0;
    this.scan_states.length = 0;

    this.parse_state_stack.innerHTML = "";
    this.scan_state_stack.innerHTML = "";

    this.active_parse_state.innerHTML = "";
    this.active_scan_state.innerHTML = "";
  }


  extra(label: string, value: any): HTMLElement {
    let ele = document.createElement("div");
    ele.classList.add("label-value")
    ele.innerHTML = `
    <div class=label>${label}</div>
    <div class=value>${value}</div>
    `
    return ele;
  }

  mnemonic(label: string): HTMLElement {
    let ele = document.createElement("div");
    ele.classList.add("mnemonic")
    ele.innerHTML = label;
    return ele;
  }

  address(address: number, label: string = "", goto_state: boolean = false): HTMLElement {
    let ele = document.createElement("div");
    ele.classList.add("label-value")

    if (goto_state)
      this.state_headers.set(address, false);

    if (label) {
      ele.innerHTML = `
      <div class=label>${label}</div>
      <div class=address>${address.toString(16)}</div>

      `
    } else {
      ele.innerHTML = `
      <div class=address>${address.toString(16)}</div>
      `
    }

    ele.addEventListener("click", () => {
      this.show_address(address);
    })

    return ele;
  }

  create_debug_info(dv: StatefullDV) {
    let len = dv.dv.byteLength;
    outer: while (dv.off < len) {
      let instruction_address = dv.off;
      let instruction_byte = dv.u8();
      switch (instruction_byte) {
        case radlr.Opcode.NoOp: {
          this.instr_ele(instruction_address, "NOOP")
        } break;
        case radlr.Opcode.Pass: {
          this.instr_ele(instruction_address, "PASS")
        } break;
        case radlr.Opcode.Fail: {
          this.instr_ele(instruction_address, "FAIL")
        } break;
        case radlr.Opcode.ShiftChar: {
          this.instr_ele(instruction_address, "SCHAR")
        } break;
        case radlr.Opcode.ShiftToken: {
          this.instr_ele(instruction_address, "SHIFT")
        } break;
        case radlr.Opcode.PeekToken: {
          this.instr_ele(instruction_address, "PTOK")
        } break;
        case radlr.Opcode.PeekTokenScanless: {
          this.instr_ele(instruction_address, "PSTOKS")
        } break;
        case radlr.Opcode.SkipToken: {
          this.instr_ele(instruction_address, "SKIP")
        } break;
        case radlr.Opcode.SkipTokenScanless: {
          this.instr_ele(instruction_address, "SKIPS")
        } break;
        case radlr.Opcode.PeekSkipToken: {
          this.instr_ele(instruction_address, "PSKIP")
        } break;
        case radlr.Opcode.PeekReset: {
          this.instr_ele(instruction_address, "RESET")
        } break;
        case radlr.Opcode.Accept: {
          this.instr_ele(instruction_address, "ACCEPT")
        } break;
        case radlr.Opcode.PopGoto: {
          this.instr_ele(instruction_address, "POP")
        } break;
        case radlr.Opcode.PushGoto: {
          let parse_mode = dv.u8();
          this.instr_ele(instruction_address, "PUSH").append(
            this.address(dv.u32(), "", true)
          )
        } break;
        case radlr.Opcode.Goto: {
          let parse_mode = dv.u8()
          this.instr_ele(instruction_address, "GOTO").append(this.address(dv.u32(), "", true))
        } break;
        case radlr.Opcode.AssignToken: {
          this.instr_ele(instruction_address, "TOKID").append(
            this.extra("tok_id", dv.u32()),
          )
        } break;
        case radlr.Opcode.Reduce: {
          this.instr_ele(instruction_address, "REDUCE").append(
            this.extra("nonterm #", dv.u32()),
            this.extra("rule #", dv.u32()),
            this.extra("sym len", dv.u16()),
          )
        } break;
        case radlr.Opcode.VectorBranch: {
          this.generate_table(dv, this.instr_ele(instruction_address, "BVEC"));
        } break;
        case radlr.Opcode.HashBranch: {
          this.generate_table(dv, this.instr_ele(instruction_address, "BHASH"));
        } break;
        case radlr.Opcode.ByteSequence: {
          let off = dv.off - 1;
          let len = dv.u16();
          let offset = dv.u32();
          let data = new Uint8Array(dv.dv.buffer, dv.off, len);
          let data_str = String.fromCodePoint(...data);

          let success_address = dv.off + len;

          this.instr_ele(instruction_address, "BYTES").append(
            this.extra("match", data_str),
            this.address(success_address, "pass", true),
            (offset > 0) ? this.address(off + offset, "fail", true) : this.extra("fail", 0),
          )
        } break;
        case radlr.Opcode.Fork: {
          this.instr_ele(instruction_address, "FORK")
        } break;
        case radlr.Opcode.ReadCodepoint: {
          this.instr_ele(instruction_address, "CP")
        } break;
        default: {
          break;
        }
      }
    }

    for (const [state_address, is_scanner] of this.state_headers) {
      let lookup = this.inst_map.get(state_address);
      if (lookup) {
        let ele = this.instructions[lookup].ele;
        this.instructions[lookup].state_head = true;

      }
    }

    for (let i = 0; i < this.instructions.length;) {
      let { ele, state_head, address } = this.instructions[i];

      if (state_head) {
        let start = i++;
        while (i < this.instructions.length && !this.instructions[i].state_head) {
          i++;
        }

        let elements = this.instructions.slice(start, i).map(d => d.ele);
        let ele = document.createElement("div");

        ele.classList.add("bytecode-state-group")


        ele.innerHTML = `
        <div class=bytecode-state-group-header> state ${address.toString(16).toUpperCase()} 
        </div>
        <div class=pseudo-code><code><pre>${create_pseudo_code(dv.to(address), false)}</pre></code></div>
      
        
        `
        ele.append(...elements);
        this.instruction_area.append(ele);

        (<HTMLDivElement>ele.firstElementChild!).addEventListener("click", e => {
          ele.classList.toggle("show-pseudo")
        });

      } else {
        this.instruction_area.append(ele)
        i++
      }
    }
  }

  generate_table(dv: StatefullDV, instruction_ele: HTMLDivElement) {
    let { parse_block_address,
      table_type, table_base_address, input_type, table_meta, table_length, table_start_iter, default_address, scan_address
    } = getLUTableData(dv);

    let val = "tok";
    let convert_val_to_string = (val: number): string => val.toString();
    let convert_codepoint = (val: number): string => `\"${String.fromCodePoint(val)}\"`;

    switch (input_type) {
      case radlr.MatchInputType.NonTerminal: {
        val = "NTERM"
      } break;
      case radlr.MatchInputType.Token: {
      } break;
      case radlr.MatchInputType.Class: {
        val = "CLASS"
      } break;
      case radlr.MatchInputType.Codepoint: {
        val = "CP"
        convert_val_to_string = convert_codepoint;
      } break;
      case radlr.MatchInputType.Byte: {
        val = "BYTE"
        convert_val_to_string = convert_codepoint;
      } break;
      case radlr.MatchInputType.EndOfFile: {
        val = "EOF"
      } break;
      case radlr.MatchInputType.ByteScanless: {
        val = "byte"
        convert_val_to_string = convert_codepoint;
      } break;
      case radlr.MatchInputType.CodepointScanless: {
        val = "cp"
        convert_val_to_string = convert_codepoint;
      } break;
      case radlr.MatchInputType.ClassScanless: {
        val = "CLASS"
      } break;
      case radlr.MatchInputType.ByteSequence: {
        throw "HUH?";
      } break;
      case radlr.MatchInputType.CSTNode: {
        throw "HUH?";
      } break;
    }

    instruction_ele.append(this.mnemonic(val));

    if (scan_address < 0xFFFF_FFFF) {
      instruction_ele.append(this.address(scan_address, "scanner", true))
    }

    if (table_type == "hash") {
      for (let i = 0; i < table_length; i++) {
        let entry = table_start_iter.u32();
        let val_id = (entry & 0x7FF);
        let address_offset = ((entry >> 11) & 0x7FF);
        let address = table_base_address + address_offset;
        instruction_ele.append(this.address(address, convert_val_to_string(val_id)))
      }
    } else {
      for (let i = 0; i < table_length; i++) {
        let entry = table_start_iter.u32();
        let val_id = table_meta + i;
        let address_offset = entry;
        let address = table_base_address + address_offset;
        instruction_ele.append(this.address(address, convert_val_to_string(val_id)))
      }
    }

    if (default_address < dv.dv.byteLength) {
      instruction_ele.append(this.address(default_address, "default"))
    }

    dv.off = parse_block_address;
  }

  update_states() {
    let data = "";

    for (const state of this.parse_states.slice().reverse()) {
      data += "<div class='state-address'>" + state.toString(16) + "</div>"
    }

    this.parse_state_stack.innerHTML = data;

    data = ""
    for (const state of this.scan_states.reverse()) {
      data += "<div class='state-address'>" + state.toString(16) + "</div>"
    }

    this.scan_state_stack.innerHTML = data;

    this.active_scan_state.innerHTML = "<div class='state-address'>" + this.active_scan_state_val.toString(16) + "</div>";
    this.active_parse_state.innerHTML = "<div class='state-address'>" + this.active_parse_state_val.toString(16) + "</div>";
  }

  handle_state(debug_info: radlr.JSDebugPacket) {
    let address = debug_info.instruction
    let is_scanner = debug_info.is_scanner;
    this.set_active_state(is_scanner, address);
  }

  private set_active_state(is_scanner: boolean, address: number) {

    if (!address) return;

    if (is_scanner) {
      this.active_scan_state_val = address
    } else {
      this.active_parse_state_val = address
    }


  }

  handle_instruction(dv: StatefullDV, debug_info: radlr.JSDebugPacket, input: string) {
    this.update_states();
    let instruction = dv.off;
    let is_scanner = debug_info.is_scanner;

    let index = this.inst_map.get(instruction);
    if (index) {
      if (this.active_instruction >= 0) {
        this.instructions[this.active_instruction].ele.classList.remove("active")
      }

      this.active_instruction = index

      let { ele } = this.instructions[index];
      ele.classList.add("active")

      this.show_address(instruction);
    }

    switch (dv.u8()) {
      case radlr.Opcode.PopGoto: {
        if (is_scanner) {
          this.scan_states.pop();
        } else {
          this.parse_states.pop();
        }
      } break
      case radlr.Opcode.Pass: {
        if (is_scanner) {
          this.set_active_state(is_scanner, this.scan_states.pop()!);
        } else {
          this.set_active_state(is_scanner, this.parse_states.pop()!);
        }
      } break
      case radlr.Opcode.Goto: {
        let parse_mode = dv.u8();
        let address = dv.u32();
        this.set_active_state(is_scanner, address);
      } break
      case radlr.Opcode.PushGoto: {
        let parse_mode = dv.u8();
        let address = dv.u32();
        if (is_scanner) {
          this.scan_states.push(address);
        } else {
          this.parse_states.push(address);
        }

      } break;
    }


  }

  private show_address(address: number) {
    let index = this.inst_map.get(address);
    if (index) {

      let { ele } = this.instructions[index];

      let height = this.instruction_area.getBoundingClientRect().height;
      let top = this.instruction_area.getBoundingClientRect().top;

      let ele_top = ele.getBoundingClientRect().top

      let diff = ele_top - top;

      if (diff > (height - 40) || diff < 0) {
        ele.scrollIntoView();
        this.instruction_area.scrollTop -= height / 8;
      }
    }
  }
}


function create_pseudo_code(dv: StatefullDV, is_scanner: boolean = false): string {

  let gotos: string[] = [];

  let pseudo_code = "";
  pseudo_code += `s_${dv.off.toString(16)}${is_scanner ? " SCANNER " : ""}(lex, ctx, in)`;

  if (!is_scanner) {
    pseudo_code += "\n  if ctx.pop > 0"
    pseudo_code += "\n    ctx.pop--"
    pseudo_code += "\n    return"
  }

  pseudo_code += merge_lines(process_instructions(dv, gotos).split("\n",));
  pseudo_code += merge_goto_lines(gotos);

  return pseudo_code;
}

function merge_lines(internal_data: string[], prefix: string = "\n  ") {
  let pseudo_code: string = "";
  for (const goto of internal_data) {
    if (!goto) continue
    pseudo_code += prefix + goto;
  }
  return pseudo_code;
}

function merge_goto_lines(internal_data: string[], prefix: string = "\n  ") {
  let pseudo_code: string = "";
  let i = -1;
  for (const goto of internal_data) {
    i++
    if (!goto) continue

    if (i == internal_data.length - 1) {
      pseudo_code += prefix + "tail return " + goto;
    } else {
      pseudo_code += prefix + goto;
    }

  }
  return pseudo_code;
}

function process_instructions(dv: StatefullDV, gotos: string[], root_name: string = "", is_scanner: boolean = false): string {
  let pseudo_code: string = ""
  let i = 0;
  let have_root = false

  outer: while (i++ < 10) {
    let instruction_address = dv.off;
    let instruction_byte = dv.u8();
    switch (instruction_byte) {
      case radlr.Opcode.NoOp: {
      } break;
      case radlr.Opcode.Pass: {
        if (gotos.length == 0) {
          pseudo_code += `\nreturn`;
        }
      } break outer;
      case radlr.Opcode.Fail: {
        if (is_scanner)
          pseudo_code += `\nreturn`;
        else
          pseudo_code += `\nthrow "could not continue"`;
      } break outer;
      case radlr.Opcode.ShiftChar: {
        pseudo_code += "\nlex.head += 1";
      } break;
      case radlr.Opcode.ShiftToken: {
        pseudo_code += "\nlen = lex.head - lex.tail";
        pseudo_code += "\nctx.emit(SHIFT { id: lex.tok_id, len })";
        pseudo_code += "\nlex.tail   = lex.head";
        pseudo_code += "\nlex.anchor = lex.head";
      } break;
      case radlr.Opcode.PeekToken: {
        pseudo_code += "\nctx.peek(tok)";
      } break;
      case radlr.Opcode.PeekTokenScanless: {
        pseudo_code += "\nctx.tok_len = ctx.sym_len\nctx.peek(tok)";
      } break;
      case radlr.Opcode.SkipToken: {
        pseudo_code += "\nlen = lex.head - lex.tail";
        pseudo_code += "\nctx.emit(SKIP { id: lex.tok_id, len })";
        pseudo_code += "\nlex.tail   = lex.head";
        pseudo_code += "\nlex.anchor = lex.head";
        pseudo_code += "\ntail return " + root_name + "( lex, ctx, in )";
      } break outer;
      case radlr.Opcode.SkipTokenScanless: {
        pseudo_code += "\nlen = lex.head - lex.tail";
        pseudo_code += "\nctx.emit(SKIP { id: lex.tok_id, len })";
        pseudo_code += "\nlex.tail   = lex.head";
        pseudo_code += "\nlex.anchor = lex.head";
        pseudo_code += "\ntail return " + root_name + "( lex, ctx, in )";
      } break outer;
      case radlr.Opcode.PeekSkipToken: {
        pseudo_code += "\nlex.tail = lex.head";
        pseudo_code += "\ntail return " + root_name + "( lex, ctx, in )";
      } break outer;
      case radlr.Opcode.PeekReset: {
        pseudo_code += "\nlex.head = lex.anchor";
        pseudo_code += "\nlex.tail = lex.anchor";
      } break;
      case radlr.Opcode.Accept: {
        pseudo_code += "\nctx.emit(ACCEPT)";
      } break outer;
      case radlr.Opcode.PopGoto: {
        pseudo_code += "\nctx.pop+=1";
      } break;
      case radlr.Opcode.PushGoto: {
        let parse_mode = dv.u8();
        let address = dv.u32();
        gotos.unshift(`s_${address.toString(16)}(lex, ctx, in)`);
      } break;
      case radlr.Opcode.Goto: {
        let parse_mode = dv.u8();
        let address = dv.u32();
        gotos.unshift(`s_${address.toString(16)}(lex, ctx, in)`);
      } break outer;
      case radlr.Opcode.AssignToken: {
        let tok_id = dv.u32();
        pseudo_code += `\nlex.tok_id = ${tok_id}`;
      } break;
      case radlr.Opcode.Reduce: {
        let nterm = dv.u32();
        let rule_id = dv.u32();
        let symbol_count = dv.u16();

        pseudo_code += `\nctx.nt_id = ${nterm}`;
        pseudo_code += `\nctx.emit(REDUCE { nt_id: ${nterm}, rule: ${rule_id}, sym_count: ${symbol_count} })`;
      } break;
      case radlr.Opcode.VectorBranch: {
        throw "Vector branch not implemented"
        pseudo_code += generate_table_string(dv);
      } break;
      case radlr.Opcode.HashBranch: {
        pseudo_code += generate_table_string(dv, is_scanner);
      } break outer;
      case radlr.Opcode.ByteSequence: {
        let off = dv.off - 1;
        let len = dv.u16();
        let offset = dv.u32();
        let data = new Uint8Array(dv.dv.buffer, dv.off, len);
        let data_str = String.fromCodePoint(...data);

        let success_address = dv.off + len;

        pseudo_code += `if lex.slice(${len}) == "${data_str}"`
        pseudo_code += `\n lex.head += (${len})`

        let gotos: string[] = [];
        pseudo_code += merge_lines(process_instructions(dv.to(success_address), gotos, root_name, true).split("\n"));
        pseudo_code += merge_goto_lines(gotos);
        pseudo_code += "\nelse"

        if (offset > 0) {
          let fail_address = off + offset;
          let gotos: string[] = [];
          pseudo_code += merge_lines(process_instructions(dv.to(fail_address), gotos, root_name, true).split("\n"));
          pseudo_code += merge_goto_lines(gotos);
        } else {
          pseudo_code += `\n  throw \"Lexer does not have sequence '${data_str}' at current offset\"`;
        }
      } break outer;
      case radlr.Opcode.Fork: {
        console.log("FORK")
      } break;
      case radlr.Opcode.ReadCodepoint: {
      } break;
      default: {
        break outer;
      }
    }
  }

  return pseudo_code;
}


function generate_table_string(dv: StatefullDV, is_scanner: boolean = false): string {
  let out_string = "";
  let { table_base_address, input_type, scan_address, table_length, table_start_iter, default_address, } = getLUTableData(dv);

  let val = "tok";
  let error = "unrecognized symbol";
  let convert_val_to_string = (val: number): string => val.toString();
  let convert_val_to_bool = (val: number): string => (val > 0) + "";
  let convert_codepoint = (val: number): string => `\"${String.fromCodePoint(val)}\"`;

  let root_name = `s_${table_base_address.toString(16)}`

  switch (input_type) {
    case radlr.MatchInputType.NonTerminal: {
      val = "ctx.nt_id"
      error = "incorrect non-terminal was produced"
    } break;
    case radlr.MatchInputType.Token: {
      if (scan_address < 0xFFFF_FFFF) {
        out_string += `\ns_${scan_address.toString(16)}(lex, ctx, in)`;
      }
      val = "lex.tok_id"
    } break;
    case radlr.MatchInputType.Class: {
      val = "in.class_at(lex.head)"
      is_scanner = true;
    } break;
    case radlr.MatchInputType.Codepoint: {
      val = "in.cp_at(lex.head)"
      convert_val_to_string = convert_codepoint;
      is_scanner = true;
    } break;
    case radlr.MatchInputType.Byte: {
      val = "in.byte_at(lex.head)"
      convert_val_to_string = convert_codepoint;
      is_scanner = true;
    } break;
    case radlr.MatchInputType.EndOfFile: {
      val = "in.is_eof_at(lex.head)"
      convert_val_to_string = convert_val_to_bool;
      is_scanner = true;
    } break;
    case radlr.MatchInputType.ByteScanless: {
      val = "byte"
      out_string += `\nbyte = in.byte_at(lex.tail)`
      out_string += `\nlex.head = lex.tail + 1`
      convert_val_to_string = convert_codepoint;
    } break;
    case radlr.MatchInputType.CodepointScanless: {
      val = "cp"
      out_string += `\ncp = in.cp_at(lex.tail)`
      out_string += `\nlex.head = lex.tail + in.cp_len_at(lex.tail)`
      convert_val_to_string = convert_codepoint;
    } break;
    case radlr.MatchInputType.ClassScanless: {
      val = "class"
      out_string += `\nclass = lex.class_at(lex.tail)`
      out_string += `\nlex.head = lex.tail + in.cp_len_at(lex.tail)`
    } break;
    case radlr.MatchInputType.ByteSequence: {
      throw "HUH?";
    } break;
    case radlr.MatchInputType.CSTNode: {
      throw "HUH?";
    } break;
  }

  out_string += `\n \nmatch ${val}`;

  let inlined_default = false;

  let map: Map<number, number[]> = new Map();

  for (let i = 0; i < table_length; i++) {
    let entry = table_start_iter.u32();

    let val_id = (entry & 0x7FF);
    let address_offset = ((entry >> 11) & 0x7FF);
    //let meta = ((entry >> 22) & 0x3FF) - 512;
    let address = table_base_address + address_offset;

    if (!map.get(address)) {
      map.set(address, []);
    }

    let entries = map.get(address)!;
    entries.push(val_id)
  }

  for (const [address, entries] of map.entries()) {
    if (address == default_address) {
      out_string += `\n  else`;
      inlined_default = true;
    } else {
      for (const entry of entries) {
        out_string += `\n  ${convert_val_to_string(entry)}`;
      }
    }

    let gotos: string[] = [];
    out_string += merge_lines(process_instructions(dv.to(address), gotos, root_name, is_scanner).split("\n"), "\n    ");
    out_string += merge_goto_lines(gotos, "\n    ");
  }

  if (!inlined_default) {
    if (default_address < dv.dv.byteLength) {
      out_string += `\n  else`;
      let gotos: string[] = [];
      out_string += merge_lines(process_instructions(dv.to(default_address), gotos, root_name, is_scanner).split("\n"), "\n    ");
      out_string += merge_goto_lines(gotos, "\n    ");
    } else if (is_scanner) {
      out_string += `\n  else\n    return"`;
    } else {
      out_string += `\n  else\n    throw "${error}"`;
    }
  }

  return out_string
}

class StatefullDV {
  dv: DataView
  off: number = 0

  constructor(dv: DataView, offset: number) {
    this.off = offset
    this.dv = dv
  }

  u32(): number {
    let off = this.off;
    this.off += 4;
    return this.dv.getUint32(off, true)
  }

  u16(): number {
    let off = this.off;
    this.off += 2;
    return this.dv.getUint16(off, true)
  }

  u8(): number {
    let off = this.off;
    this.off += 1;
    return this.dv.getUint8(off)
  }

  to(offset: number = this.off): StatefullDV {
    return new StatefullDV(this.dv, offset)
  }
}

