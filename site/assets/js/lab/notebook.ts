

import { MoveFieldDragOperation, ResizeFieldOperation } from "./dragndrop_operations";


const TRANSITION_DURATION_MS = 200;
export const MIN_EXPANDED_FIELD_HEIGHT = 160;
const COLLAPSED_FIELD_HEIGHT = 40;
const UNASSIGNED_COLUMN = -100;
const MINI_COLUMN = -1;

export class NB {
  ele: HTMLElement
  columns: NBColumn[];
  max_columns: number = 3;
  mini_col: NBColumn

  constructor(num_of_columns: number) {

    let ele = <HTMLElement | null>document.querySelector("#notebook")

    if (ele) {
      this.ele = ele;

      num_of_columns = Math.min(Math.max(1, num_of_columns), 3);

      this.columns = new Array(num_of_columns).fill(0).map((_, i) => new NBColumn(this, i, false))

      this.mini_col = new MININBColumn(this);

      this.ele.append(this.mini_col.ele);

      for (const row of this.columns) {
        this.ele.append(row.ele);
      }
    } else {
      throw "Could not locate #notebook element"
    }
  }

  add_field<T extends NBField>(field: T, col: number = 0, row: number = Infinity): T {
    let column = this.mini_col;

    if (col >= 0) {
      col = Math.min(Math.max(0, col), this.columns.length - 1);
      column = this.columns[col];
    }


    field.nb_host = this;

    // Remove from any existing rows

    for (const row of this.columns) {
      row.remove(field);
    }

    column.add(field, row);

    return field
  }

  remove_field<T extends NBField>(field: T): boolean {
    let { col, v_row: row, nb_host } = field;

    if (nb_host != this || col <= UNASSIGNED_COLUMN) return false;

    if (col == MINI_COLUMN) {
      this.mini_col.remove(field)
    } else {
      this.columns[col].remove(field);
    }

    field.col = UNASSIGNED_COLUMN;
    field.v_row = -1;
    field.r_row = -1;
    field.nb_host = null;

    return true
  }

  remove_column(col_index: number) {
    if (col_index >= 0 && col_index < this.columns.length && this.columns.length > 1) {
      let col = this.columns[col_index];

      if (col.cell_count > 0) {
        throw "Attempted to delete a non-empty column"
      }

      this.columns.splice(col_index, 1);
      this.columns.forEach((col, i) => col.set_index(i));
      col.delete();
    }
  }


  insert_column(col_index: number) {
    let new_col = new NBColumn(this, col_index, false);
    this.columns.splice(col_index, 0, new_col);

    if (new_col != this.columns[col_index]) {
      throw "wt"
    }

    this.columns.forEach((col, i) => col.set_index(i));

    if (col_index == 0) {
      this.ele.insertBefore(new_col.ele, this.mini_col.ele.nextElementSibling);
    } else {
      this.ele.insertBefore(new_col.ele, <any>this.columns[col_index - 1].ele.nextElementSibling);
    }

  }


  calculate_heights() {
    for (const col of this.columns) {
      col.latchHeights();
      col.distribute_height();
    }

    this.mini_col.latchHeights();
  }
  column_area_width(): number {
    return this.ele.getBoundingClientRect().width - this.mini_col.ele.getBoundingClientRect().width
  }
}



export class NBColumn {
  ele: HTMLElement;
  cells: NBField[]
  nb_host: NB
  index: number
  is_mini: boolean = false
  IS_SIDEBAR: boolean = false

  constructor(nb_host: NB, index: number, animate_in: boolean, is_mini: boolean = false) {
    this.is_mini = is_mini;


    this.ele = document.createElement("div");
    this.ele.classList.add("nb-column");
    this.cells = [];
    this.nb_host = nb_host;
    this.index = index;

    if (animate_in) {
      this.ele.style.width = "0"
      setTimeout(() => {
        this.ele.style.width = ""
      }, 10)
    }
  }


  swap(other_field: NBField, own_index: number) {
    let own_field = this.cells[own_index];

    if (own_field && own_field != other_field && other_field.nb_host == this.nb_host) {


      let other_col = this.nb_host.columns[other_field.col]

      if (!other_col) return;

      let own_height = own_field.latched_height;
      let other_height = other_field.latched_height;

      own_field.latched_height = other_height;
      other_field.latched_height = own_height;

      let other_index = other_field.r_row;

      let temp_ele = document.createElement("span");
      let own_ele = own_field.ele;
      let other_ele = other_field.ele;

      this.cells[own_index] = other_field;
      other_col.cells[other_index] = own_field;

      this.set_index();
      other_col.set_index();

      this.distribute_height();
      other_col.distribute_height();

      this.ele.replaceChild(temp_ele, own_ele);
      other_col.ele.replaceChild(own_ele, other_ele);
      this.ele.replaceChild(other_ele, temp_ele);
    }
  }


