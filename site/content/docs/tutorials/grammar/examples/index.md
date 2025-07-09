
# Examples of various languages as a RADLR grammar

```radlr { lab=true }
IGNORE { c:sp  } 

A => match : BYTE (65 /* A */ | 66 /* B */) { goto B }

B => shift then pass

============

A

```


```radlr { lab=true }
IGNORE { c:sp  } 

<> A > ( B | ":" C )(+)

<> B > id "=>" c:id

<> C > a_id(+)

<> a_id > id "!"? 

<> id > tk:id_tok

<> id_tok > c:id
============

:t t => g :t! t!

```


```radlr { lab=true }
IGNORE { tk:space } 

<> compound > "(" ( compound | atom )(*)  ")"

<> sexpr > compound | tk:atom

<> atom = c:id ( c:id | c:num )(+)

<> space >  (c:sp | c:nl)(+)

============

:t t => g :t! t!

```
### The YACC syntax

> Yacc uses some preprocessing during the parsing of syntactic input. This has been emulated with ir code.

```radlr { lab=true }

IGNORE { c:nl c:sp c:tab tk:comment }

EXPORT spec as yacc

<> spec   > defs? "%%" rules "%%"?

// --------------- Header Definition --------------------------------

<> defs   > def(+)

<> def    > "$start" id 
          |  "%union" "{" "}"
          |  "{" "}"
          |  rword tag? nlist

<> rword  > "%token" | "%left" | "%nonassoc" | "%type"

<> tag    > "<" id ">"

<> nlist  > nmno | nlist nmno | nlist "," nmno

<> nmno   > id num?

//---------------- Rule Definitions --------------------------------

<> rules   > rule(+)

<> rule    > c_ident (rbody)(+"|") prec? ";"

<> c_ident > id ":"

<> rbody   > ( id act? | tok act? )(+) | "*"

<> prec    > "%prec" id act?

<> act     > "{"  "}"

//---------------- Nontrivial Token Definitions --------------------

<> num     > tk:number

<> number  > c:num(+)

<> id      > tk:ident

<> ident   > c:id ( c:id | c:num | "_" | "-" )(*) 

<> tok     > tk:token

<> token   > "\"" ( c:sym | c:id | c:num | c:sp )(*) "\""
           | "'"  ( c:sym | c:id | c:num | c:sp )(*) "'"

<> comment > line | block

<> line > '/' "/" ( c:id | c:sp | c:sym | c:num )(*) c:nl

<> block > '/' "*"  ( c:id | c:sp | c:sym | c:num | c:nl )(*) '*' "/"

============

						/* grammar for the input to yacc */ 

  						/* basic entries */ 
%token IDENTIFIER				/* includes identifiers and literals */ 

%token C_IDENTIFIER			/* identifier (but not literal) */
                     		/* followed by a : */ 

%token NUMBER				  	/* [0-9]+ */

    			/* reserved words: %type=>TYPE %left=>LEFT,etc.  */ 

%token LEFT RIGHT NONASSOC TOKEN PREC TYPE START UNION 

%token MARK			     		/* the %% mark */ 

%token LCURL			 		/* the %{ mark */ 

%token RCURL					/* the %) mark */ 

    				            /* ASCII character literals stand for themselves */ 

%% 

spec   		: defs MARK rules tail 
          	; 
tail	   	: MARK 
         	{ 
          			
         	} 
       		| *	     	    /* empty: the second MARK is optional */ 
       		; 
defs	   	: *	      		/* empty */ 
       		| defs def 
       		; 
def	    	: START IDENTIFIER 
       		| UNION 
       		  { 
            	/* Copy union definition to output */
         	  } 
         	| LCURL 
       		  { 
          		/* Copy C code to output file */
       		  } 
       		  RCURL 
            | rword tag nlist
         		;
rword       : TOKEN 
       		| LEFT 
       		| RIGHT 
       		| NONASSOC 
         	| TYPE 
       		; 
tag	  	    : *
       		| '<' IDENTIFIER '>' 
       		; 
nlist	  	: nmno 
       		| nlist nmno 
       		| nlist ',' nmno 
            ;
%%

```



A parse produced by this grammar will now be able to take an input such as `"1+2+3"` and return a numeric literal `6`. 

## What Next

Checkout [creating the bash parser](./tutorial.bash_parser.index.md) if you desire a thorough guide on generating a parser with Radlr.

You can also checkout the Grammar API to gain an in-depth understanding of grammar constructs. 


