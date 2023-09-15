import { DebuggerButton } from "./debugger_buttons";
import { FlowNode } from "../../common/flow";
import * as sherpa from "js/sherpa/sherpa_wasm.js";
import { CSTNode } from "./cst";
import { JSReductionType } from "js/sherpa/sherpa_wasm";
import { StateEffect, Range } from "@codemirror/state";
import { Decoration } from "@codemirror/view";
import { DebuggerData, EnableRestartButton, DebuggerError, EnableTransportButtons, DisableTransportButtons, DisableRestartButton } from "./debugger";

const highlight_effect = StateEffect.define<Range<Decoration>[]>();
const filter_effects = StateEffect.define<((from: number, to: number) => boolean)>();
const head_dec = Decoration.mark({ attributes: { style: "background-color: red" } });
const scan_dec = Decoration.mark({ attributes: { style: "background-color: blue" } });
const end_dec = Decoration.mark({ attributes: { style: "background-color: green" } });
export class TransportHandler extends FlowNode<DebuggerData> {

    parser: sherpa.JSByteCodeParser | null = null;
    debugger_offset: number = -1;
    debugger_steps: any[] = [];
    PARSING: boolean = false;
    allow_play: boolean = false;
    play_interval: number = -1;
    active_search_symbols: Set<string> = new Set();
    active_state_source = '';
    last_step: any = null;
    active_scanner_state_source = '';
    step_to_next_action: boolean = false;
    parser_off: [number, number] = [0, 0];
    scanner_off: [number, number] = [0, 0];
    cst_nodes: CSTNode[] = [];
    input: string = "";
    _restartParser: any;
    _stepInstruction: any;
    _stepAction: any;
    _togglePlay: any;


    constructor() {
        super();
        this._restartParser = this.restartParser.bind(this);
        this._stepInstruction = this.stepInstruction.bind(this);
        this._stepAction = this.stepAction.bind(this);
        this._togglePlay = this.togglePlayAction.bind(this);
    }

    resetData(data: DebuggerData) {
        this.debugger_offset = -1;
        this.debugger_steps = [];
        this.PARSING = false;
        this.allow_play = false;
        this.play_interval = -1;
        this.active_search_symbols = new Set();
        this.active_state_source = '';
        this.last_step = null;
        this.active_scanner_state_source = '';
        this.step_to_next_action = false;
        this.parser_off = [0, 0];
        this.scanner_off = [0, 0];
        this.cst_nodes = [];

        if (data.parser_editor) {
            this.input = data.parser_editor.state.doc.toString();
        } else {
            this.input = "";
        }
    }

    deleteParser() { if (this.parser) { this.parser.free(); } this.parser = null; }
    restartParser() { this.emit("TransportHandler_restartParser"); }
    stepInstruction() { this.emit("TransportHandler_stepInstruction"); }
    stepAction() { this.emit("TransportHandler_stepAction"); }
    togglePlayAction() { this.emit("TransportHandler_togglePlay"); }

    setupInputs() {
        DebuggerButton.get("restart").addEventListener("click", this._restartParser);
        DebuggerButton.get("step").addEventListener("click", this._stepInstruction);
        DebuggerButton.get("step-action").addEventListener("click", this._stepAction);
        DebuggerButton.get("play").addEventListener("click", this._togglePlay);
    }

    removeInputs() {
        DebuggerButton.get("restart").removeEventListener("click", this._restartParser);
        DebuggerButton.get("step").removeEventListener("click", this._stepInstruction);
        DebuggerButton.get("step-action").removeEventListener("click", this._stepAction);
        DebuggerButton.get("play").removeEventListener("click", this._togglePlay);
    }

    render_cst() {
        let ele = document.getElementById("debugger-cst-output");
        if (ele) {
            ele.innerHTML = "";
            for (const node of this.cst_nodes) {
                ele.appendChild(node.toDOM());
            }
        }
    }


