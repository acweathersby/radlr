import { Eventable } from "./eventable";
import { LabEngineWebsocketClient } from "./lab_client";
import { sleep } from "./pipeline";


export enum LocalStoreKeys {
  ParserInput = "lab-parser-input",
  GrammarInput = "lab-grammar-input",
  StorageEnabled = "lab-local-data-enabled",
  LocalRADLRPort = "lab-radlr-port",
  ParserConfig = "lab-parser-config",
  ActiveTheme = "lab-active-theme",
}

export function dataStorageWorkflowsEnabled() {
  return window.localStorage.getItem(LocalStoreKeys.StorageEnabled) == "true";
}

export function setLocalValue(key: LocalStoreKeys, value: string) {
  if (dataStorageWorkflowsEnabled()) {
    localStorage.setItem(key, value);
  }
}

export function getLocalValue(key: LocalStoreKeys): string {
  if (dataStorageWorkflowsEnabled()) {
    return localStorage.getItem(key) || "";
  } else {
    return ""
  }
}

function setupSetting() {
  let setting_panel = document.querySelector("#settings-panel");

  if (!setting_panel) {
    console.error("Could not locate settings element");
    return;
  }

  const data_enable = new SettingInput(<HTMLLabelElement>setting_panel.querySelector(".data-enable"));
  const data_controls = Array.from(setting_panel.querySelectorAll(".data-control")).map(e => new SettingInput(<HTMLLabelElement>e));
  const data_settings_lu = new Map(data_controls.map(e => [e.setting_id, e]));

  let port_input = data_settings_lu.get("port");
  if (port_input) {
    port_input.on("changed", async value => {
      if (value) {
        setLocalValue(LocalStoreKeys.LocalRADLRPort, value);
        check_lab_host_connection(value, port_input);
      }
    })

    if (getLocalValue(LocalStoreKeys.LocalRADLRPort)) {
      port_input.input.value = getLocalValue(LocalStoreKeys.LocalRADLRPort)
      check_lab_host_connection(port_input.input.value, port_input);
    }
  }

  data_enable.on("checked", _ => {
    localStorage.setItem(LocalStoreKeys.StorageEnabled, "true");
    for (const input of data_settings_lu.values()) {
      input.set_enabled_state(true);
    }
  })

  data_enable.on("unchecked", _ => {
    for (const key in LocalStoreKeys) {
      //@ts-ignore
      let value = <any>LocalStoreKeys[key];
      window.localStorage.removeItem(value);
    }

    for (const input of data_settings_lu.values()) {
      input.set_enabled_state(false);
    }
  })

  if (getLocalValue(LocalStoreKeys.StorageEnabled) == "true") {
    data_enable.input.checked = true;

    for (const input of data_settings_lu.values()) {
      input.set_enabled_state(true);
    }
  } else {
    data_enable.input.checked = false;

    for (const input of data_settings_lu.values()) {
      input.set_enabled_state(false);
    }
  }

  setupOpenCloseTriggers(setting_panel);
  setupThemes(setting_panel);
}


async function check_lab_host_connection(value: any, port_input: SettingInput) {
  if (!await LabEngineWebsocketClient.ping(value, 100)) {
    port_input.set_invalid_state(true, "Could not connect to lab server on port " + value);
  } else {
    port_input.set_invalid_state(false);
  }
}


type SettingInputEvents = {
  "checked": undefined,
  "unchecked": undefined,
  "changed": any
};
/**
 * @description Takes a Label composed as fig.1, and extracts and binds values to 
 * convenient getters and methods.
 * 
 * ```
 * <label>
 *  <div class="title"/>
 *  <div class="note"/>
 *  <input/>
 * </label>
 * ```
 * fig.1
 */
class SettingInput extends Eventable<SettingInputEvents> {
  input: HTMLInputElement
  title: HTMLDivElement
  note: HTMLDivElement
  setting_id: string

  event_handlers: Map<string, (e: any) => any> = new Map;

