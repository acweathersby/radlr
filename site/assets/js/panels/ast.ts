import { JSBytecodeParserDB, get_nonterminal_name_from_id } from "js/radlr/radlr_wasm";
import { NBContentField, NBEditorField } from "../lab/notebook";
import { Parser } from "../lab/parser";
import { GrammarDBNode, InputNode } from "../lab/pipeline";
import * as radlr from "js/radlr/radlr_wasm";
import { RadlrError } from "../lab/error";
import { SyntaxGraphEngine } from "js/graph_sys";


export function init(
  ast_field: NBContentField,
  parser_input_field: NBEditorField,
  grammar_pipeline: GrammarDBNode,
) {

  let pending = false;
  let engine = new SyntaxGraphEngine(<HTMLDivElement>ast_field.body);
  let input_string: string = "";
  let db: null | JSBytecodeParserDB = null;
  let ast: Rules | null;


  grammar_pipeline.addListener("grammar_db", db => {
    try {
      ast = JSON.parse(db.get_ast_generating_script());
    } catch (e) {
      ast = null;
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

        console.log(errors);

      } else {
        console.error(e);
        throw e;
      }
    }
  });

  function build_ast() {
    if (!pending) {
      pending = true;
      setTimeout(() => {
        pending = false;
        if (ast && db)
          run_cst_render(ast, input_string, db, engine)
      }, 100)
    }
  }

  parser_input_field.addListener("text_changed", field => {
    input_string = field.get_text();

    if (!ast_field.is_mini)
      build_ast();
  })

  grammar_pipeline.addListener("loading", _ => {
    db = null;
    ast_field.set_loading(true)
  });

  grammar_pipeline.addListener("failed", _ => {
    ast_field.set_loading(false);
  })

  grammar_pipeline.addListener("bytecode_db", new_db => {
    db = new_db;
    ast_field.set_loading(false);

    if (!ast_field.is_mini)
      build_ast()
  });
}

enum ExpressionType {
  Null,
  Token,
  Array,
  Map,

}

class Token { id: number = 0; off: number = 0; len: number = 0 };
type Struct = {
  name: string,
  params: { [k: string]: Node }
}

type Expr = ["tok"]
  | ["tok", number]
  | ["sym", number]
  | ["str", Expr]
  | ["add", Expr, Expr]
  | ["mul", Expr, Expr]
  | ["trim", Expr, number, number]
  | ["vec", ...Expr[]]
  | ["num", Expr]
  | ["map", Expr, Expr]
  | ["merge", string, Expr];


type StructRule = {
  name: string,
  params: { [k: string]: Expr }
}

type Rules = {
  [k: number]: StructRule | Expr
}

type Node = { name: string, params: { [k: string]: Node } } | string | number | Node[] | Map<string, Node> | null | Token