  max_free(): number {
    return this.ele.getBoundingClientRect().height - this.cell_count * MIN_EXPANDED_FIELD_HEIGHT;
  }

  delete() {
    this.ele.parentElement?.removeChild(this.ele);
    for (const cell of this.cells) {
      cell.col = UNASSIGNED_COLUMN;
    }
  }

  remove(field: NBField) {
    let index = this.find(field);

    if (index >= 0) {
      this.cells.splice(index, 1);
      this.ele.removeChild(field.ele);
      field.nb_host = null;
      field.col = UNASSIGNED_COLUMN;
      field.v_row = -1;

      this.set_index()
    }
  }

  add(field: NBField, row: number = Infinity, using_real_index: boolean = false) {
    row = using_real_index ? row : this.find_real_index(row);

    if (row < this.cells.length) {
      this.cells.splice(row, 0, field);
      this.ele.insertBefore(field.ele, this.cells[row + 1].ele);
      field.col = this.index;
    } else {
      this.cells.push(field);
      this.ele.appendChild(field.ele);
    }

    field.nb_host = this.nb_host;
    field.v_row = -1;
    field.is_mini = false;

    this.set_index()
  }

  set_index(col_index: number = this.index) {
    this.index = col_index;
    this.cells.forEach((i, index) => { i.r_row = index; i.col = col_index });
    this.cells.filter(n => !(n instanceof NBBlankField)).forEach((i, index) => i.v_row = index);
  }

  find(field: NBField): number {
    return this.cells.findIndex(f => f === field);
  }

  find_real_index(virtual_index: number): number {
    let index = -1;
    for (const cell of this.cells) {
      index++;
      if (cell.v_row >= 0 && cell.v_row >= virtual_index) {
        return index;
      }
    }

    return this.cells.length
  }

  is_point_inside(x: number, y: number, edge_size: number = 0): { insert_row: number, alignment: number } | null {

    const { x: col_x, y: col_y, width, height } = this.ele.getBoundingClientRect();
    const col_max_x = col_x + width;
    const col_max_y = col_y + height;
    const inside_x = x > col_x && x < col_max_x;
    const inside_y = y > col_y && y < col_max_y;

    if (inside_x && inside_y) {
      let alignment = 0;

      if (x - col_x <= edge_size) {
        alignment = 1
      } else if (col_max_x - x <= edge_size) {
        alignment = 2
      }

      let target_offset = y - col_y;
      const real_cells = this.cells.filter(f => !(f instanceof NBBlankField));
      for (const cell of real_cells) {
        const height = cell.latched_height;
        if (target_offset <= height) {
          if (target_offset < (height / 2)) {
            // Insert before field
            return { insert_row: cell.v_row, alignment }
          } else {
            // Insert after field
            return { insert_row: cell.v_row + 1, alignment }
          }

        } else {
          target_offset -= height;
        }
      }

      return { insert_row: real_cells.length, alignment }
    }

    return null;
  }

  latchHeights() {
    this.cells.forEach(c => c.latch_height())
  }

  get cell_count(): number {
    return this.cells.filter(f => !(f instanceof NBBlankField)).length
  }


  distribute_height(fixed_heights_settings: { index: number, height: number }[] = []) {

    let real_height = this.ele.getBoundingClientRect().height;

    let cell_heights = this.cells.map((c, i) => {

      if (c instanceof NBBlankField && c.deleting) {
        return { f: false, h: 0 }
      }

      return this.get_field_height(fixed_heights_settings.find(f => f.index == i), c);
    });

    let cell_height_sum = cell_heights.reduce((r, l) => {
      if (l.f) { r.f += l.h } else { r.l += l.h }
      return r
    }, { f: 0, l: 0 });

    let remainder_percentage = 1 / (cell_height_sum.l / (real_height - cell_height_sum.f));
    let inv_real_height = 1 / real_height;
    let normalized_heights = [];
    let cells_length = this.cells.length;

    for (let i = 0; i < cells_length; i++) {
      let height = cell_heights[i];
      if (height.f) {
        normalized_heights.push(height.h * inv_real_height);
      } else {
        normalized_heights.push(height.h * remainder_percentage * inv_real_height);
      }
    }

    let normalized_ratio = 1 / normalized_heights.reduce((v, a) => v + a, 0);

    for (let i = 0; i < cells_length; i++) {
      this.cells[i].set_relative_height(normalized_heights[i] * normalized_ratio);
    }
  }