    printInstruction({ states, bytecode, grammar_ctx: { db }, parser_editor }: DebuggerData, step: any = this.last_step) {


        function markSource(source: string, offsets: [number, number]) {
            return source.slice(0, offsets[0]) + "<span class=source-match>" + source.slice(...offsets) + "</span>" + source.slice(offsets[1]);
        }

        let parser = this.parser;

        let view = parser_editor?.state.doc;

        if (step && view && parser && states && bytecode && db && this.PARSING) {
            document.getElementById("debugger-ir-state").innerHTML = markSource(this.active_state_source, this.parser_off)
                + "\n\n"
                + markSource(this.active_scanner_state_source, this.scanner_off);

            document.getElementById("debugger-disassembly").innerText = sherpa.create_instruction_disassembly(step.instruction, bytecode);


            document.getElementById("debugger-metrics").innerText = JSON.stringify(step, undefined, 2)
                + "\n\n"
                + [...this.active_search_symbols].join(" | ");

        }
    }

    step(data: DebuggerData) {
        let { states, bytecode, grammar_ctx: { db }, parser_editor } = data;

        let parser = this.parser;
        let view = parser_editor;
        let input = this.input;

        if (!parser || !db || !bytecode || !states || !view || !input)
            return;

        this.debugger_offset += 1;

        if (this.debugger_offset >= this.debugger_steps.length) {
            let result = parser.next();
            if (Array.isArray(result))
                this.debugger_steps.push(...result);
            if (this.debugger_offset >= this.debugger_steps.length) { }
        }

        this.debugger_offset = Math.min(this.debugger_offset, this.debugger_steps.length - 1);

        let step;
        outer: while ((step = this.debugger_steps[this.debugger_offset])) {

            switch (step.type) {
                case "ExecuteState": {
                    let ctx = <sherpa.JSCTXState>step.ctx;
                    if (!ctx.is_scanner) {

                        let name = sherpa.get_debug_state_name(step.instruction, bytecode, db);
                        if (name) {
                            this.active_state_source = sherpa.get_state_source_string(name, states);
                            break;
                        }
                    } else {

                        let name = sherpa.get_debug_state_name(step.instruction, bytecode, db);
                        if (name) {
                            this.active_scanner_state_source = sherpa.get_state_source_string(name, states);
                            break;
                        }
                    }

                } break;
                case "ExecuteInstruction": {

                    this.last_step = step;

                    let ctx = <sherpa.JSCTXState>step.ctx;

                    let effects: any[] = [filter_effects.of((from, to) => false)];

                    let { head_ptr, scan_ptr } = ctx;

                    effects.push(highlight_effect.of([
                        head_dec.range(head_ptr, head_ptr + 1)
                    ]));

                    if (scan_ptr > head_ptr) {
                        effects.push(highlight_effect.of([
                            scan_dec.range(scan_ptr, scan_ptr + 1)
                        ]));
                    }

                    view.dispatch({ effects });

                    let token_offset = sherpa.get_debug_tok_offsets(step.instruction, bytecode);
                    if (token_offset) {
                        if (ctx.is_scanner) {
                            this.scanner_off[0] = token_offset.start - 1;
                            this.scanner_off[1] = token_offset.end - 1;
                        } else {
                            this.parser_off[0] = token_offset.start - 1;
                            this.parser_off[1] = token_offset.end - 1;
                        }
                    }

                    let debug_symbols: number[] | undefined = sherpa.get_debug_symbol_ids(step.instruction, bytecode);
                    if (debug_symbols && debug_symbols.length > 0) {
                        //debug_symbols.forEach(s => active_search_symbols.add(sherpa.get_symbol_name_from_id(s, db)));
                    }

                    if (!ctx.is_scanner) {
                        this.active_scanner_state_source = "";
                    }

                    if (this.step_to_next_action) { break; }

                    this.printInstruction(data);

                    let next_step = this.debugger_steps[this.debugger_offset + 1];

                    if (next_step && ["Shift", "Reduce", "Skip"].includes(next_step.type)) {
                        break;
                    } else {
                        break outer;
                    }
                };
                case "Skip": {
                    this.printInstruction(data);
                } break outer;;
                case "Shift": {
                    let { offset_start, offset_end } = step;
                    this.cst_nodes.push(new CSTNode(input.slice(offset_start, offset_end), "", true));
                    this.render_cst();
                    this.printInstruction(data);
                } break outer;;
                case "Reduce": {
                    let { nonterminal_id, rule_id, symbol_count } = step;
                    let expr = sherpa.get_rule_expression_string(rule_id, db);

                    if (true) {
                        let name = sherpa.get_nonterminal_name_from_id(nonterminal_id, db);
                        let node = new CSTNode(name, expr, false);
                        node.children = this.cst_nodes.slice(-symbol_count);
                        this.cst_nodes.length -= symbol_count;
                        this.cst_nodes.push(node);
                    } else {

                        switch (sherpa.get_rule_reduce_type(rule_id, db)) {
                            case JSReductionType.LeftRecursive:
                                {
                                    let nodes = this.cst_nodes.slice(-symbol_count);
                                    let first = nodes.shift();
                                    if (first) {
                                        if (first?.name != sherpa.get_nonterminal_name_from_id(nonterminal_id, db)) {
                                            // intentional fall through;
                                        } else {
                                            first.children.push(...nodes);
                                            this.cst_nodes.length -= symbol_count;
                                            this.cst_nodes.push(first);
                                            break;
                                        }
                                    }
                                }
                            case JSReductionType.Mixed:
                            case JSReductionType.SingleTerminal:
                            case JSReductionType.SemanticAction:
                                {
                                    let name = sherpa.get_nonterminal_name_from_id(nonterminal_id, db);
                                    let node = new CSTNode(name, expr, false);
                                    node.children = this.cst_nodes.slice(-symbol_count);
                                    this.cst_nodes.length -= symbol_count;
                                    this.cst_nodes.push(node);
                                }
                                break;
                        }
                    }
                    this.render_cst();
                    this.printInstruction(data);
                } break outer;;
                case "Complete": {
                    console.log("COMPLETE");
                    this.printInstruction(data);
                    this.emit("TransportHandler_disableTransportButtons");
                    this.PARSING = false;
                } break outer;;
                case "Error": {
                    console.log("FAILURE");
                    this.printInstruction(data);
                    this.emit("TransportHandler_disableTransportButtons");
                    this.PARSING = false;
                } break outer;;
                case "EndOfFile": {
                    this.PARSING = false;
                } break outer;;
                case "Undefined": {
                } break outer;
                case "ShiftToken": {
                    this.active_search_symbols.clear();
                } break outer;
            }

            this.debugger_offset++;
        }
    }

