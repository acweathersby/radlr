import { JSParserConfig } from "js/radlr/radlr_wasm";
import { RadlrError } from "./error";
import { Eventable } from "./eventable";
import { LocalStoreKeys, dataStorageWorkflowsEnabled, getLocalValue } from "./settings-modal";
import * as radlr from "js/radlr/radlr_wasm";
import { sleep } from "./pipeline";

export type LabEngineEvents = {
  "grammar_db": radlr.JSGrammarDB,
  "parser_classification": string,
  "parser_bytecode_db": Uint8Array
  "compile_errors": RadlrError[]
}

abstract class LabEngine extends Eventable<LabEngineEvents> {
  grammar_db: radlr.JSGrammarDB | null = null

  /** 
     Compiles a parser_db from grammar string, emitting `parser_bytecode_db` if the parser compiled, or `compile_errors` 
     with relevant errors otherwise
  */
  abstract build_parser(grammar_string: string, config: JSParserConfig): void;

  /** 
   Compiles a parser_db from grammar string, emitting `grammar_db` if the parser compiled, or `compile_errors` 
   with relevant errors otherwise
  */
  async build_grammar(grammar: string, config: JSParserConfig) {

    let soup = radlr.create_soup();

    try {
      soup.add_grammar(grammar, "main");

      if (this.grammar_db) {
        this.grammar_db.free();
        this.grammar_db = null;
      }

      this.grammar_db = radlr.create_grammar_db("main", soup, config);
      
      if (soup) soup.free();

      this.emit("grammar_db", this.grammar_db);

    } catch (e) {

      
      if (e instanceof radlr.PositionedErrors) {
        let l = e.length;
        let error;
        let errors: RadlrError[] = [];

        for (let i = 0; i < l; i++) {
          if (error = new RadlrError(e.get_error_at(i))) {
            errors.push(error);
          }
        }

        e.free();

        this.emit("compile_errors", errors);

        if (soup) soup.free();
       
      } else {

        console.error(e);

        if (soup) soup.free();

        throw e;
      }
    }
  }

}

export class WasmBytecodeCompiler extends LabEngine {
  listeners: Map<any, ((arg: any) => void)[]> = new Map;
  worker: LabEngineWorkerClient
  client: LabEngineWebsocketClient | null = null

  constructor(worker_path: string) {
    super()
    this.worker = new LabEngineWorkerClient(worker_path);
    this.worker.extender = this;
  }

  // Attempts to connect to a localhost lab-server using the current port configuration
  async getClientConnection(): Promise<LabEngineWebsocketClient> {

    if (!dataStorageWorkflowsEnabled()) {
      if (this.client) {
        await this.client.disconnect();
        this.client = null;
      }
      throw "Lab host connection not available when data workflow is disabled";
    }

    let port = getLocalValue(LocalStoreKeys.LocalRADLRPort);

    if (!port) {
      port = "15421";
    }

    if (this.client) {
      if (this.client.port != port) {
        await this.client.disconnect();
        this.client = null;
      } else if (await this.client.is_connected()) {
        return this.client;
      }
    }

    let client = new LabEngineWebsocketClient(port);

    client.extender = this;

    if (await client.is_connected()) {
      return client
    } else {
      throw "Failed to connect to lab host";
    }
  }
  async build_parser(grammar: string, config: JSParserConfig) {
    try {
      let client = await this.getClientConnection();
      client.build_parser(grammar, config);
      return
    } catch (e) {
      console.error(e);
    }

    try {
      if (await this.worker.ready) {
        this.worker.build_parser(grammar, config)
      } else {
        throw "Could not establish a working connection"
      }
    } catch (e) {
      console.error(e);
    }
  }
}



class LabEngineWorkerClient extends LabEngine {
  worker: Worker
  _ready: Promise<boolean>

  constructor(worker_path: string) {
    super()

    const worker = new Worker(worker_path, { type: "module" });
    this.worker = worker;


    worker.addEventListener('message', <T extends keyof LabEngineEvents, A = LabEngineEvents[T]>(event: {
      data: {
        lab_event: {
          type: T,
          val: A
        }
      }
    } & MessageEvent) => {
      if (event.data.lab_event) {
        let l_event: {
          type: T,
          val: A
        } = event.data.lab_event;

        this.emit(l_event.type, l_event.val)
      } else {
        console.error("Lab Worker: Unhandled message - ", event.data);
      }
    });


    worker.addEventListener("error", function (error) {
      console.error(error)
    });

    this._ready = new Promise((res, rej) => {
      worker.addEventListener('message', function (event) {
        if (event.data.type == "ready") {
          res(true);
        }
      });
    });

    this._ready.catch(err => {
      console.error(err)
    })

    worker.postMessage({ type: "init", eventData: void 0 });
  }


