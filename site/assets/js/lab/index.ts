import "./settings-panel";
import "./config-panel";
import * as pipeline from "./pipeline";
import radlr_init, * as radlr from "js/radlr/radlr_wasm.js";
import { NB, NBContentField, NBEditorField, NBField } from "./notebook";
import { Controls } from "./control";
import { LocalStoreKeys, getLocalValue, dataStorageWorkflowsEnabled, setLocalValue } from "./settings-panel";
import { setupConfig } from "./config-panel";
import { Debounce } from "./debounce";
import * as parse_info from "./parser_info";
import * as syntax_highlight from "./syntax_highlight"
import * as ast from "./ast"
import * as cst from "./cst"
import init_help from "./help_docs";


export async function init(compiler_worker_path: string) {
  let rad_init = radlr_init();

  let nb = new NB(2);

  let grammar_input_field = nb.add_field(new NBEditorField("Grammar"), 0)
  grammar_input_field.set_content_visible(true);
  grammar_input_field.set_text("");
  grammar_input_field.set_icon(`<pre><></pre>`);
  grammar_input_field.set_help_doc_path("./docs/lab/grammar_panel");

  let parser_info_field = nb.add_field(new NBContentField("Parser Info"), -1);
  parser_info_field.set_content_visible(false);
  parser_info_field.set_icon(`<i class="fa-solid fa-circle-info"></i>`);
  parser_info_field.set_help_doc_path("./docs/lab/parser_info_panel");

  pipeline.GrammarDBNode.worker_path = compiler_worker_path;

  let parser_input_field = nb.add_field(new NBEditorField("Parser Input"), 0);
  parser_input_field.set_content_visible(true)
  parser_input_field.set_icon(`<i class="fa-solid fa-quote-left"></i>`);
  parser_input_field.set_help_doc_path("./docs/lab/parser_input_panel");

  let ast_field = nb.add_field(new NBContentField("AST Nodes"), -1);
  ast_field.set_icon(`<i class="fa-solid fa-share-nodes"></i>`);
  ast_field.set_help_doc_path("./docs/lab/ast_panel");

  let cst_field = nb.add_field(new NBContentField("CST Nodes"), 1, 1);
  cst_field.set_icon(`<i class="fa-solid fa-sitemap"></i>`);
  cst_field.set_help_doc_path("./docs/lab/cst_panel");

  let syntax_highlighting_field = nb.add_field(new NBContentField("Syntax Highlighting"), -1);
  syntax_highlighting_field.set_content_visible(true)
  syntax_highlighting_field.set_icon(`<i class="fa-solid fa-palette"></i>`);
  syntax_highlighting_field.set_help_doc_path("./docs/lab/syntax_highlighting_panel");


  let formatting_rules_field = nb.add_field(new NBContentField("Syntax Formatting"), -1);
  formatting_rules_field.set_content_visible(true)
  formatting_rules_field.set_icon(`<i class="fa-solid fa-align-right"></i>`);
  formatting_rules_field.set_help_doc_path("./docs/lab/formatting_rules_panel");


  let help_panel_field = nb.add_field(new NBContentField("Help"), 1, 0)
  help_panel_field.set_content_visible(true);
  help_panel_field.set_icon(`<pre>?</pre>`);
  help_panel_field.set_help_doc_path("./docs/lab");




  const controls = new Controls();

  const grammar_input = new pipeline.InputNode();
  const parser_input = new pipeline.InputNode();
  const config_input = new pipeline.ConfigNode();
  const grammar_pipeline_node = new pipeline.GrammarDBNode([grammar_input, config_input],);
  const parser_player_node = new pipeline.ParserPlayerNode([grammar_pipeline_node, parser_input]);

  let grammar_input_debounce = new Debounce(() => {
    let text = grammar_input_field.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.GrammarInput, text);
      grammar_input.update(text)
      grammar_input_field.remove_highlights();
      grammar_input_field.remove_messages();
    }
  });

  let parser_input_debounce = new Debounce(() => {
    let text = parser_input_field.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.ParserInput, text);
      parser_input.update(text)
      parser_input_field.remove_messages();
    }
  });

  grammar_input_field.addListener("text_changed", grammar_input => {
    grammar_input_debounce.call();
  })

  parser_input_field.addListener("text_changed", grammar_input => {
    parser_input_debounce.call();
  })

  parser_input_field.addListener("text_changed", parser_input => {
    let text = parser_input.get_text();
    if (text) {
      setLocalValue(LocalStoreKeys.ParserInput, text);
      //parser.restart(text);
    }
  })

  let error_reporter = <HTMLDivElement>document.getElementById("error-reporter");
  let grammar_classification = <HTMLDivElement>document.getElementById("controls")?.querySelector(".classification");

  grammar_pipeline_node.addListener("loading", _ => {
    error_reporter.innerText = "";
    grammar_classification.innerHTML = "...";
  })

  grammar_pipeline_node.addListener("failed", errors => {
    grammar_classification.innerHTML = "error";
    for (const error of errors) {
      if (error.origin == radlr.ErrorOrigin.Grammar) {
        error_reporter.innerText = (`Unhandled Error: \n${radlr.ErrorOrigin[error.origin]}\n${error.msg}\n${error.line}:${error.col}`);
        if (error.start_offset >= grammar_input_field.get_text().length) {
          grammar_input_field.add_highlight(error.start_offset - 1, error.end_offset, "red");
          grammar_input_field.add_message(error.start_offset - 1, error.end_offset, "Unexpected end of grammar");
        } else {
          grammar_input_field.add_highlight(error.start_offset, error.end_offset, "red");
          grammar_input_field.add_message(error.start_offset, error.end_offset, error.msg);
        }
      }
    }
    parser_info_field.set_content_visible(false);
    parser_info_field.set_loading(false);
    controls.setActive(false);
  })

  grammar_pipeline_node.addListener("parser-classification", classification => {
    grammar_classification.innerHTML = classification;
    controls.setActive(true);
  })

  parser_player_node.addListener("destroyed", data => {
    // Disable transport controls
    console.log("Parser Destroyed");
    controls.setActive(false);
  })

  parser_player_node.addListener("reset", data => {
    // Enable transport controls
    console.log("Parser Reset");
    controls.setActive(true);
  })

  parser_player_node.addListener("complete", action => {
    console.log("complete")
  });

  parser_player_node.addListener("eof", action => {
    console.log("eof")
  });

  parser_player_node.addListener("error", action => {
    console.log("error")
  });

  controls.addListener("step", () => {
    parser_player_node.step();
  });

  controls.addListener("play", () => {
    console.log("playing parser");
  });

  await pipeline.sleep(10);

  nb.calculate_heights();

  await rad_init;

  parse_info.init(parser_info_field, parser_input_field, grammar_pipeline_node, parser_player_node, controls);

  cst.init(cst_field, parser_input_field, grammar_pipeline_node);
  ast.init(ast_field, parser_input_field, grammar_pipeline_node);


  syntax_highlight.init(syntax_highlighting_field, parser_input_field, grammar_pipeline_node)

  let { grammar: example_grammar, input: example_input } = grammar_examples[Math.round(Math.random() * (grammar_examples.length - 1))];

  var text = getLocalValue(LocalStoreKeys.ParserInput) || example_input;
  parser_input_field.set_text(text);


  var text = getLocalValue(LocalStoreKeys.GrammarInput) || example_grammar;
  grammar_input_field.set_text(text);
  grammar_input.update(text)

  setupConfig(config => {
    config_input.update(config);
  });

  setTimeout(() => {
    document.getElementById("loading-screen")!.classList.remove("active");
  }, 500);


  init_help(nb, help_panel_field, grammar_input_field, parser_input_field, [
    <NBContentField><any>grammar_input_field,
    parser_info_field,
    <NBContentField><any>parser_input_field,
    ast_field,
    cst_field,
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