    update(t: string, data: DebuggerData): FlowNode<DebuggerData>[] {
        let base_return = [this, new EnableRestartButton];
        switch (t) {
            case "init":
                this.setupInputs();
            // Intentional fall through
            case "TransportHandler_restartParser": {
                this.deleteParser();

                if (!data.parser_editor)
                    return [...base_return, new DebuggerError("editor is not enabled")];

                if (!data.parser_editor.state.doc.toString())
                    return [...base_return, new DebuggerError("data is empty")];

                if (!data.bytecode)
                    return [...base_return, new DebuggerError("Bytecode is invalid")];

                if (!data.grammar_ctx.db)
                    return [...base_return, new DebuggerError("Database is invalid")];

                let parser_input = data.debugger_entry_selection.value;

                try {
                    this.parser = sherpa.JSByteCodeParser.new(data.parser_editor.state.doc.toString(), data.bytecode);
                    this.parser.init(parser_input, data.bytecode, data.grammar_ctx.db);
                } catch (e) {
                    return [...base_return, new DebuggerError("Parser Compiler Error")];
                }

                this.debugger_offset = -1;
                this.PARSING = true;
                this.resetData(data);

                return [...base_return, new EnableTransportButtons];
            }
            case "TransportHandler_stepInstruction": {
                this.step_to_next_action = false;
                this.step(data);
            }
            case "TransportHandler_stepAction": {
                this.step_to_next_action = true;
                this.step(data);
                this.step_to_next_action = false;
            }
            case "TransportHandler_togglePlay": {
                this.allow_play = !this.allow_play; if (this.allow_play) {
                    this.emit("TransportHandler_play");
                }
            }
            case "TransportHandler_step": {
                if (this.PARSING)
                    this.step(data);
            }
            case "TransportHandler_play": {
                if (this.allow_play && this.PARSING) { this.step(data); this.emit("TransportHandler_play"); } else {
                    this.allow_play = false;
                }
            }
            case "Input_changed": {
                return base_return;
            }
            case "TransportHandler_disableTransportButtons":
                return [...base_return, new DisableTransportButtons];
            case "db_deleted":
            case "config_changed": {
                this.deleteParser();
                this.removeInputs();
                return [new DisableTransportButtons, new DisableRestartButton];
            }
            default:
                return base_return;
        }
    }
}
