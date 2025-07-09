// worker.js

import radlr_init, * as radlr from "js/radlr/radlr_wasm";
import { RadlrError } from "./error";
import { LabEngineEvents } from "./lab_client"

let grammar_db: radlr.JSGrammarDB | null = null;
let states: radlr.JSIRParser | null = null;

function emit<T extends keyof LabEngineEvents, A = LabEngineEvents[T]>(type: T, val: A, is_transferable: boolean = false) {
  if (is_transferable) {
    postMessage({ lab_event: { type, val } }, { transfer: [<any>val] });
  } else {
    postMessage({ lab_event: { type, val } });
  }
}

// Handle incoming messages
self.addEventListener('message', async function (event) {

  const { type, eventData, eventId } = event.data;

  if (type === "init") {
    try {
      await radlr_init()
    } catch (e) {
      console.error(e);
    }

    postMessage({ type: "ready" });

  } else if (type === "compile_grammar") {

    let { grammar, config } = <{ grammar: string, config: any }>eventData;

    let soup = null, cfg: radlr.JSParserConfig | null = null;

    try {

      if (grammar_db) {
        grammar_db.free(); grammar_db = null;
      }

      if (states) {
        states.free(); states = null;
      }

      cfg = radlr.JSParserConfig.import(config);

      let optimize = true;

      soup = radlr.create_soup();
      soup.add_grammar(grammar, "main");
      grammar_db = radlr.create_grammar_db("main", soup, cfg);

      states = radlr.create_parser_states(grammar_db, optimize, cfg);

      emit("parser_classification", states.classification.to_string());

      let bytecode_db_export = radlr.export_bytecode_db(states);

      emit("parser_bytecode_db", bytecode_db_export, true);

    } catch (e) {
      if (e instanceof radlr.PositionedErrors) {
        let l = e.length;
        let error;
        let errors: RadlrError[] = [];

        for (let i = 0; i < l; i++) {
          if (error = new RadlrError(e.get_error_at(i))) {
            errors.push(error)
          }
        }

        e.free();

        emit("compile_errors", errors);
      } else {
        console.error(e);
        throw e;
      }
    } finally {
      try {

        if (grammar_db) {
          grammar_db.free(); grammar_db = null;
        }

        if (states) {
          states.free(); states = null;
        }

        if (soup) {
          soup.free();
        }

        if (cfg) {
          cfg.free();
        }
      } catch (e) {
        console.error(e)
      }
    }
  }

}, false);

// Test -- Conne