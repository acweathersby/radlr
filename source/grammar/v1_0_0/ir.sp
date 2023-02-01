IGNORE { c:sp c:nl }

EXPORT state as state
EXPORT ir as ir

<> ir > state(+)

<> state > 

        state_declaration scanner_declaration? top_level_instructions on_fail? expected_symbols?

            :ast { t_IR_STATE, c_IR, c_IrState, id:$1, scanner: $2, instructions: $3, fail: $4, symbol_meta:$5 }

<> state_declaration > 

        "state" '['  tk:state_hash_token ']'

            :ast str($3)

<> scanner_declaration > 

        "scanner" '['  tk:state_hash_token ']'

            :ast str($3)

<> state_reference > 

        "state" '[' tk:state_hash_token  ']'

            :ast { t_HASH_NAME, val:str($3) } 

<> top_level_instructions > 

        assertion_instruction(+) 

    |   instruction_sequence

<> instruction_sequence > 

        sequence_instruction(+ "then" ) 
        ( "then" goto_instruction(+ "then" ) )? 
        ( "then" "repeat" "state" :ast { t_Repeat, c_IR, c_IR_Instruction } )? 

        :ast [ $1, $2, $3]

    |   goto_instruction(+ "then" ) ( "then" "repeat" "state" :ast { t_Repeat, c_IR, c_IR_Instruction } )? 

        :ast [$1, $2 ]

    |   ( 
        
        "lazy" '(' c:num(+) \: c:num(+) ')' state_reference 

            :ast { t_Lazy, c_IR, c_IR_Instruction, cp_start:$3, cp_end:$5, state: $7 } 
        )

        :ast [$1]

<> assertion_instruction >

        "assert" "peek"? assert_class production_id_list '(' instruction_sequence ')'

            :ast { t_ASSERT, c_IR, c_IR_Instruction, is_peek: bool($2), c_IR_Branch, mode:str($3), ids: $4, instructions: $6}

        | "skip" production_id_list

            :ast { t_ASSERT, c_IR, c_IR_Instruction, c_IR_Branch, ids: $2, is_skip: true }

        | "default"  '(' instruction_sequence ')'                     

            :ast { t_DEFAULT, c_IR, c_IR_Instruction, c_IR_Branch, instructions: $3}

<> assert_class > 'PRODUCTION' | 'TOKEN'  | 'BYTE' | 'CODEPOINT' | 'CLASS' 

<> goto_instruction > 

    "goto" state_reference

        :ast { t_Goto, c_IR, c_IR_Instruction, state: $2 }

<> sequence_instruction >

        reduce_instruction

    |   breadcrumb

    |   "assign" "token" token_id_list

        :ast { t_TokenAssign, c_IR, c_IR_Instruction, ids: $3 }

    | "set" "prod" "to" token_num

        :ast { t_SetProd, c_IR, c_IR_Instruction, id: $4  }

    | "fork" "to" '(' state_reference(+) ')' "to" "complete" "prod" token_num

        :ast { t_ForkTo, c_IR, c_IR_Instruction, states: $4, production_id: $9   }

    | "scan" "back"? "until" token_id_list

        :ast { t_ScanUntil, c_IR, c_IR_Instruction, ids: $4, SCAN_BACKWARDS:bool($2) }

    | "set" "token" "id" token_num token_num

        :ast { t_TokenId, c_IR, c_IR_Instruction, id: $5  }

    | "skip"

        :ast { t_Skip, c_IR, c_IR_Instruction  }

    | "pass"

        :ast { t_Pass, c_IR, c_IR_Instruction  }

    | "fail"

        :ast { t_Fail, c_IR, c_IR_Instruction }

    | "not" "within" "scopes" '[' ( token_num )(+) ']'

        :ast { t_NotInScope, c_IR, c_IR_Instruction, ids:$5 }

    | "set" "scope" "to" tk:integer

        :ast { t_SetScope, c_IR, c_IR_Instruction, scope:i64($1) }

    | "shift" "nothing"?

        :ast { t_Shift, c_IR, c_IR_Instruction, EMPTY:bool($2) }

<> reduce_instruction > 

    "reduce" tk:integer ( "symbols" "with" "rule" )? tk:integer

        :ast { t_Reduce, c_IR, c_IR_Instruction, len: i32($2), rule_id: i32($4) }

<> breadcrumb > 

    "crumb" '[' tk:integer '|' breadcrumb_action ']'

        :ast { t_Crumb, lane:i32($3), action: $5 }

    | "crumb" "complete" '[' tk:integer ']' 

        :ast { t_CrumbComplete, lane: i32($4)  }

<> breadcrumb_action >

    "map" tk:integer

        :ast { t_Map, lane: i32($2) }

    | reduce_instruction

    | "shift"

        :ast { t_Shift }

<> on_fail > 

    "on" "fail" state_declaration top_level_instructions on_fail? expected_symbols?

        :ast { t_FailState, c_IR, c_IR_State, id:$3, instructions: $4, symbol_meta: $6, fail: $5 }

<> expected_symbols > 

        'symbols:' 'expected' token_id_list ( 'skipped' token_id_list )?

            :ast { t_Symbols, c_IR, expected:$3, skipped:$4 }

<> token_id_list > 

        '[' ( token_num  )(+)  ']' 

            :ast [ $2 ]

<> production_id_list > 

        '[' tk:integer ']' 

            :ast { t_Num, val: i64($2) }

<> state_hash_token > 

        state_hash_token ( '_' | '-' | c:id | c:num )
    |   c:id
    |   c:num
    |   '_'
    |   '-'

<> token_num > 
        tk:integer :ast { t_Num, val: i64($1) }

<> integer > 
        c:num(+)