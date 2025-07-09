import "./settings-modal";
import "../panels/config-panel";
import * as pipeline from "./pipeline";
import radlr_init, * as radlr from "js/radlr/radlr_wasm.js";
import { NB, NBContentField, NBEditorField, NBField } from "./notebook";
import { Controls } from "./control";
import { LocalStoreKeys, getLocalValue, dataStorageWorkflowsEnabled, setLocalValue } from "./settings-modal";
import { setupConfig } from "../panels/config-panel";
import { Debounce } from "./debounce";
import * as parse_info from "../panels/parser_info";
import * as syntax_highlight from "../panels/syntax_highlight"
import * as ast from "../panels/ast"
import * as cst from "../panels/cst"
import init_help from "../panels/help_docs";
import { GlobalParser } from "./parser";


export async function init(compiler_worker_path: string) {
  let rad_init = radlr_init();

  let nb = new NB(2);

  let { grammar_panel, parser_input_panel, parser_info_panel, cst_panel, ast_panel, syntax_highlighting_field, help_panel_field, formatting_rules_field } = initPanels(nb, compiler_worker_path);

  const controls = new Controls();
  const grammar_input = new pipeline.InputNode();
  const parser_input = new pipeline.InputNode();
  const config_input = new pipeline.ConfigNode();
  const grammar_pipeline_node = new pipeline.GrammarDBNode([grammar_input, config_input],);
  //const parser_player_node = new pipeline.ParserPlayerNode([grammar_pipeline_node, parser_input]);

  initGrammarPanel(grammar_panel, grammar_input, parser_input_panel, parser_input, grammar_pipeline_node, parser_info_panel, controls);

  initTransportControls(controls);

  await pipeline.sleep(10);

  nb.calculate_heights();

  await rad_init;

  GlobalParser.initPipeline(parser_input_panel, grammar_pipeline_node);

  cst.init(cst_panel, parser_input_panel, grammar_pipeline_node, GlobalParser);

  ast.init(ast_panel, parser_input_panel, grammar_pipeline_node);

  parse_info.init(parser_info_panel, parser_input_panel, grammar_pipeline_node, GlobalParser);

  syntax_highlight.init(syntax_highlighting_field, parser_input_panel, grammar_pipeline_node)

  let { grammar: example_grammar, input: example_input } = grammar_examples[Math.round(Math.random() * (grammar_examples.length - 1))];

  var text = getLocalValue(LocalStoreKeys.ParserInput) || example_input;

  parser_input_panel.set_text(text);


  var text = getLocalValue(LocalStoreKeys.GrammarInput) || example_grammar;
  grammar_panel.set_text(text);
  grammar_input.update(text)

  setupConfig(config => {
    config_input.update(config);
  });

  setTimeout(() => {
    document.getElementById("loading-screen")!.classList.remove("active");
  }, 500);


  init_help(nb, help_panel_field, grammar_panel, parser_input_panel, [
    <NBContentField><any>grammar_panel,
    parser_info_panel,
    <NBContentField><any>parser_input_panel,
    ast_panel,
    cst_panel,
    syntax_highlighting_field,
    formatting_rules_field,
    help_panel_field
  ]);
}

const grammar_examples = [
  {
    grammar:
      /**/
      `IGNORE { c:sp }  
    
<> entry > "Hello" "World"
  `.trim(),
    input: "Hello World"
  },
  {
    grammar:
      /**/
      `IGNORE {c:sp c:nl}

<> json 

  > entry                 :ast { t_JSON, body: $1, tok }

<> entry > obj | array
  
<> obj 
  
  > "{" key_val(*",") "}" :ast { t_Object, values: $2, tok }

<> array
  
  > "[" val(*",") "]"     :ast { t_Array, values: $2, tok }


<> key_val 

  > key ":" val           :ast map($1, $3)


<> key 

  > tk:string             :ast str(tok<1,1>)


<> val 
  > tk:string :ast str(tok<1,1>)
  | tk:( c:num(+) )     :ast f64($1)
  | obj
  | array
  | "true"    :ast bool($1)
  | "false"   :ast bool
  | "null"    :ast {t_Null}


<> string > "\\"" ( c:id | c:sym | c:num | c:sp | c:nl | escaped )(*) "\\""

<> escaped > "\\\\"{:9999} ( c:id | c:sym | c:num | c:sp | c:nl )
      
       
      
  `,
    input: `{ "hello" : "world" }`
  }
]

