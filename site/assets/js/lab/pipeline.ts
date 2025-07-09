import { JSDebugEvent } from "js/radlr/radlr_wasm";
import * as radlr from "js/radlr/radlr_wasm.js";
import { WasmBytecodeCompiler } from "js/lab/lab_client";
import { RadlrError } from "./error";

export async function sleep(time_in_ms: number) {
  return new Promise(function (res) {
    setTimeout(res, time_in_ms)
  })
}

class PipelineNode<event_names = any> {
  private enables: PipelineNode<any>[] = [];
  private enabled_by: PipelineNode<any>[] = [];
  protected ENABLED = false;
  listeners: Map<any, ((arg: any) => void)[]> = new Map

  constructor(enabled_by: PipelineNode<any>[] = []) {
    for (const e of enabled_by) {
      e.enables.push(<any>this);
    }
    this.enabled_by = enabled_by
  }

  protected enable() {
    this.ENABLED = true;
    this.signal();
  }

  protected disable() {
    this.ENABLED = false;
    this.end();
    this.signal();
  }

  protected async end() { }
  protected async start(data: object) { }
  protected async stop() { }

  private signal() {
    for (const enable of this.enables) {
      enable.pre_check();
    }
  }

  private async pre_check() {
    let data: any = {};
    for (const enabled_by of this.enabled_by) {
      if (enabled_by.ENABLED != true) {
        if (this.ENABLED == true) {
          this.ENABLED = false;
          this.signal();
          this.stop();
        }
        return;
      } else {
        data[enabled_by.name()] = enabled_by.data();
      }
    }

    await this.start(data);
  }


  protected data(): any { }

  protected name(): string { return ""; }

  protected emit<T extends keyof event_names, A = event_names[T], D = (arg: A) => void>(event: T, data: A) {
    for (const listener of this.listeners.get(event) ?? []) {
      (listener)(<any>data)
    }
  }

  protected haveListeners<T extends keyof event_names, A = event_names[T], D = (arg: A) => void>(event: T): boolean {
    return !!this.listeners.get(event)
  }

  addListener<T extends keyof event_names, A = event_names[T], D = (arg: A) => void>(event: T, listener: D) {
    if (!this.listeners.get(event)) {
      this.listeners.set(event, [<any>listener]);
    } else {
      this.listeners.get(event)?.push(<any>listener);
    }
  }

  removeListener<T extends keyof event_names, A = event_names[T], D = (arg: A) => void>(event: T, listener: D) {
    let listeners = this.listeners.get(event);
    if (listeners) {
      let index = listeners.findIndex(<any>listener);
      if (index > 0) {
        listeners.splice(index, 1)
      }
    }
  }
}

export class InputNode extends PipelineNode {
  input_string: string = "";

  protected name() { return "InputNode" }
  protected data() { return this.input_string; }

  update(grammar_string: string) {
    this.input_string = grammar_string;

    if (this.input_string) {
      this.enable();
    } else {
      this.disable();
    }
  }
}

export class ConfigNode extends PipelineNode {
  config: radlr.JSParserConfig | null = null;

  protected name() { return "ConfigNode" }
  protected data() { return this.config; }


  update(config: radlr.JSParserConfig) {
    if (this.config) {
      this.config.free()
      this.config = null;
    }

    this.config = config;

    this.enable();
  }
}

export class GrammarDBNode extends PipelineNode<{
  "loading": void
  "loaded": void
  "failed": RadlrError[],
  "grammar_db": radlr.JSGrammarDB,
  "bytecode_db": radlr.JSBytecodeParserDB,
  "bytecode_ready": string,
  "parser-classification": string
}> {
  static worker_path: string = ""

  grammar_string: string = ""

  parser_db: radlr.JSBytecodeParserDB | null = null;
  config: radlr.JSParserConfig | null = null;
  compile_nonce = 0
  compiler = new WasmBytecodeCompiler(GrammarDBNode.worker_path);

  DEDUP_ACTIVE = false

  protected name() { return "GrammarDB" }
  protected data() { return { parser_db: this.parser_db }; }

  constructor(...args: any[]) {
    super(...args);

    this.compiler.addListener("grammar_db", grammar => {
      this.emit("grammar_db", grammar);
      // Compile the parser
      if (this.config)
        this.compiler.build_parser(this.grammar_string, this.config);
    });

    this.compiler.addListener("compile_errors", errors => {
      console.error(errors);
      this.emit("failed", errors)
      this.disable()
    });

    this.compiler.addListener("parser_classification", classification => {
      this.emit("parser-classification", classification);
    })

    this.compiler.addListener("parser_bytecode_db", bytecode_db_export => {
      try {
        if (this.parser_db) { this.parser_db.free(); this.parser_db = null }

        this.parser_db = radlr.import_bytecode_db(bytecode_db_export);


        if (this.haveListeners("bytecode_ready")) {
          this.emit("bytecode_ready", radlr.create_bytecode_disassembly(this.parser_db));
        }

        if (this.haveListeners("bytecode_db")) {
          this.emit("bytecode_db", this.parser_db);
        }

        this.emit("loaded", void 0);

        this.enable()
      } catch (error) {
        console.error(error);
        this.emit("failed", void 0);
      }
    });
  }

  async load(compile_nonce: number, data: any) {
    if (compile_nonce != this.compile_nonce) {
      return;
    }

    this.emit("loading", void 0);

    this.grammar_string = data.InputNode;
    this.config = data.ConfigNode;

    if (this.config)
      this.compiler.build_grammar(this.grammar_string, this.config);

    return;
  }

  async start(data: any) {

    // Submit job for the compiler
    this.compile_nonce++;

    let compile_nonce = this.compile_nonce;

    await sleep(200);

    this.load(compile_nonce, data)
  }
}