  protected get_field_height(fixed_height_data: { index: number; height: number; } | undefined, c: NBField): { f: boolean, h: number } {
    if (fixed_height_data) {
      return { f: true, h: fixed_height_data.height };
    } else if (c.collapsed) {
      return { f: true, h: COLLAPSED_FIELD_HEIGHT };
    } else if (c.latched_height <= MIN_EXPANDED_FIELD_HEIGHT) {
      return { f: true, h: MIN_EXPANDED_FIELD_HEIGHT };
    } else {
      return { f: false, h: c.latched_height };
    }
  }
}


export class MININBColumn extends NBColumn {
  constructor(nb_host: NB) {
    super(nb_host, -1, false, true);
    this.ele.classList.add("mini");
  }

  add(field: NBField, row?: number, using_real_index?: boolean): void {
    super.add(field, row, using_real_index);
    field.is_mini = true;
  }

  distribute_height() {
    // Every field is set to be 40px high
    let ratio = 50 / this.ele.getBoundingClientRect().height;
    for (const field of this.cells) {

      if (field instanceof NBBlankField && field.deleting)
        continue;

      field.set_relative_height(ratio);
    }

  }
}

export class NBField {
  ele: HTMLElement;
  nb_host: null | NB = null
  // The index of the row excluding any blank cells
  v_row: number = 0;
  // The index of the row including blank cells
  r_row: number = 0;
  col: number = 0;
  latched_height: number = 0
  collapsed: boolean = false
  is_mini = false;

  constructor(ele = document.createElement("div")) {
    this.ele = ele;
    this.ele.classList.add("nb-field")
  }

  latch_height() {
    if (this.collapsed) return;

    const { height } = this.ele.getBoundingClientRect();
    this.latched_height = height;
  }

  /**
   * @param height - a ratio of the parent containers height
   */
  set_relative_height(height: number) {
    this.ele.style.height = `${height * 100}%`;
  }

  unset_relative_height() {
    this.ele.style.height = ""
  }

  add_class(class_: string) {
    this.ele.classList.add(class_);
  }

  remove_class(class_: string) {
    this.ele.classList.remove(class_);
  }
}

export class NBBlankField extends NBField {
  deleting: boolean = false

  constructor(width: number, height: number, force_height: boolean = false) {
    super()
    this.ele.classList.add("nb-blank-field");
    this.latched_height = height;
    this.ele.appendChild(document.createElement("div"))
    setTimeout(() => {
      this.ele.style.opacity = "1"
    }, 10)
  }

  set_relative_height(height: number): void {
    setTimeout(() => {
      this.ele.style.height = `${height * 100}%`
    }, 10)
  }

  delete() {

    this.deleting = true;
    this.ele.style.opacity = `${0}`
    this.ele.style.padding = `${0}`
    this.ele.style.margin = `${0}`
    this.set_relative_height(0);

    if (this.nb_host) {
      let nb_host = this.nb_host;
      setTimeout(() => { nb_host.remove_field(this); }, TRANSITION_DURATION_MS)
    } else {
      setTimeout(() => {
        if (this.ele.parentElement)
          this.ele.parentElement.removeChild(this.ele)
      }, TRANSITION_DURATION_MS)
    }
  }
}

export class NBContentField<EventObj = null, event_names = ""> extends NBField {
  public help_button: HTMLElement;
  header: HTMLElement;
  body: HTMLElement;
  label: HTMLElement;
  resize_handle: HTMLElement;
  expand_button: HTMLElement;
  collapsed: boolean = false;
  help_doc_path: string = ""
  pre_collapse_size: number = 0
  listeners: Map<event_names, ((arg: EventObj) => void)[]> = new Map;
  relative_height: string = ""

