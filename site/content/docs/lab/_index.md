---
title: "Lab"
description: "Learn about the RADLR lab and how it can be used to create language tools"
layout: "docs"
draft: false
---

# The Lab

This lab provides a suite of tools to interact with and develop RADLR grammars. Using the [Grammar Editor](./grammar_panel/) panel you can edit and compile grammars with near feature parity to the CLI version of RADLR. Error messages and symbol information provide guide paths to creating a suitable grammar. The type of parser that is produced from the grammar can be adjusted through the  [Parser Configuration](./parser_config.md) modal ![Parser Config Modal](./parser_config_modal.png), where you can adjust parameters such the amount of symbol lookahead the parser is allowed to use.

Following a successful compilation of your grammar, you can evaluate the resulting parser's behavior on some input through the [Parse Input](./parser_input_panel/) panel. Step-by-step parsing can be controlled by the [Transport Control](./transport_control/), where you can also find information about the parser. The [Parser Info](./parser_info_panel/) panel provides detailed information about the parser, and lists out the full byte code "disassembly".

The [CST](./ast_panel/) and [AST](./ast_panel/) panels provide interactive views of the tree structures that can be produced from the input. Finally, the [Formatting](./formatting_rules_panel.md) and [Syntax Highlighting](./syntax_highlighting_panel.md) panels allow for programmatic editing of the input.

---

Click a panel's help button ![Help Button](./help_button.png) to learn more about it.

## Learn More

- [Using the lab]()
- [Arranging Panels]()
- [Using Local Compilation]()