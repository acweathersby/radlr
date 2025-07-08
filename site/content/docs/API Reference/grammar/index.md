---
title: "Radlr Grammar Reference"
description: "Radlr Documentation"
draft: false
---

## Fundamentals

Basic symbol definition

```
<> non_terminal_A > "terminal" non_terminal_B
<> non_terminal_B > "terminal"
```

Symbols can be grouped together using parenthetical brackets , `(` `)`. Under the hood, this is treated a anonymous non-terminal symbol.


## Terminal Symbols

Terminal symbols are processed by lexer sub-parsers and represent a specific atom of characters matched in an input string. Terminal can either be a specific sequence of characters, or classes.

#### Character Strings

The forward slash (`\`) can be used to escape characters that would otherwise not be treated as part of the character string, include double quotes (`"`) characters.

##### Example:
```
IGNORE { c:sp c:nl }
```


#### Character Classes

|Class Symbol | Matches |
|:-|:-| 
|c:num| Any numeric character |
|c:id| Any identifier START or CONT character |
|c:sym| Any character not matched by other character classes |
|c:sp| Any whitespace character|
|c:nl| Any newline character|
|c:tab| Any horizontal tab character|
|c:vtab| Any vertical tab character|
|c:all| Any single character | 


## Non-Terminal Symbols

##### Example Definition:
```
<> NonTerm > c:id c:num
```

##### Example Usage:
```
<> OtherNonTerm > NonTerm
```

### Imported

### Extended

### Token

A non-terminal reference can be prefixed with `tk:` to treat it as a terminal symbol. Any symantic actions defined within the symbol or it's sub symbols are ignored, and only the characters matched by a token non-term will be tracked. 

### Template

## Preambles

#### `IGNORE`

Ignores a set of terminal symbols. The most common use of this is preamble is to allow whitespace to be ignored.  When an ignored symbol appear in an input the parser will behave as if id did not exist and will continue parsing character following the symbol. Ignored symbols must be terminal. Non-terminals bodies that specifically match on an otherwise ignored symbol will still match that symbol.

##### Definition:
```
"IGNORE" "{" sym::terminal(+) "}" 
```
##### Example:
```
IGNORE { c:sp c:nl }
```

#### `Export`

Exports a symbol as a named parser entry point. This allows for one more non-terminals to be used as the goal for a grammar's parser, in addition to the default primary symbol.

##### Definition:
```
"EXPORT" sym::nonterm (( "AS" | "as" ) prim::id)?
```

##### Example: 
```
EXPORT A as A_entry

<> B > c:id

<> A > c:num
```


#### `IMPORT` 

Imports symbols from another grammar file. Imported symbols are referenced using an import name but otherwise can be used like any other non-terminal symbol
within the host grammar.

##### Definition:
```
"IMPORT" ( c:id | c:sym | c:num  )(+) c:sp ( "AS" | "as" ) prim::id 
```

##### Example: 
```
IMPORT "../symbols.radlr" as sym

<> entry > sym::id
```


## Symbol Modifiers

### Optional
?


### Repeat

#### One Or More
+

#### Zero Or More
*

#### Delimiter
(+, ",")

### Optional



### See Also

[Ascript Reference](../ascript)