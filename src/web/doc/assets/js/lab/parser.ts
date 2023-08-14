/**
 * Provides functionality for parsing inputs using compiled parsers 
 */
import * as sherpa from "js/sherpa/sherpa_wasm.js";
import { EventType, GrammarContext } from "./grammar_context";
import { ViewPlugin, DecorationSet, ViewUpdate } from "@codemirror/view"
import { StateField, StateEffect, Range } from "@codemirror/state"
import { EditorView, Decoration } from "@codemirror/view"


const head_dec = Decoration.mark({ attributes: { style: "background-color: red" } });
const scan_dec = Decoration.mark({ attributes: { style: "background-color: blue" } });
const end_dec = Decoration.mark({ attributes: { style: "background-color: green" } });

const highlight_effect = StateEffect.define<Range<Decoration>[]>();
const filter_effects = StateEffect.define<((from: number, to: number) => boolean)>();

export function parserHost(ctx: GrammarContext, {
    debugger_start_stop_button,
    debugger_step_button,
    debugger_into_button,
    debugger_out_button,
    debugger_output,
}: {
    debugger_start_stop_button: HTMLButtonElement,
    debugger_step_button: HTMLButtonElement,
    debugger_into_button: HTMLButtonElement,
    debugger_out_button: HTMLButtonElement,
    debugger_output: HTMLDivElement
}) {
    let view: EditorView | null = null;
    let bytecode: sherpa.JSBytecode | null = null;
    let states: sherpa.JSParseStates | null = null;
    let parser: sherpa.JSByteCodeParser | null = null;
    let PARSING: boolean = false;
    let debugger_steps: any[] = [];
    let debugger_offset: number = -1;
    let play_interval = -1;

    ctx.addListener(EventType.GrammarAdded, ctx => {
        console.log("Grammar Added")
    })

    ctx.addListener(EventType.DBDeleted, ctx => {
        console.log("DBDeleted")
    })

    ctx.addListener(EventType.DBCreated, ctx => {
        console.log("DBCreated")

        if (states) {
            states.free()
            states = null;
        }

        if (bytecode) {
            bytecode.free()
            bytecode = null;
        }

        // Now we can create a parser. 
        let db = ctx.db;

        if (!db) return;

        try {
            states = sherpa.create_parser_states(db, false);
            bytecode = sherpa.create_bytecode(db, states);
            // Build the soup.
            let output = document.getElementById("bytecode-output");
            if (output) {
                output.innerText = sherpa.create_bytecode_disassembly(bytecode);
            }

        } catch (e) {
            console.log(e)
        }
    })

    function destroy_parser() {
        if (parser) {
            parser.free();
            parser = null;
        }
    }

    function stop_parser() {
        destroy_parser();
        toggle_play(true);
        PARSING = false;
        debugger_start_stop_button.innerHTML = "start";
        debugger_start_stop_button.classList.remove("started");
        debugger_output.innerText = "";
    }

    function start_parser() {
        if (view && states && bytecode && ctx.db && !PARSING) {
            debugger_start_stop_button.innerHTML = "stop";
            debugger_start_stop_button.classList.add("started");
            destroy_parser();

            debugger_offset = -1;
            debugger_steps.length = 0;

            parser = sherpa.JSByteCodeParser.new(view.state.doc.toString(), bytecode);
            parser.init("entry", bytecode, ctx.db);

            view.dispatch({ userEvent: "debugger.start" })
            PARSING = true;

            step_forward();
        }
    }

    let active_search_symbols: Set<string> = new Set()

    function step_forward() {
        if (view && parser && states && bytecode && ctx.db && PARSING) {

            let db = ctx.db;
            debugger_offset += 1;

            if (debugger_offset >= debugger_steps.length) {
                let result = parser.next();
                if (Array.isArray(result))
                    debugger_steps.push(...result);

                if (debugger_offset >= debugger_steps.length) {
                    toggle_play(true);
                }
            }

            debugger_offset = Math.min(debugger_offset, debugger_steps.length - 1);

            let step;
            outer: while ((step = debugger_steps[debugger_offset])) {
                switch (step.type) {
                    case "ShiftToken":
                        active_search_symbols.clear();
                        break;
                    case "ExecuteInstruction": {

                        if (!step.is_scanner) {
                            let debug_symbols: number[] = sherpa.get_debug_symbol_ids(step.instruction, bytecode);
                            if (debug_symbols.length > 0) {
                                debug_symbols.forEach(s => active_search_symbols.add(sherpa.get_symbol_name_from_id(s, db)));
                            }
                        }

                        debugger_output.innerText = JSON.stringify(step, undefined, 2)
                            + "\n\n"
                            + [...active_search_symbols].join(" | ")
                            + "\n\n"
                            + sherpa.create_instruction_disassembly(step.instruction, bytecode)
                            ;

                        let effects: any[] = [filter_effects.of((from, to) => false)]

                        let { head_ptr, scan_ptr } = step;

                        effects.push(highlight_effect.of([
                            head_dec.range(head_ptr, head_ptr + 1)
                        ]))

                        if (scan_ptr > head_ptr) {
                            effects.push(highlight_effect.of([
                                scan_dec.range(scan_ptr, scan_ptr + 1)
                            ]))
                        }
                        view.dispatch({ effects })
                    } break outer;
                }

                debugger_offset++;
            }
        }
    }

    function toggle_play(force_stop: boolean = false) {
        if (play_interval >= 0 || force_stop) {
            clearInterval(play_interval);
            play_interval = -1;
        } else if (PARSING) {
            play_interval = setInterval(step_forward, 1);
        }
    }

    debugger_step_button.addEventListener("click", step_forward);

    debugger_into_button.addEventListener("click", e => toggle_play());

    debugger_start_stop_button.addEventListener("click", e => {
        if (PARSING)
            stop_parser();
        else
            start_parser();
    });

    stop_parser();

    return [
        ViewPlugin.fromClass(class {
            update(update: ViewUpdate) {
                if (update.transactions.find(e => e.isUserEvent("debugger.start"))) {
                    console.log("Started");
                } else if (update.docChanged) {
                    stop_parser();
                }
            }
        }, {}),
        EditorView.updateListener.of(function (e) {
            view = e.view;
        }),
        StateField.define({
            create() { return Decoration.none },
            update(value, tr) {
                value = value.map(tr.changes)

                for (let effect of tr.effects) {
                    if (effect.is(highlight_effect)) value = value.update({ add: effect.value, sort: true });
                    else if (effect.is(filter_effects)) value = value.update({ filter: effect.value });
                }

                return value
            },
            provide(f) { return EditorView.decorations.from(f) }
        })
    ]
}