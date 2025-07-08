import radlr_init, * as radlr from "js/radlr/radlr_wasm.js";
import { LocalStoreKeys, getLocalValue, setLocalValue, setupOpenCloseTriggers } from "./settings-panel";


/**
 * Updates parser configuration based on UI input
 * @param settings_changed 
 * @returns 
 */
export async function setupConfig(settings_changed: (cfg: radlr.JSParserConfig) => void) {
  await radlr_init();

  let config_panel = document.querySelector("#config-panel");

  if (!config_panel) return;

  let config = null;

  let existing_value = getLocalValue(LocalStoreKeys.ParserConfig);

  if (existing_value) {
    config = radlr.JSParserConfig.import(JSON.parse(existing_value));
  } else {
    config = (new radlr.JSParserConfig()).lab_default();
    setLocalValue(LocalStoreKeys.ParserConfig, JSON.stringify(config.export()));
  }

  for (const label of <HTMLElement[]><any>config_panel.querySelectorAll("label")) {
    let id = label.id;
    let input = <HTMLInputElement>label.querySelector("input");

    switch (input.type) {
      case "checkbox":
        //@ts-ignore
        input.checked = config[id];
        break;
      case "number":
        //@ts-ignore
        input.value = parseInt(config[id]).toString();
        break
    }

    input.addEventListener("change", e => {
      switch (input.type) {
        case "checkbox":
          //@ts-ignore
          config[id] = input.checked;
          break;
        case "number":
          //@ts-ignore
          config[id] = input.value;
          break
      }

      setLocalValue(LocalStoreKeys.ParserConfig, JSON.stringify(config.export()));

      settings_changed(radlr.JSParserConfig.import(config.export()));
    })
  }

  setupOpenCloseTriggers(config_panel, "#open-config-button");

  settings_changed(radlr.JSParserConfig.import(config.export()));
}
