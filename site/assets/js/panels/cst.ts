import { get_nonterminal_name_from_id } from "js/radlr/radlr_wasm";
import { NBContentField, NBEditorField } from "../lab/notebook";
import { ParserPlayer } from "../lab/parser";
import { GrammarDBNode } from "../lab/pipeline";
import { SyntaxGraphEngine } from "js/graph_sys";

export function init(
  cst_field: NBContentField,
  parser_input_field: NBEditorField,
  grammar_pipeline: GrammarDBNode,
  parser_player: ParserPlayer
) {

  let renderer = new SyntaxGraphEngine(<HTMLDivElement>cst_field.body);
  let input: string = "";
  type Node = { id: number, nodes: Node[], width: number, from: number, to: number };
  let SHOULD_RENDER = false;
  let symbols: Node[] = [];

  grammar_pipeline.addListener("loading", _ => {
    cst_field.set_loading(true)
  });

  grammar_pipeline.addListener("failed", _ => {
    cst_field.set_loading(false);
  })

  parser_input_field.addListener("text_changed", field => {
    input = field.get_text();
  })

  grammar_pipeline.addListener("bytecode_db", _ => {
    cst_field.set_loading(false);
  });

  parser_player.addListener("reduce", reduce_data=> {

    let offset = symbols.length - reduce_data.symbols;
    let r_syms = symbols.splice(offset, reduce_data.symbols);

    let name = get_nonterminal_name_from_id(reduce_data.non_terminal_id, reduce_data.db);

    let from = r_syms[0].from;
    let to = r_syms[r_syms.length - 1].to;
    let id = renderer.addNode(0, 0, name);
    let width = 0;
    for (const sym of r_syms) {
      renderer.add_connection(id, sym.id);
      width += sym.width;
    }

    symbols.push({ id, nodes: r_syms, width, from, to });
    id++;

    scheduleRender();
  });

  parser_player.addListener("reset", ({interactive}) => {
    if(interactive) {
      symbols.length = 0;
      renderer.clear_nodes();
      renderer.clear_node_buffers();
      renderer.update();
      renderer.draw();
    } else {
      symbols.length = 0;
      renderer.clear_nodes();
    }
  })


  parser_player.addListener("shift", shift_data => {
    let id = renderer.addNode(0, 0, shift_data.token, [123, 120, 255]);
    symbols.push({ id, nodes: [], width: 1, from: shift_data.byte_offset, to: shift_data.byte_offset + shift_data.byte_len });
    id++;

    scheduleRender();
  });

  parser_player.addListener("error", error_data => {
    let end_token = input.slice(error_data.ctx.sym_ptr, error_data.ctx.sym_ptr + 5);

    let id = renderer.addNode(0, 0, `${end_token} +[${input.length - error_data.ctx.sym_ptr - end_token.length}]`);
    symbols.push({ id, nodes: [], width: 1, from: 0, to: 0 });


    scheduleRender();
  })


  function scheduleRender(timeout: number = 100) {
    if(!SHOULD_RENDER) {
      SHOULD_RENDER = true;
      setTimeout(render, timeout)
    }
  }

  let node_width = 1
  function map_positions(symbols: Node[], lvl: number = 0, positions: [number, number, number, number][], x: number = 0, offsets: number[][] = []): [number, number] {
    
    let desired_center = x - (symbols.length - 1) / 2;
    if (offsets.length < lvl + 1) {
      offsets.push([]);
    }
    let last = offsets[lvl];
    let start = last.length > 0 ? Math.max(desired_center, positions[last[last.length - 1]][0] + 1) : desired_center;
    let end = start;
    for (const p_node of symbols) {

      last.push(p_node.id);

      let [s, e] = map_positions(p_node.nodes, lvl + 1, positions, end, offsets);

      let center = s + (e - s) / 2;

      let error = (end - center);

      if (error > 0) {
        for (let i = lvl + 1; i < offsets.length; i++) {
          let last = -1;
          for (const id of offsets[i].slice()) {
            if (last > 0) {
              if (positions[id][0] <= last) {
                positions[id][0] += error
                last = positions[id][0];
              } else {
                continue;
              }
            } else {
              positions[id][0] += error
              last = positions[id][0];
            }
          }
        }
      } else { }

      positions[p_node.id] = [end, lvl * -1, p_node.from, p_node.to];

      // this nodes position is a function of its children positions

      end += node_width;
    }

    return [start, end - 1]
  }

  function render() {
    if (SHOULD_RENDER) { 
      renderer.flip_nodes();
      
      SHOULD_RENDER = false;
      
      if (cst_field.collapsed) {
        return;
      }

      let id = 0;
      let positions = new Array(id);
    
      map_positions(symbols, 0, positions, 0, []);
    
      for (let i = 0; i < positions.length; i++) {
        let [x, y] = positions[i];
        renderer.update_node(i, x * 800, y * 800);
      }
    
      renderer.update();
      renderer.draw();

      
    }
  }
}