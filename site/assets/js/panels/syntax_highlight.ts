import { JSBytecodeParserDB } from "js/radlr/radlr_wasm";
import { NBContentField, NBEditorField } from "../lab/notebook";
import { Parser } from "../lab/parser";
import { GrammarDBNode, InputNode } from "../lab/pipeline";

export function init(
  syntax_field: NBContentField,
  parser_input_field: NBEditorField,
  grammar_pipeline: GrammarDBNode,
) {
  let input_string: string = "";
  let db: null | JSBytecodeParserDB = null;

  parser_input_field.addListener("text_changed", field => {
    input_string = field.get_text();
    setTimeout(() => run_highlighting(parser_input_field, input_string, db), 10);
  })

  grammar_pipeline.addListener("loading", _ => {
    db = null;
    syntax_field.set_loading(true)
  });

  grammar_pipeline.addListener("failed", _ => {
    syntax_field.set_loading(false);
  })

  grammar_pipeline.addListener("bytecode_db", new_db => {
    db = new_db;

    syntax_field.set_loading(false);

    run_highlighting(parser_input_field, input_string, db);
  });
}

function run_highlighting(input_field: NBEditorField, input: string, db: JSBytecodeParserDB | null) {

  if (!input || !db) return;

  let parser = new Parser(db, input);

  input_field.remove_highlights();

  let symbols: { start: number, end: number }[] = [];

  let args: [number, number, string][] = [];


  parser.on_shift = shift_data => {
    let start = shift_data.byte_offset;
    let end = shift_data.byte_offset + shift_data.byte_len;
    args.push([start, end, "green"]);
  };

  parser.on_complete = parser.on_error = () => {
    let last = symbols.pop()!;
    if (last && last.end < input.length) {
      input_field.add_highlight(last.end, input.length, "red");
      //input_field.add_message(last.end, input.length - 1, "Unparsed content")
    }
  }

  parser.init("default", input);
  parser.play();

  parser.on_reduce = null;
  parser.on_shift = null;

  parser.destroy();


  input_field.add_highlights(...args);
}