  constructor(name: string = "") {
    let ele = document.createElement("div");
    ele.append((<HTMLTemplateElement>document.querySelector("#nb-field-template")).content.cloneNode(true));

    super(<any>ele.firstElementChild)

    this.header = <any>this.ele.querySelector(".nb-header");
    this.label = <any>this.ele.querySelector(".nb-label");
    this.body = <any>this.ele.querySelector(".nb-body");
    this.expand_button = <any>this.ele.querySelector(".nb-expand-button");
    this.resize_handle = <any>this.ele.querySelector(".nb-resize-handle");
    this.help_button = <any>this.ele.querySelector(".nb-help-button");
    this.ele.querySelector(".nb-icon-container")!.setAttribute("title", name);

    this.label.innerHTML = name;

    this.expand_button.addEventListener("pointerdown", e => {
      e.preventDefault();
      e.stopImmediatePropagation();
      e.stopPropagation();
      return false
    }, {
      capture: true,
      passive: true
    });

    this.expand_button.addEventListener("click", e => {
      this.set_fullscreen(!this.is_fullscreen);
      e.preventDefault();
      e.stopImmediatePropagation();
      e.stopPropagation();
      return false;
    }, {
      capture: true,
      passive: true
    })

    this.resize_handle.addEventListener("pointerdown", e => {
      new ResizeFieldOperation(e, this);
    }, { capture: true, passive: false })

    this.header.addEventListener("pointerdown", async e => {
      if (this.nb_host && !this.is_fullscreen) {
        let drag_op = new MoveFieldDragOperation(e, this);
        await drag_op.willDrag()
      }
    }, { capture: false, passive: false });
  }

  private get is_expanded(): boolean {
    return !this.ele.classList.contains("collapsed");
  }

  set_help_doc_path(path: string) {
    this.help_doc_path = path
  }
  
  /** Updates the color of the body border, or unsets it if the color arg is the empty string  */
  set_info_border(color: string =  "")  {
    this.body.style.borderColor = color;
  }

  set_icon(ele: HTMLElement | string) {
    if (typeof ele == "string") {
      this.ele.querySelector(".nb-icon-container")!.innerHTML = ele;
    } else {
      this.ele.querySelector(".nb-icon-container")!.innerHTML = "";
      this.ele.querySelector(".nb-icon-container")!.appendChild(ele);
    }
  }

  set_content_hidden(is_content_hidden: boolean) {
    if (is_content_hidden) {
      this.ele.classList.add("content-hidden");
    } else {
      this.ele.classList.remove("content-hidden");
    }
  }

  set_loading(is_loading: boolean) {
    if (is_loading) {
      this.ele.classList.add("loading");
    } else {
      this.ele.classList.remove("loading");
    }
  }

  set_expanded(is_collapsed: boolean) {
    if (!this.nb_host) return;

    if (!is_collapsed) {
      this.nb_host.calculate_heights();
      this.pre_collapse_size = this.latched_height
      this.ele.classList.add("collapsed");
      this.collapsed = true
      this.nb_host.calculate_heights();
    } else {
      this.ele.classList.remove("collapsed");
      this.collapsed = false
      this.latched_height = this.pre_collapse_size
      this.nb_host.columns[this.col].distribute_height()
    }
  }

  set_fullscreen(is_fullscreen: boolean) {
    if (is_fullscreen) {
      this.add_class("fullscreen");
      this.ele.style.height = ""
    } else {
      this.remove_class("fullscreen");
      this.ele.style.height = this.relative_height;
    }
  }

  get is_fullscreen(): boolean {
    return this.ele.classList.contains("fullscreen")
  }

  protected emit(event: event_names) {
    for (const listener of this.listeners.get(event) ?? []) {
      listener(<EventObj><any>this);
    }
  }

  addListener(event: event_names, listener: (arg: EventObj) => void) {

    if (!this.listeners.get(event)) {
      this.listeners.set(event, [listener]);
    } else {
      this.listeners.get(event)?.push(listener);
    }
  }

  set_relative_height(height: number) {
    this.relative_height = `${height * 100}%`;
    if (!this.is_fullscreen) {
      this.ele.style.height = this.relative_height
    }
  }
}

import * as language from "@codemirror/language";
import * as state from "@codemirror/state";
import * as view from "@codemirror/view";
import * as commands from "@codemirror/commands";
import * as autocomplete from "@codemirror/autocomplete";
import * as search from "@codemirror/search";
import * as lint from "@codemirror/lint";

const basicSetup = (() => [
  view.EditorView.theme({ "*": { "color": "unset !important", "background-color": "unset !important" } }),
  autocomplete.autocompletion(),
  autocomplete.closeBrackets(),
  commands.history(),
  language.bracketMatching(),
  language.foldGutter(),
  language.indentOnInput(),
  language.syntaxHighlighting(language.defaultHighlightStyle, { fallback: true }),
  search.highlightSelectionMatches(),
  state.EditorState.allowMultipleSelections.of(true),
  view.crosshairCursor(),
  view.drawSelection(),
  view.dropCursor(),
  view.highlightActiveLine(),
  view.highlightActiveLineGutter(),
  view.highlightSpecialChars(),
  view.lineNumbers(),
  view.rectangularSelection(),
  view.keymap.of([
    ...autocomplete.closeBracketsKeymap,
    ...commands.defaultKeymap,
    ...search.searchKeymap,
    ...commands.historyKeymap,
    ...language.foldKeymap,
    ...autocomplete.completionKeymap,
    ...lint.lintKeymap
  ])
])();

