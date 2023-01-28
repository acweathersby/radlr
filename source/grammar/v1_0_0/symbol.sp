NAME sherpa_symbol

IGNORE { c:sp c:nl }

<> annotated_symbol > 

        symbol^s [unordered tk:reference?^r "?" ?^o  priority?^p ]

            :ast { t_AnnotatedSymbol, prority:$p, symbol:$s, is_optional:bool($o), reference:str($r), tok  }

        | symbol

<> priority > "{" tk:priority_num '}' :ast { t_Priority, val: u32($2) }
        | "!" :ast { t_Priority, exclusive: true }

<> priority_num > c:num(+)

<> symbol >

        terminal

        | non_terminal

        | list

        | terminal_non_terminal

        | class

<> class >

        "c:" ( 'num' | 'nl' | 'sp' | 'id' | 'sym' | 'any' )

            :ast { t_ClassSymbol, c_Symbol , c_Terminal, val:str($2),  tok }

<> terminal_non_terminal >

        "tk:" non_terminal

             :ast { t_Production_Terminal_Symbol , c_Symbol , c_Terminal, production:$2, tok }

<> non_terminal > 

        production_symbol

        | import_production_symbol

<> production_symbol > 

        tk:identifier_syms

             :ast { t_Production_Symbol , c_Symbol, name:str($1),   tok }

<> import_production_symbol>

        tk:identifier_syms '::' tk:identifier_syms

             :ast { t_Production_Import_Symbol , c_Symbol , module:str($1), name:str($3), tok } 

<> list >

        symbol "(+"  terminal?  ')'

            :ast { t_List_Production, c_Symbol, terminal_symbol:$3, symbols:$1, tok }

        | symbol "(*" terminal?  ')'

            :ast { t_List_Production, c_Symbol, terminal_symbol:$3, symbols:$1, tok, optional:true }

<> terminal > 

        ( c:sym | c:num )(+) c:sp?

        :ast { t_Terminal , c_Symbol , c_Terminal, val:str($1), tok } 

        | "\"" ( c:id | c:sym | c:num | c:sp | escaped )(+) "\""

            :ast { t_Terminal , c_Symbol , c_Terminal, val:str($2), tok, is_exclusive:true } 

        | "'" ( c:id | c:sym | c:num | c:sp | escaped )(+) "'"

            :ast { t_Terminal , c_Symbol , c_Terminal, val:str($2),  tok }   
               

<> escaped > "\\" ( c:id | c:sym | c:num | c:sp ) 

<> reference > 

        "^" tk:identifier_syms

<> identifier > 

        tk:identifier_syms 

<> identifier_syms >  

        identifier_syms c:id

        | identifier_syms '_'

        | identifier_syms '-'

        | identifier_syms c:num

        | '_'

        | '-'

        | c:id