  constructor(label: HTMLLabelElement) {
    super()
    this.setting_id = <string>label.dataset.setting_id;
    this.title = <HTMLDivElement>label.querySelector(".title")
    this.note = <HTMLDivElement>label.querySelector(".note")
    this.input = <HTMLInputElement>label.querySelector("input")

    this.input.addEventListener("change", e => {
      if (this.input.type == "checkbox") {
        if (this.input.checked) {
          this.emit("checked", undefined);
        } else {
          this.emit("unchecked", undefined);
        }
      } else {
        this.emit("changed", this.input.value)
      }
    });
  }

  is(setting_id: string): boolean {
    return setting_id == this.setting_id;
  }

  on<T extends keyof SettingInputEvents, A = SettingInputEvents[T], D = (arg: A) => void>(event: T, listener: D): void {
    super.addListener(event, listener);
  }

  set_enabled_state(set_enabled: boolean) {
    if (set_enabled) {
      this.input.removeAttribute("disabled")
    } else {
      this.input.setAttribute("disabled", "");
    }
  }

  set_invalid_state(set_invalid: boolean, msg: string = "") {
    if (set_invalid) {
      this.input.setAttribute("invalid", "");
    } else {
      this.input.removeAttribute("invalid")
    }
  }
}


export function setupOpenCloseTriggers(panel: Element, button_selector: string = "#open-settings-button") {

  let close_button = <HTMLDivElement>panel.querySelector(".close-button");

  let open_button = <HTMLDivElement>document.body.querySelector(button_selector);
  open_button.classList.add("inactive");

  console.log(open_button);

  open_button.addEventListener("click", () => {
    if (panel.classList.contains("inactive")) {
      panel.classList.replace("inactive", "active");
      open_button.classList.replace("inactive", "active");
    } else {
      panel.classList.replace("active", "inactive");
      open_button.classList.replace("active", "inactive");
    }
  });

  close_button.addEventListener("click", () => {
    panel.classList.replace("active", "inactive");
    open_button.classList.replace("active", "inactive");
  });
}

function setupThemes(setting_panel: Element) {

  let active_theme = getLocalValue(LocalStoreKeys.ActiveTheme);

  //document.body.classList.add(active_theme);
  let node = <HTMLTemplateElement>document.querySelector("#theme-entry-template");

  if (!node) return;


  Array.from(setting_panel.querySelectorAll(".theme")).map(t => {
    let theme = <HTMLDivElement><any>t;
    let theme_entry = <HTMLElement>(<HTMLElement>node.content.cloneNode(true)).firstElementChild;
    let title = <HTMLElement>theme_entry.querySelector(".theme-entry-title");
    let theme_entry_theme = <HTMLElement>theme_entry.querySelector(".theme");

    let { text, bg, inner_border, inner_bg, border, target: target_class } = theme.dataset;

    theme_entry.style.color = <string>text;
    theme_entry.style.backgroundColor = <string>bg;
    theme_entry.style.borderColor = <string>border;

    theme_entry_theme.style.borderColor = <string>inner_border;
    theme_entry_theme.style.backgroundColor = <string>inner_bg;
    title.style.color = <string>text

    title.innerHTML = (<string>target_class).replace(/^(\w)|-(\w)/g, " <span style='text-transform:uppercase'>$1$2</span>");

    theme_entry.addEventListener("click", async _ => {
      if (active_theme != target_class) {
        let old_theme = active_theme;
        active_theme = <string>target_class;
        setLocalValue(LocalStoreKeys.ActiveTheme, active_theme);
        if (active_theme)
          document.body.classList.add(active_theme);
        if (old_theme)
          document.body.classList.remove(old_theme);
        await sleep(400);
      }
    });

    theme.parentElement?.replaceChild(theme_entry, theme);

    if (active_theme) {
      document.body.classList.add(active_theme);
    }

    let palette = document.querySelector("#palette")!;
    for (const child of Array.from(palette.children)) {
      let style = window.getComputedStyle(child)["backgroundColor"].matchAll(/^rgba?\((?:\s*([\d\.]+)\s*,?)+\)/g);


      //console.log([...style], window.getComputedStyle(child)["backgroundColor"])
    }

    return {
      theme_entry, bg, fg: inner_bg, target_class
    };
  });
}



window.addEventListener("load", setupSetting);