function run_cst_render(ast: Rules, input: string, db: JSBytecodeParserDB | null, renderer: SyntaxGraphEngine) {
  renderer.clear_nodes();
  renderer.flip_nodes();

  if (!input)
    renderer.draw();

  if (!input || !db) return;

  let parser = new Parser(db, input);

  console.log(parser.generate_error_corrected_output(input))

  let tokens: Token[] = [];
  let nodes: Node[] = [];

  let id = 0;

  function process_expression(expr: Expr, offset: number, len: number, root_token: Token): Node {
    if (!expr) {
      return null;
    }
    switch (expr[0]) {
      case "vec":
        let vec = [];
        for (const expr_ of expr.slice(1)) {
          vec.push(process_expression(<Expr>expr_, offset, len, root_token))
        }
        return vec;
      case "mul": {
        let node_a = process_expression(expr[1], offset, len, root_token);
        let node_b = process_expression(expr[2], offset, len, root_token);
        //@ts-ignore
        return node_a * node_b
      }
      case "merge": {
        return process_expression(expr[2], offset, len, root_token)
      }
      case "map": {
        let key = process_expression(expr[1], offset, len, root_token);
        let val = process_expression(expr[2], offset, len, root_token);
        return new Map([[key + "", val]])
      }
      case "add":
        let node_a = process_expression(expr[1], offset, len, root_token);
        let node_b = process_expression(expr[2], offset, len, root_token);

        let is_array = (+Array.isArray(node_a)) << 1 | (+Array.isArray(node_b));
        let is_map = (+(node_a instanceof Map)) << 1 | (+(node_b instanceof Map));

        if (is_array == 3) {
          //@ts-ignore
          let node = node_a.slice()
          //@ts-ignore
          node.push(...node_b);
          return node
        } else if (is_map == 3) {
          return new Map([...node_a, ...node_b]);


        } else if (is_map == 0) {
          if (is_array == 1) {
            //@ts-ignore
            let node = node_b.slice();
            node.unshift(node_a);
            return node;
          } else if (is_array == 2) {
            //@ts-ignore
            let node = node_a.slice()
            node.push(node_b);
            return node;
          }
        } else {
          throw "Invalid combination"
        }
        //@ts-ignore
        return node_a + node_b
      case "num": {
        let node_a = process_expression(expr[1], offset, len, root_token);
        if (node_a instanceof Token) {
          let string = input.slice(node_a.off, node_a.off + node_a.len);
          return parseFloat(string);
        }
        //@ts-ignore
        return parseFloat(node_a);
      } break
      case "trim": {
        let [_, e, left, right] = expr;
        let node_a = process_expression(e, offset, len, root_token);
        if (node_a instanceof Token) {
          node_a.len -= right + left;
          node_a.off += left;
          return node_a
        }
      }
      case "str": {
        let node_a = process_expression(expr[1], offset, len, root_token);
        if (node_a instanceof Token) {
          return input.slice(node_a.off, node_a.off + node_a.len);
        } else {
          return node_a?.toString() ?? ""
        }
      } break
      case "sym":
        return nodes[offset + expr[1]]
      case "tok":
        if (typeof expr[1] == "number") {
          return tokens[offset + expr[1]];
        } else {
          return root_token;
        }
        break
      default:
        console.log(expr);
        return null
    }
  }

  parser.on_reduce = reduce_data => {
    let rule_id = reduce_data.rule_id;
    let len = reduce_data.symbols;
    let offset = nodes.length - len;

    let rule = ast[rule_id];

    let tok_a = tokens[offset];
    let tok_b = tokens[offset + len - 1];
    let root_token = new Token;
    root_token.off = tok_a.off;
    root_token.len = (tok_b.off - tok_a.off) + tok_b.len;

    if (Array.isArray(rule)) {
      nodes.splice(offset, len, process_expression(rule, offset, len, root_token));
    } else {
      let name = rule.name;
      let struct = <Struct>{ name, params: {} }

      for (const param in rule.params) {
        struct.params[param] = process_expression(rule.params[param], offset, len, root_token);
      }

      nodes.splice(offset, len, struct);
    }

    tokens.splice(offset, len, root_token);
  };

  parser.on_shift = shift_data => {
    let token = new Token;
    token.id = shift_data.token_id
    token.off = shift_data.byte_offset;
    token.len = shift_data.byte_len

    tokens.push(token);
    nodes.push(null);
    id++;
  };

  parser.on_error = error_data => {
  }


  parser.init("default", input);
  parser.play();


  parser.on_reduce = null;
  parser.on_shift = null;

  parser.destroy();

  function draw_node(node: Node, off_x: number = 0, off_y: number = 0, scale = 1): { id: number, x: number, y: number } {
    switch (typeof node) {
      case "string": {
        let id = renderer.addNode(off_x, off_y, node + "", [123, 120, 255]);;
        return { id, x: off_x, y: off_y };
      }
      case "number": {
        let id = renderer.addNode(off_x, off_y, node + "", [123, 120, 255]);;
        return { id, x: off_x, y: off_y };
      }
      case "object":
        if (node === null) {
          let id = renderer.addNode(off_x, off_y, "null", [123, 120, 255]);;
          return { id, x: off_x, y: off_y };
        } else if (node instanceof Token) {
          let id = renderer.addNode(off_x, off_y, `tok[${node.off}:${node.len}]`, [170, 200, 100]);;
          return { id, x: off_x, y: off_y };
        } else if (Array.isArray(node)) {
          let par_id = renderer.addNode(off_x, off_y, "[]", [180, 200, 255]);
          off_y -= 1 * scale;
          for (let i = 0; i < node.length; i++) {
            off_y -= 1 * scale;

            let c_id = renderer.addNode(off_x, off_y, i + "", [123, 120, 255]);
            renderer.add_connection(par_id, c_id);

            let { x, y, id } = draw_node(node[i], off_x + 1 * scale, off_y, scale);

            off_y = y;

            renderer.add_connection(id, c_id);
          }
          return { id: par_id, x: off_x, y: off_y };
        } else if (node instanceof Map) {
          let par_id = renderer.addNode(off_x, off_y, "Map", [123, 120, 255]);
          for (const [name, val] of node.entries()) {
            off_y -= 1 * scale;

            let c_id = renderer.addNode(off_x, off_y, name, [123, 120, 255]);
            renderer.add_connection(par_id, c_id);

            let { x, y, id } = draw_node(val, off_x + 1 * scale, off_y, scale);

            off_y = y;

            renderer.add_connection(id, c_id);
          }
          return { id: par_id, x: off_x, y: off_y };
        } else {
          let name = node.name;
          let par_id = renderer.addNode(off_x, off_y, name, [200, 0, 20]);
          for (const name in node.params) {
            off_y -= 1 * scale;
            let c_node = node.params[name];

            let c_id = renderer.addNode(off_x, off_y, name, [123, 0, 20]);
            renderer.add_connection(par_id, c_id);

            let { x, y, id } = draw_node(c_node, off_x + 1 * scale, off_y, scale);

            off_y = y;

            renderer.add_connection(id, c_id);
          }

          return { id: par_id, x: off_x, y: off_y };
        }
    }
  }


  let x = 0;
  for (const node of nodes) {
    let { x: d } = draw_node(node, x, 0, 400);
    x = d + 400
  }

  renderer.update();
  renderer.draw();
}