  async ready(): Promise<boolean> {
    return this._ready
  }

  async build_parser(grammar: string, config: JSParserConfig) {
    this.worker.postMessage({ type: "compile_grammar", eventData: { grammar, config: config.export() }, })
  }
}

export class LabEngineWebsocketClient extends LabEngine {
  port = "15421";
  private static host = "localhost"
  private connected: Promise<boolean>
  private socket: WebSocket

  constructor(port = "15421", connection_timeout_ms: number = 200) {
    super()

    this.port = port;

    let socket = new WebSocket(`ws://${LabEngineWebsocketClient.host}:${this.port}`);

    this.socket = socket;

    this.connected = new Promise((res, rej) => {
      let timeout_id = setTimeout(() => {
        res(false);
        socket.close();
      }, connection_timeout_ms);

      socket.onopen = msg => {
        clearTimeout(timeout_id);

        socket.onerror = null;
        res(true);

        socket.addEventListener("close", msg => {
          this.connected = new Promise(res => res(false));
        })

        socket.addEventListener("err", msg => {
          console.error("WS Error", msg);
        })

        socket.addEventListener("message", async msg => {
          let data = <Blob>msg.data;
          let bytes = new Uint8Array(await data.arrayBuffer());

          let response_code = bytes[0];

          switch (response_code) {
            case radlr.WSResponseCodes.Classification: {
              let classification = radlr.JSParserClassification.deserialize(bytes.slice(1));
              this.emit("parser_classification", classification.to_string());
              classification.free();
            } break;
            case radlr.WSResponseCodes.ByteCode: {
              this.emit("parser_bytecode_db", bytes.slice(1));
            }
            default: {
              console.warn("Unrecognized message received");
            }
          }
        })
      }

      socket.onerror = msg => {
        rej(false)
      }


    });
  }

  async disconnect() {
    try {
      if (await this.connected) {
        this.socket.close();
      }
    } catch (e) { }
  }

  async is_connected(): Promise<boolean> {
    try {
      return await this.connected;
    } catch (e) {
      return false;
    }
  }

  async build_parser(grammar_string: string, config: JSParserConfig) {
    if (await this.connected) {
      let encoder = new TextEncoder;
      let min_len = (grammar_string.length * 2.5) | 0;
      let config_buffer = new Uint8Array(config.serialize());
      let config_buffer_size = config_buffer.byteLength;

      let data = new Uint8Array(8 + config_buffer_size + min_len);

      let transfer = new Uint8Array(data.buffer, 8, config_buffer_size);

      config_buffer.forEach((v, i) => transfer[i] = v)

      let { read, written } = encoder.encodeInto(grammar_string, data.subarray(8 + config_buffer_size));

      if (read < grammar_string.length) {
        throw "Did not read all grammar bytes";
      }

      let dv = new DataView(data.buffer);

      dv.setUint8(0, radlr.WSRequestCodes.BuildGrammar);
      dv.setUint32(4, written, true);

      let buffer = data.buffer.slice(0, 8 + config_buffer_size + written);

      this.socket.send(buffer);
    }
  }

  public static async ping(port: number, connection_timeout_ms: number = 200): Promise<boolean> {
    await sleep(200);
    return new Promise(
      res => {
        let timeout_id = setTimeout(() => {
          res(false);
          socket.close();
        }, connection_timeout_ms);

        let socket = new WebSocket(`ws://${LabEngineWebsocketClient.host}:${port}`);
        socket.onerror = _ => res(false);
        socket.onclose = _ => res(false);
        socket.onopen = _ => {
          clearInterval(timeout_id);
          socket.send(new Uint8Array([radlr.WSRequestCodes.Ping]).buffer);
        }
        socket.onmessage = async msg => {
          socket.onclose = null;
          let data = <Blob>msg.data;
          let bytes = new Uint8Array(await data.arrayBuffer());
          if (bytes.length == 1 && bytes[0] == radlr.WSResponseCodes.Pong) {
            res(true)
          } else {
            res(false)
          }
        }
      }
    )
  }
}


export function createConnection(): LabEngineWebsocketClient {
  return new LabEngineWebsocketClient();
}