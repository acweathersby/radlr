---
title: "Radlr Grammar"
description: "Radlr Documentation"
draft: false
---

# Grammars
## The heart of Parsers and Lexers

In general terms, a lexer maps a sequence of characters or bytes onto a sequence of tokens. A parser maps the resulting sequence of tokens to a hierarchy of symbols, often referred to a parse tree, a concrete syntax tree (CST), or an abstract syntax tree (AST). Regular expressions are generally suited for producing tokens, and more powerful parsing algorithms such as LL, LR, recursive descent, and Early parsers are utilized for creating the CSTs. RADLR grammars are a combination of lexer and parser rules that describe how characters should be turned into tokens, and how tokens should be grouped into symbols. The syntax of these grammars are inspired by [Extended Backus Naur form](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form) syntax, and adapted to suit the capabilities of RADLR.

## A Brief Introduction To Grammars

> If you are familiar with grammars you may want to skip to [here](#radlr-rules) to get to the meat of things.


When we replace the set of symbols on the left part of the rule with the symbols on the right part, we say we are generating a string in the language the grammar defines. 

> e.g. With the rules `feline -> "tiger"` and `canine -> "wolf"` and `hybrid -> feline canine` we can construct a new string belonging to this language, and in this case the only string that belongs to the langauge, by performing a sequence of transformations like this:
>
> hybrid
>
> feline canine
>
> `"tiger"` canine
>
> [`"tigerwolf"`](https://en.wikipedia.org/wiki/Thylacine)

When the right part of a rule is replaced by its left side, we are discovering whether a particular string belongs to the language, or to put it another way, whether the string can be _recognized_ as a member of the language. If we are able to replace all parts of a string with the left parts of various rules, and are then able to iteratively repeat this process until we are left with the left part of a single rule, designated as the start rule, then we can be certain the original string belonged to our language. This process is the heart of parsing.

> e.g. In this case, givin the rules 
> - `mountain ->  "Everest"`
> - `range    ->  "Himalayas"`
> - `sentence ->  mountain " is in the" range " range` 
>
> when we have the string `"Everest is in the Himalayas"` it can be replaced with the left parts of our rules in a sequence like this: 
> 
> `"Everest is in the Himalayas"` 
> 
> mountain `" is in the Himalayas"` 
>
> mountain `" is in the "` range
>
> sentence

 In general, the number of symbols on either side of the rule is unlimited, but Radlr grammars are based on Context Free Grammars, and as such restrict the number of symbols on the left to just one. The symbol on the left side of a rule is called a _non-terminal_, and the symbols on the right side can be a mixture of other _non-terminals_ ( including the _non-terminal_ on the left part of a rule ) and _terminals_, which are the atomic parts of the grammar and cannot  the left part of a rule and hence cannont be expanded into other symbols. 
 
 So to reiterate, to generate a string in a language defined by a grammar we start with a single _non-terminal_, and replace it through an iterative process of applying grammars rules from left to right until we are left with only _terminal_ symbols. 
 
 Going in the opposite direction, we can take a string and find all matching _terminal_ symbols in  that string. Then we can go through an iterative process of applying our grammar rules from right to left,  replacing both  _terminals_ and _non-terminal_ with other _non-terminals_. If we get to a point where we are left with a single _non-terminal_ that happens to be our goal, then the string has been successfully parsed.


### The Radlr Style

A rule in a Radlr grammar looks appears like this 

```
<> name > sym1 sym2 ... symN
``` 

where `name` on the left of `>` is our _nonterminal_ symbol, and the `sym*`s to the right of the `>` represent other _nonterminals_ and/or _terminals_ symbols. 


## Radlr Rules

### Multiple Rule Bodies

### Grouping

### Lists

## Symbols

### Tokens

### Classes

### Token Nonterminals

### 

## The Preamble

### Exporting Entrypoints

### Importing Grammars

### Ignored Symbols

## Imported Grammars




> The process outlined above is how Concrete Syntax Trees are created, where every part of the input is defined within the context of a _nonterminal_. Other representations of a parsed input usually involve abstracting away details of the input, or performing further transformations beyond the basic reduce operation

of a basic production is `"<>" <production_name> ">" <production_rules>`, where `production_name` is a sequence of alphanumeric and/or `_` underscore characters, and `production_rules` is one or more production rules separated by a `|` character:

```shepa
<> Start_Production > rule1 | rule2 | rule3 | ...
```
Production rules are comprised of symbols that can represent sequences of characters, known as *terminal* symbols, 
and/or identifiers that refer to other productions, called *non-terminal* symbols. 

```Radlr
<> ... > \im_a_terminal_symbol t:im_also_a_terminal_symbol im_a_non_terminal_symbol
```
<dance/>

Terminal symbols can be defined in *escaped literal* form by preceding a sequence of characters with a `\` character and following that character 
sequence with a newline or space. An example of this is the sequence `\hello \world `, which defines the terminal symbols
`hello` and `world`, which is differs from `\hello\world `, which defines the single terminal symbol `hello\world`.

> An alternate terminal symbol syntax, *exclusive literal*, uses the prefix `t:` define a sequence of characters, e.g. 
> `t:hello t:world`. There certain cases where you would want to define an *exclusive literal* over an *escaped literal*. Check
> out [grammar symbols](./api.symbols.index.md#exclusive-terminal-symbol) to find out about the use cases for these symbol types.

#### A Simple Grammar

Let's create a simple grammar to demonstrate the process of writing a Radlr grammar. In order to produce a parser that can 
recognize simple arithmetic expressions, such as `1+2+3`, one could write the following grammar:
```radlr { lab=true }

<> sum > sum "+" num | num 

<> num > "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0"

============
1+2+3+4+5+6+7
```
In this grammar, the production `sum` contains two rules. The first, `sum \+ \num`, has two non-terminals `sum` and `num`, and
on terminal symbol `+`. The second rule of `sum`, following the `|` symbol, contains a single non-terminal `num`. 

The second production `num` contains multiple rules, each with a single terminal symbol representing a character in the range [0..9]. 

> One thing to note about this grammar is the use of left recursion to represent repeated sequences of nonterminals. In this
> case, the non-terminal `<> sum > sum + num` is left recursive, where the first *terminal symbol* `sum` directly references the non-terminal
> in which it is a part of. 
>
> Certain parser compiling algorithms, such as parsers in the LL family and recursive descent parsers, 
> are unable to parse such productions in that form, as this would let to an infinite recursion. There a methods to
> transform such productions, however Radlr is able to directly work with left recursive grammars without transformations. 
> It is actually encouraged to use left recursion, since this leads to the compilation of a more efficient parser.

Radlr provides some modifiers and built in symbols that make it easier to write complex grammars. In our example
grammar, we can use the *generated* symbol `g:num` to represent a character in the range [0..9]. 

```Radlr
<> num > g:num
```

We may also to replace the trivial non-terminal `num` with this generated symbol, which in turn allows us to remove the 
`num` non-terminal entirely.

```Radlr
<> sum > sum \+ g:num | g:num
```

We can represent a sequence that repeats zero or more times with the symbol *modifier* `(*)`. To apply a modifier to a sequence
of symbols, we can wrap them in `(` `)` parenthetic brackets. Applying this syntax to our grammar, we can re-write it as:

```Radlr
<> sum > g:num ( \+ g:num )(*)
```
This single non-terminal represents the same grammar as our first version, but more compactly.

#### Functions

As it is, this grammar will produce a parser that can do nothing more than tell us if a givin input can be produced by the grammar,
in effect the parser can only recognize an input as being part of the grammar. However, if we want are parser to do something more 
interesting, we can provide reduce functions that act on the recognized symbols of the grammar symbols. Radlr calls a reduce
function when the parser recognizes an input as being part of a particular production rule. Going back to an early iteration of our 
grammar, we can add a reduce function to the `num` production's rule to convert the `g:num` symbol into a numeric object:

```Radlr
<> num > g:num :ast { parseInt($1 + "") }
```

> When a *terminal-symbol* is passed to a reduce function, it is represented by a [Token](#todo) object. This object provides several utility
> methods useful when analyzing and transforming an AST into other forms. 

We're using JavaScript expressions to define action used in the reduce function `:ast { ... }`. Radlr supports other syntaxes, but
its native form is JavaScript, so we'll stick with that. The contents of a reduce function is an expression that takes the parsed symbols
of a production rule, modifies them in some way, and returns a new object or primitive. Inside a reduce function symbols are referenced 
by the syntax `\$ g:num`, where `g:num` is the one-based index position of the symbol.

An alternative to using the `\$ g:num` syntax to reference a symbol in a production rule is to use *annotated* symbols and define a
reference name for a symbol: 

```Radlr
<> num > g:num^number :ast { parseInt(number + "") }
```

The `g:num` symbol has been annotated with the reference annotation `^number`, which can now be used within the reduce function to 
refer to that symbol. We can also add a reduce function to the `sum` grammar non-terminal. We'll also use an previous iteration of this
non-terminal to apply the reduce function:

```Radlr
<> sum >    sum^operandA \+ num^operandB   :ast { operandA + operandB }
        
        |   num
```

Note in the second production `|  num` we do not need to apply a parse action, as Radlr will automatically pass the value of 
number primitive when it completed the production `<> num > g:num^number :ast { parseInt(number + "") }`  

> By default, Radlr will return the last symbol within a production rule when it recognizes that rule.

Likewise, the value of num in `... \+ num^operandB ...` is the numeric literal that was created when the non-terminal `num` was completed. 

Now our grammar looks like this:

```radlr { lab=true }
<> sum >   sum^opA "+" num^opB   
                                        :ast{ $opA + $opB }
       |   num

<> num > c:num^number                   :ast { u32($number) }
============

1+2+3

```


```radlr { lab=true }
IGNORE { c:sp c:nl }

<> element_block > '<' component_identifier
    ( element_attribute(+) )?
    ( element_attributes | general_data | element_block | general_binding )(*)
    ">"

<> component_identifier >
    identifier ( ':' identifier )?

<> element_attributes >c:nl element_attribute(+)

<> element_attribute > '-' identifier attribute_chars c:sp


    | '-' identifier ':' identifier

    | '-' "store" '{' local_values? '}'
    | '-' "local" '{' local_values? '}'
    | '-' "param" '{' local_values? '}'
    | '-' "model" '{' local_values? '}'

<> general_binding > ':' identifier

<> local_values > local_value(+)

<> local_value > identifier ( '`' identifier )? ( '='  c:num )? ( ',' )(*)


<> attribute_chars > ( c:id | c:num | c:sym  )(+)
<> general_data > ( c:id | c:num  | c:nl  )(+)

<> identifier > tk:tok_identifier

<> tok_identifier > ( c:id | c:num )(+)
============

"<i -test : soLongMySwanSong - store { test } <i> <i>>

```

Radlr makes it easy to solve the precedence problem for mathematical expressions. 

```radlr { lab=true }
IGNORE { tk:space }

<> expr > expr "+"{1} expr{1}
| expr "^"{4} expr{4}
| expr "*"{3} expr{3}
| expr "/"{2} expr{2}
| expr "-"{1} expr{1}
| c:num

<> space > c:sp(+)
============

1 + 2 * 2 ^ 2 + 2 * 2 + 1 + 1