function initPanels(nb: NB, compiler_worker_path: string) {
  let grammar_panel = nb.add_field(new NBEditorField("Grammar"), 0);
  grammar_panel.set_content_hidden(false);
  grammar_panel.set_text("");
  grammar_panel.set_icon(`<pre><></pre>`);
  grammar_panel.set_help_doc_path("./docs/lab/grammar_panel");

  let parser_info_panel = nb.add_field(new NBContentField("Parser Info"), -1);
  parser_info_panel.set_content_hidden(true);
  parser_info_panel.set_icon(`<i class="fa-solid fa-circle-info"></i>`);
  parser_info_panel.set_help_doc_path("./docs/lab/parser_info_panel");

  pipeline.GrammarDBNode.worker_path = compiler_worker_path;

  let parser_input_panel = nb.add_field(new NBEditorField("Parser Input"), 0);
  parser_input_panel.set_content_hidden(false);
  parser_input_panel.set_icon(`<i class="fa-solid fa-quote-left"></i>`);
  parser_input_panel.set_help_doc_path("./docs/lab/parser_input_panel");

  let ast_panel = nb.add_field(new NBContentField("AST Nodes"), -1);
  ast_panel.set_icon(`<i class="fa-solid fa-share-nodes"></i>`);
  ast_panel.set_help_doc_path("./docs/lab/ast_panel");

  let cst_panel = nb.add_field(new NBContentField("CST Nodes"), 1, 1);
  cst_panel.set_icon(`<i class="fa-solid fa-sitemap"></i>`);
  cst_panel.set_help_doc_path("./docs/lab/cst_panel");

  let syntax_highlighting_field = nb.add_field(new NBContentField("Syntax Highlighting"), -1);
  syntax_highlighting_field.set_content_hidden(true);
  syntax_highlighting_field.set_icon(`<i class="fa-solid fa-palette"></i>`);
  syntax_highlighting_field.set_help_doc_path("./docs/lab/syntax_highlighting_panel");


  let formatting_rules_field = nb.add_field(new NBContentField("Syntax Formatting"), -1);
  formatting_rules_field.set_content_hidden(false);
  formatting_rules_field.set_icon(`<i class="fa-solid fa-align-right"></i>`);
  formatting_rules_field.set_help_doc_path("./docs/lab/formatting_rules_panel");

  let help_panel_field = nb.add_field(new NBContentField("Help"), 1, 0);
  help_panel_field.set_content_hidden(false);
  help_panel_field.set_icon(`<pre>?</pre>`);
  help_panel_field.set_help_doc_path("./docs/lab");

  return { grammar_panel, parser_input_panel, parser_info_panel, cst_panel, ast_panel, syntax_highlighting_field, help_panel_field, formatting_rules_field };
}

function initTransportControls(controls: Controls) {
  controls.setActive(true);

  controls.addListener("step", () => {
    GlobalParser.step_instruction();
  });

  controls.addListener("jump", () => {
    GlobalParser.step_state();
  });
  
  controls.addListener("play", () => {   
    GlobalParser.play_till_complete();
  });

  controls.addListener("stop", () => {
    GlobalParser.play_till_complete();
  });

  controls.addListener("reset", () => {
    GlobalParser.restart();
  });
}

function initGrammarPanel(grammar_panel: NBEditorField, grammar_input: pipeline.InputNode, parser_input_field: NBEditorField, parser_input: pipeline.InputNode, grammar_pipeline_node: pipeline.GrammarDBNode, parser_info_field: NBContentField<null, "">, controls: Controls) {
  let grammar_input_debounce = new Debounce(() => {
    let text = grammar_panel.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.GrammarInput, text);
      grammar_input.update(text);
      grammar_panel.remove_highlights();
      grammar_panel.remove_messages();
    }
  });

  let parser_input_debounce = new Debounce(() => {
    let text = parser_input_field.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.ParserInput, text);
      parser_input.update(text);
      parser_input_field.remove_messages();
    }
  });

  grammar_panel.addListener("text_changed", grammar_input => {
    grammar_input_debounce.call();
  });

  parser_input_field.addListener("text_changed", grammar_input => {
    parser_input_debounce.call();
  });

  parser_input_field.addListener("text_changed", parser_input => {
    let text = parser_input.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.ParserInput, text);
      //parser.restart(text);
    }
  });

  let error_reporter = <HTMLDivElement>document.getElementById("error-reporter");
  let grammar_classification = <HTMLDivElement>document.getElementById("controls")?.querySelector(".classification");

  grammar_pipeline_node.addListener("loading", _ => {
    grammar_panel.set_info_border("#2244AA");
    grammar_panel.set_loading(true);
    error_reporter.innerText = "";
    grammar_classification.innerHTML = "...";
  });

  grammar_pipeline_node.addListener("failed", errors => {
    grammar_panel.set_info_border("#FF0000");
    grammar_panel.set_loading(false);
    grammar_classification.innerHTML = "error";
    for (const error of errors) {
      if (error.origin == radlr.ErrorOrigin.Grammar) {
        error_reporter.innerText = (`Unhandled Error: \n${radlr.ErrorOrigin[error.origin]}\n${error.msg}\n${error.line}:${error.col}`);
        if (error.start_offset >= grammar_panel.get_text().length) {
          grammar_panel.add_highlight(error.start_offset - 1, error.end_offset, "red");
          grammar_panel.add_message(error.start_offset - 1, error.end_offset, "Unexpected end of grammar");
        } else {
          grammar_panel.add_highlight(error.start_offset, error.end_offset, "red");
          grammar_panel.add_message(error.start_offset, error.end_offset, error.msg);
        }
      }
    }
    parser_info_field.set_content_hidden(false);
    parser_info_field.set_loading(false);
    controls.setActive(false);
  });

  grammar_pipeline_node.addListener("parser-classification", classification => {
    grammar_panel.set_info_border();
    grammar_panel.set_loading(false);
    grammar_classification.innerHTML = classification;
    controls.setActive(true);
  });
}