export const highlight_effect = state.StateEffect.define<state.Range<view.Decoration>[]>();
export const filter_effects = state.StateEffect.define<((from: number, to: number, decoration: view.Decoration & { attrs: any }) => boolean)>();



export class NBEditorField extends NBContentField<NBEditorField, "text_changed"> {

  cm: view.EditorView;
  diagnostics: lint.Diagnostic[] = [];

  constructor(name: string) {
    super(name);

    this.cm = new view.EditorView({
      doc: "",
      extensions: [basicSetup, state.StateField.define({
        create() {
          return state.RangeSet.of([]);

        },
        update(value, tr) {

          //value = value.map(tr.changes,)

          for (let effect of tr.effects) {
            if (effect.is(highlight_effect)) {
              value = value.update({ add: effect.value, filterTo: 20, sort: true })
            }
            else if (effect.is(filter_effects)) value = value.update({ filter: effect.value });
          }

          return value.map(tr.changes)
        },
        provide: f => view.EditorView.decorations.from(<any>f)
      }), view.ViewPlugin.define(
        (_) => ({
          update: (update) => {
            if (update.docChanged) {
              this.emit("text_changed");
            }
          }
        })
      )],
      parent: this.body
    });
  }

  set_text(text: string) {
    this.cm.dispatch(
      {
        changes: { from: 0, to: this.cm.state.doc.length, insert: text }
      }
    );
  }

  get_text(): string {
    return this.cm.state.doc.toString();
  }

  add_highlight(start_char: number, end_char: number, color: string) {
    end_char = Math.min(this.cm.state.doc.length, end_char);
    if (start_char == end_char) return;
    this.cm.dispatch({
      effects: [highlight_effect.of([
        view.Decoration.mark({ attributes: { style: `color: ${color}` }, id: "highlight" }).range(start_char, end_char)
      ])]
    });
  }

  add_highlights(...highlights: [number, number, string][]) {
    let len = this.cm.state.doc.length;
    this.cm.dispatch({
      effects: highlight_effect.of(highlights.filter(([from, to, color]) => {
        to = Math.min(len, to);
        return (to - from) > 0
      }).map(([from, to, color]) => {
        to = Math.min(len, to);
        return view.Decoration.mark({ attributes: { style: `color: ${color}` }, id: "highlight" }).range(from, to)
      }))
    });
  }

  remove_highlights() {
    this.cm.dispatch({
      effects: [filter_effects.of((from, to, decoration) => {
        return !(decoration.spec.id == "highlight")
      })]
    });
  }

  add_character_class(start_char: number, end_char: number, _class: string) {
    end_char = Math.min(this.cm.state.doc.length, end_char);
    if (start_char == end_char) return;
    this.cm.dispatch({
      effects: [highlight_effect.of([
        view.Decoration.mark({ class: _class, id: "class" }).range(start_char, end_char)
      ])]
    });
  }

  add_character_classes(...highlights: [number, number, string][]) {
    let len = this.cm.state.doc.length;
    this.cm.dispatch({
      effects: highlight_effect.of(highlights.filter(([from, to,]) => {
        to = Math.min(len, to);
        return (to - from) > 0
      }).map(([from, to, _class]) => {
        to = Math.min(len, to);
        return view.Decoration.mark({ class: _class, id: "class" }).range(from, to)
      }))
    });
  }

  remove_specific_character_classes(class_: string) {
    this.cm.dispatch({
      effects: [filter_effects.of((from, to, decoration) => {
        return !(decoration.spec.id == "class" && decoration.spec.class == class_)
      })]
    });
  }

  remove_character_classes() {
    this.cm.dispatch({
      effects: [filter_effects.of((from, to, decoration) => {
        return !(decoration.spec.id == "class")
      })]
    });
  }

  add_message(start_char: number, end_char: number, msg: string) {

    end_char = Math.min(this.cm.state.doc.length, end_char);
    if (start_char == end_char || start_char > end_char) return;

    this.diagnostics.push({
      from: start_char,
      to: end_char,
      message: msg,
      severity: "warning",
    });

    this.cm.dispatch(lint.setDiagnostics(this.cm.state, this.diagnostics));
  }

  remove_messages() {
    this.diagnostics.length = 0;
    this.cm.dispatch(lint.setDiagnostics(this.cm.state, this.diagnostics));
  };
}

;