import { DebuggerButton } from "./debugger/debugger_io";
import { Eventable } from "./eventable";

export class Controls extends Eventable<{
  "reset": undefined,
  "play": undefined,
  "step": undefined,
  "stop": undefined,
  "jump": undefined
}> {
  play_button: DebuggerButton
  step_button: DebuggerButton
  jump_button: DebuggerButton
  reset_button: DebuggerButton
  controls: HTMLDivElement
  active: boolean = false

  constructor() {
    super()

    this.controls = <HTMLDivElement>document.getElementById("controls");
    DebuggerButton.gatherButtons(this.controls);
    this.play_button = DebuggerButton.get("play");
    this.step_button = DebuggerButton.get("step");
    this.jump_button = DebuggerButton.get("jump");
    this.reset_button = DebuggerButton.get("reset");


    this.step_button.addEventListener("click", _ => {
      this.emit("step", undefined);
    });

    this.play_button.addEventListener("click", _ => {
      this.emit("play", undefined);
    });

    this.jump_button.addEventListener("click", _ => {
      this.emit("jump", undefined);
    });

    this.reset_button.addEventListener("click", _ => {
      this.emit("reset", undefined);
    });

    window.addEventListener("keydown", e => {
      if (this.active) {
        let key = e.key;
        if (!e.altKey || e.ctrlKey || e.shiftKey) return;
        switch (key) {
          case "n": {
            this.step_button.ele.click();
            e.preventDefault();
            e.stopImmediatePropagation();
            e.stopPropagation()
          } return
          case "r": {
            this.reset_button.ele.click();
            e.preventDefault();
            e.stopImmediatePropagation();
            e.stopPropagation()
          } return
          case "a": {
            this.jump_button.ele.click();
            e.preventDefault();
            e.stopImmediatePropagation();
            e.stopPropagation()
          } return
          case "space": {
            this.play_button.ele.click();
            e.preventDefault();
            e.stopImmediatePropagation();
            e.stopPropagation()
          } return
        }
      }
    }, {
      capture: true
    })

  }

  setActive(active = true) {
    if (active && this.controls.classList.contains("inactive")) {
      this.controls.classList.replace("inactive", "active");
    } if (!active && this.controls.classList.contains("active")) {
      this.controls.classList.replace("active", "inactive");
    }
    this.active = active
  }
}