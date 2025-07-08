import { NB, NBContentField, NBEditorField } from "./notebook";

export default function init_help(nb: NB, help_field: NBContentField, grammar_input_field: NBEditorField, parse_input_field: NBEditorField, fields: NBContentField[]) {

  let iframe = document.createElement("iframe");
  iframe.classList.add("nb-help-iframe")


  iframe.addEventListener("load", () => {
    if (iframe.contentDocument) {

      let header = iframe.contentDocument.querySelector("header"); 
      if (header)
        header.style.display = "none";

      let anchors = Array.from(iframe.contentDocument.querySelectorAll("a"));

      for (const a of anchors) {
        if (a.classList.contains("lab-candidate")) {
          if (a.parentElement) {

            let grammar_input = a.parentElement.querySelector(".lab-grammar");
            let parser_input = a.parentElement.querySelector(".lab-parser");

            if (grammar_input && parser_input) {

              a.addEventListener("click", e => {
                e.preventDefault();
                e.stopImmediatePropagation();
                e.stopPropagation();

                //@ts-ignore
                parse_input_field.set_text(parser_input.innerText);

                //@ts-ignore
                grammar_input_field.set_text(grammar_input.innerText);

                console.log({ grammar_input, parser_input })

                return false;

              })
            }
          }
        }
      }
    }

    help_field.set_loading(false);

  })

  for (const field of fields) {
    field.help_button.addEventListener("click", _ => {
      if (field.help_doc_path) {
        help_field.set_loading(true);
        iframe.src = field.help_doc_path;
      }
    })
  }

  help_field.body.appendChild(iframe);

  help_field.set_loading(true);
  iframe.src = help_field.help_doc_path;
}