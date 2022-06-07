/* 
 * Copyright (C) 2021 Anthony Weathersby - The Hydrocarbon Parser Compiler
 * see /source/typescript/hydrocarbon.ts for full copyright and warranty 
 * disclaimer notice.
 */

import {
    ByteReader,
    goto_state_mask, instruction_pointer_mask, KernelToken, normal_state_mask,
    skipped_scan_prod,
    state_index_mask
} from '@hctoolkit/common';
import { KernelStack } from './stack.js';

let peek: (a: number, b: StateIterator) => void = _ => _;

export function assign_peek(fn: (a: number, b: StateIterator) => void) {
    peek = fn;
}

export interface ParseAction {

    [ParseActionType.REDUCE]: {
        type: ParseActionType.REDUCE;
        body: number,
        length: number,
        production: number;
    };

    [ParseActionType.LAZY]: {
        type: ParseActionType.LAZY;
        state_pointer: number,
        line: number,
        offset: number,
        length: number,
    };

    [ParseActionType.ACCEPT]: {
        type: ParseActionType.ACCEPT;
    };

    [ParseActionType.ERROR]: {
        type: ParseActionType.ERROR;
        tk_type: number,
        tk_offset: number,
        tk_length: number,
        last_state: number,
        production: number;
    };

    [ParseActionType.SHIFT]: {
        type: ParseActionType.SHIFT;
        token: KernelToken;
    };

    [ParseActionType.SKIP]: {
        type: ParseActionType.SKIP;
        offset: number;
        length: number;
        line: number;
        token_type: number;
    };

    [ParseActionType.TOKEN]: {
        type: ParseActionType.TOKEN;
        token: KernelToken;

    };

    [ParseActionType.FORK]: {
        type: ParseActionType.FORK;
    };
}

export const enum ParseActionType {
    REDUCE,

    SHIFT,

    SKIP,

    ACCEPT,

    ERROR,

    FORK,

    TOKEN,

    LAZY
}


/**
 * This is the core class that drives the recognizer system.
 *
 * It yields Tokens and reduce information that can be consumed
 * by completers to apply parse actions on the token values
 */
export class StateIterator {

    stack: KernelStack;

    reader: ByteReader;

    /**
     * Used if the rate of rule creation exceeds rule output
     */
    buffer: ParseAction[ParseActionType][];

    tokens: KernelToken[];

    bytecode: Uint32Array;

    symbol_accumulator: number;

    ACTION_BUFFER_EMPTY: boolean;

    production_id: number;

    SCANNER: boolean;

    forks?: Uint32Array[];

    handler: (arg: ParseAction[ParseActionType]) => void;

    fork_handler?: (arg: ParseAction[ParseActionType], iterator: StateIterator) => ParseAction[ParseActionType][];

    constructor(
        base_byte_reader: ByteReader,
        bytecode_buffer: Uint32Array,
        entry_pointer: number,
        SCANNER: boolean
    ) {

        this.reader = base_byte_reader;

        this.bytecode = bytecode_buffer;

        this.stack = new KernelStack();

        this.stack.reset(entry_pointer);

        this.ACTION_BUFFER_EMPTY = true;

        this.SCANNER = SCANNER;

        this.production_id = -1;

        this.symbol_accumulator = 0;

        this.buffer = [];

        this.handler = () => { };

        this.tokens = [
            KernelToken.new(),
            KernelToken.new()
        ];
    }


    start(
        handler: (arg: ParseAction[ParseActionType]) => void,
        fork_handler: (arg: ParseAction[ParseActionType], iterator: StateIterator) => ParseAction[ParseActionType][],
        root: number = -1
    ) {

        this.handler = handler;
        this.fork_handler = fork_handler;

        let fail_mode = false;

        let last_good_state = -1;

        if (this.SCANNER) {
            let token = this.tokens[0];
            token.line_number = this.reader.line_count();
            token.line_offset = this.reader.line_offset();
        }

        while (true) {

            if (this.stack.pointer < 1) {

                const token = this.tokens[0];
                const advanced = this.tokens[1];

                if (this.SCANNER) {
                    //token.byte_length = advanced.byte_offset - token.byte_offset;
                    //token.codepoint_length = advanced.codepoint_offset - token.codepoint_offset;
                    this.emit({
                        type: ParseActionType.TOKEN,
                        token: token
                    });
                } else if (fail_mode) {
                    this.emit({
                        type: ParseActionType.ERROR,
                        last_state: last_good_state,
                        tk_type: advanced.type,
                        tk_offset: advanced.codepoint_offset,
                        tk_length: advanced.codepoint_length,
                        production: this.production_id
                    });
                } else if (root >= 0 && this.production_id == root) {
                    this.emit({
                        type: ParseActionType.ACCEPT
                    });
                } else if (this.reader.offset_at_end(this.tokens[1].byte_offset)) {
                    this.emit({
                        type: ParseActionType.ACCEPT
                    });

                } else
                    this.emit({
                        type: ParseActionType.ERROR,
                        last_state: last_good_state,
                        tk_type: advanced.type,
                        tk_offset: advanced.codepoint_offset,
                        tk_length: advanced.codepoint_length,
                        production: this.production_id
                    });

                return;
            }

            // Hint to the compiler to inline this section 4 times
            const state = this.stack.pop_state();

            if (state > 0) {

                const mask_gate = normal_state_mask << +fail_mode;

                if (fail_mode) {
                    if (state & goto_state_mask) {
                        //const production = this.bytecode[(state & state_index_mask) - 1];
                    }
                } else {
                    last_good_state = state;
                }

                if (state & mask_gate) {

                    this.peek(state);

                    fail_mode = this.instruction_executor(
                        state,
                        fail_mode
                    );

                }
            }
        }
    }

    protected emit(action: ParseAction[ParseActionType]) {
        this.handler(action);
    }

    protected emitShift() {

        this.ACTION_BUFFER_EMPTY = false;

        this.symbol_accumulator++;

        const token = this.tokens[1];

        const prev_token = this.tokens[0];

        if (prev_token.byte_offset + prev_token.byte_length != token.byte_offset) {

            this.emit({
                type: ParseActionType.SKIP,
                offset: prev_token.codepoint_offset,
                length: token.codepoint_offset - (prev_token.codepoint_length + prev_token.codepoint_offset),
                line: prev_token.line_number,
                token_type: 0
            });
        }

        this.emit({ type: ParseActionType.SHIFT, token });
    }



    protected consume(instruction: number) {

        const token = this.tokens[1];

        if (instruction & 1) { //Consume nothing

            token.codepoint_length = 0;

            token.byte_length = 0;
        }

        if (this.SCANNER) {

            this.reader.next(token.byte_length);

            token.codepoint_offset += token.codepoint_length;

            token.byte_offset += token.byte_length;

            token.codepoint_length = 0;

            token.byte_length = 0;

            token.type = 0;

        } else {

            this.emitShift();

            this.reader.next(token.byte_length);

            token.codepoint_offset += token.codepoint_length;

            token.byte_offset += token.byte_length;

            token.codepoint_length = 0;

            token.byte_length = 0;

            token.type = 0;

            this.tokens[0].impersonate(token);
        }
    }

    protected emitReduce(symbol_length: number, body_id: number) {

        this.ACTION_BUFFER_EMPTY = false;

        this.emit({
            type: ParseActionType.REDUCE,
            body: body_id,
            length: symbol_length,
            production: this.production_id
        });
    }


    protected reduce(instruction: number, index: number, recover_data: number) {
        let body_id = (instruction) & 0xFFFF;
        let length = ((instruction >> 16) & 0xFFF);

        if ((body_id & 0xFFFF) == 0xFFFF) {

            let accumulated_symbols = this.symbol_accumulator
                -
                (recover_data & 0xFFFF);

            let len = accumulated_symbols;

            let fn_id = (instruction >> 16) & 0x0FFF;

            //Extract accumulated symbols inform
            this.emitReduce(len, fn_id);
        } else {
            this.symbol_accumulator -= length - 1;
            this.emitReduce(length, body_id);
        }

        //TODO: ASSERT the production is ALWAYS set after a reduction
        if ((this.bytecode[index] & 0xF0000000) == 0x30000000) {
            this.set_production(this.bytecode[index]);
            return index + 1;
        }

        return index;
    }


    peek(state: number) {
        if (peek) peek(state, this);
    }

    /* nextAction(): ParseAction[ParseActionType] {

        if (this.buffer.length > 0)
            return this.buffer.shift();
        this.ACTION_BUFFER_EMPTY = true;

        //Kernel
        //Input
        let fail_mode = false;

        while (this.ACTION_BUFFER_EMPTY) {
            if (this.stack.pointer < 1) {

                const token = this.tokens[0];
                const advanced = this.tokens[1];

                token.byte_length = advanced.byte_offset - token.byte_offset;
                token.codepoint_length = advanced.codepoint_offset - token.codepoint_offset;


                if (this.SCANNER) {
                    return {
                        type: ParseActionType.TOKEN,
                        token: token
                    };
                }

                if (this.reader.offset_at_end(this.tokens[1].byte_offset))
                    return {
                        type: ParseActionType.ACCEPT
                    };

                else return {
                    type: ParseActionType.ERROR,
                    production: this.production_id
                };

                return this.nextAction();
            }

            // Hint to the compiler to inline this section 4 times
            const state = this.stack.pop_state();

            if (state > 0) {
                const mask_gate = normal_state_mask << +fail_mode;

                if (state & mask_gate) {

                    this.peek(state);

                    fail_mode = this.instruction_executor(
                        state,
                        fail_mode
                    );

                }
            }
        }

        return this.nextAction();
    } */

    protected instruction_executor(
        state_pointer: number,
        fail_mode: boolean,
    ): boolean {

        let index = state_pointer & state_index_mask;

        let recover_data = this.stack.meta_stack[this.stack.pointer + 1];

        while (true) {

            const bytecode = this.bytecode;

            //console.log(`instr: ${(bytecode[index] >>> 28) | 0} address: ${index} off: ${this.reader.cursor} bl: ${this.tokens[1].byte_length}`);

            const instruction = this.bytecode[index];

            index += 1;

            switch ((instruction >> 28) & 0xF) {

                case 0: default: return this.pass();

                case 1: this.consume(instruction); break;

                case 2: index = this.goto(instruction, index); break;

                case 3: this.set_production(instruction); break;

                case 4: index = this.reduce(instruction, index, recover_data); break;

                case 5: index = this.set_token(instruction, index); break;

                case 6: index = this.fork(instruction, index); break;

                case 7: index = this.scan_to(index, instruction); break;

                case 8:/*NOOP*/;

                case 9: index = this.index_jump(index, instruction); break;

                case 10: index = this.hash_jump(index, instruction); break;

                case 11: this.push_fail_state(instruction); break;

                case 12: index = this.repeat(index, instruction); break;

                case 13: index = this.try_lazy(index, instruction); break;

                case 14: index = this.assert_consume(index, instruction); break;

                case 15: return this.advanced_return(instruction, fail_mode);
            }
        }
    }

    protected try_lazy(index: number, instruction: number) {
        /* let production_pointer = instruction;
        let sentinel_open = this.bytecode[index];
        let sentinel_close = this.bytecode[index + 1];

        let counter = 1;

        let scan_ahead = this.reader.clone();

        let byte_offset = this.tokens[1].byte_offset;
        let cp_offset = this.tokens[1].codepoint_offset;
        let line = this.tokens[1].line;

        while (!scan_ahead.END()) {

            const cp = scan_ahead.codepoint();

            byte_offset += scan_ahead.codepoint_byte_length();
            cp_offset += 1;

            if (cp == sentinel_open)
                counter++;

            if (cp == sentinel_close)
                counter--;

            if (scan_ahead.class() == TokenTypes.NEW_LINE)
                line += 1;

            if (counter <= 0)
                break;

            scan_ahead.next(byte_offset);
        }

        if (counter == 0) {

            this.reader.setCursorTo(byte_offset);

            this.ACTION_BUFFER_EMPTY = false;

            this.buffer.push({
                type: ParseActionType.LAZY,
                state_pointer: production_pointer,
                line: this.tokens[1].line,
                offset: this.tokens[1].byte_offset,
                length: byte_offset - this.tokens[1].byte_offset,
            });

            this.tokens[0].byte_offset = byte_offset;
            this.tokens[0].codepoint_offset = cp_offset;
            this.tokens[0].codepoint_length = 1;
            this.tokens[0].byte_length = 1;
            this.tokens[0].line += line;

            this.tokens[1].byte_offset = byte_offset;
            this.tokens[1].codepoint_offset = cp_offset;
            this.tokens[1].codepoint_length = 1;
            this.tokens[1].byte_length = 1;
            this.tokens[1].line += line;
        } */

        return index + 2;
    }

    protected assert_consume(index: number, instruction: number) {

        const mode = instruction & 0x0F000000;
        let val = instruction & 0x00FFFFFF;

        let token = this.tokens[1];

        switch (mode) {
            case 0x00000000: //CLASS
                token.byte_length = this.reader.codepoint_byte_length();
                token.codepoint_length = this.reader.codepoint_length();
                if (val != this.reader.class())
                    return 2;
                break;
            case 0x01000000://CODEPOINT
                token.byte_length = this.reader.codepoint_byte_length();
                token.codepoint_length = this.reader.codepoint_length();
                if (val != this.reader.codepoint())
                    return 2;
                break;
            case 0x02000000://BYTE
                token.byte_length = 1;
                token.codepoint_length = 1;
                if (val != this.reader.byte())
                    return 2;
                break;
        }

        this.consume(0);

        return index;

    }

    protected pass() {
        return false;
    }

    protected advanced_return(instruction: number, fail_mode: boolean): boolean {

        if (instruction & 1)
            return fail_mode;

        return true;
    }

    protected set_production(instruction: number) {

        this.production_id = instruction & 0xFFFFFFF;
    }


    protected scan_to(index: number, instruction: number): number {


        let length = instruction & 0xFFFF;

        const scanner_index = this.bytecode[index];

        const temp_token = this.tokens[1].copy();
        const prev_token = this.tokens[0];

        temp_token.type = 0;

        index += 1;

        const scan_back = (instruction & 0x00100000) > 0;

        let RUN = true;

        let start = index;

        let end = index + length;

        index += length;

        if (scan_back) {
            temp_token.byte_offset = prev_token.byte_offset;
            temp_token.codepoint_offset = prev_token.codepoint_offset;
            temp_token.byte_length = 0;
            temp_token.codepoint_length = 0;
        }

        while (RUN) {

            const token = this.scanner(temp_token, scanner_index);

            for (let i = start; i < end; i++) {

                if (token.type == this.bytecode[i]) {
                    RUN = false;
                    break;
                }
            }

            if (!RUN) break;

            if (this.reader.offset_at_end(token.byte_offset))
                return 1;

            if (token.byte_length > 0) {
                temp_token.byte_offset += token.byte_length;
                temp_token.codepoint_offset += token.codepoint_offset;
            } else {
                temp_token.byte_offset += 1;
                temp_token.codepoint_offset += 1;
            }
            temp_token.byte_length = 0;
            temp_token.codepoint_length = 0;
        }

        if (!scan_back) {
            //Shift any tokens that may have been encountered?
            this.tokens[1].impersonate(temp_token);
        }

        return index;
    }

    protected error() {
        this.ACTION_BUFFER_EMPTY = false;
    }

    protected get_input_value(
        input_type: number,
        token_transition: number,
        scanner_start_pointer: number,
    ): number {

        if (input_type > 0) { // Lexer token id input

            switch (token_transition) {
                case 1: /* set next peek lexer */ {


                    const token = this.tokens[this.tokens.length - 1];

                    this.reader.next(token.byte_length);

                    if (this.reader.END())
                        return 0;

                    let new_token = token.copy();

                    this.tokens.push(new_token);

                    new_token.type = 0;
                    new_token.byte_offset = token.byte_offset + token.byte_length;

                    switch (input_type) {

                        case 1:
                            return this.scanner(new_token, scanner_start_pointer).type;
                        case 2:
                            new_token.byte_length = this.reader.codepoint_byte_length();
                            new_token.codepoint_length = this.reader.codepoint_length();
                            return this.reader.class();
                        case 3:
                            new_token.byte_length = this.reader.codepoint_byte_length();
                            new_token.codepoint_length = this.reader.codepoint_length();
                            return this.reader.codepoint();
                        case 4:
                            new_token.byte_length = 1;
                            new_token.codepoint_length = 1;
                            return this.reader.byte();
                    }

                } break;

                default: {/* set primary lexer */

                    if (this.reader.END())
                        return 1;

                    const token = this.tokens[1];

                    if (this.tokens.length > 2) {

                        this.tokens.length = 2;

                        if (!this.reader.setCursorTo(token)) {

                            this.error();

                            return 0;
                        };

                        token.type = 0;
                    }

                    switch (input_type) {
                        case 1:
                            this.tokens[1] = this.scanner(token, scanner_start_pointer);
                            return this.tokens[1].type;
                        case 2:
                            token.byte_length = this.reader.codepoint_byte_length();
                            token.codepoint_length = this.reader.codepoint_length();
                            return this.reader.class();
                        case 3:
                            token.byte_length = this.reader.codepoint_byte_length();
                            token.codepoint_length = this.reader.codepoint_length();
                            return this.reader.codepoint();
                        case 4:
                            token.byte_length = 1;
                            token.codepoint_length = 1;
                            return this.reader.byte();
                    }
                    break;
                }
            }

        } else {
            // Production id input
            return this.production_id;
        }

        return 0;
    }
    protected hash_jump(index: number, instruction: number): number {

        const input_type = ((instruction >> 22) & 0x7);

        const lexer_type = ((instruction >> 26) & 0x3);

        let scanner_start_pointer = this.bytecode[index];

        let table_data = this.bytecode[index + 1];

        index += 2;

        const modulus = (1 << ((table_data >>> 16) & 0xFFFF)) - 1;

        const table_size = (table_data) & 0xFFFF;

        const hash_table_start = index;

        const instruction_field_start = hash_table_start + table_size;

        const instruction_field_size = instruction & 0xFFFF;

        let input_value =
            this.get_input_value(
                input_type,
                lexer_type,
                scanner_start_pointer,
            );
        let hash_index = input_value & modulus;

        while (true) {

            const cell = this.bytecode[hash_table_start + hash_index];

            const value = cell & 0x7FF;

            const next = ((cell >>> 22) & 0x3FF) - 512;

            if (value == input_value) {

                const instruction_start = (cell >> 11) & 0x7FF;

                return instruction_field_start + instruction_start;
            }

            if (next == 0) {
                //Failure
                return instruction_field_size + instruction_field_start;
            }

            hash_index += next;
        }

        return index;
    }
    protected index_jump(index: number, instruction: number) {

        let scanner_start_pointer = this.bytecode[index];

        let table_data = this.bytecode[index + 1];

        let basis__ = instruction & 0xFFFF;

        const input_type = ((instruction >> 22) & 0x7);

        const lexer_type = ((instruction >> 26) & 0x3);

        let input_value = this.get_input_value(
            input_type,
            lexer_type,
            scanner_start_pointer,
        ) - basis__;

        let number_of_rows = table_data >> 16;

        let row_size = table_data & 0xFFFF;

        index += 2;

        if (input_value >= 0 && input_value < number_of_rows) {

            return index + input_value * row_size + row_size;

        } else {
            // Use default behavior found at the beginning of the 
            // jump table
            return index;
        }
    }

    protected set_token(instruction: number, index: number) {

        const value = instruction & 0xFFFFFF;

        if (instruction & 0x01000000)
            this.consume(0);

        if (instruction & 0x08000000) {

            this.production_id = value;
            this.tokens[0].type = value;
            this.tokens[0].byte_length = this.tokens[1].byte_offset - this.tokens[0].byte_offset;
            this.tokens[0].codepoint_length = this.tokens[1].codepoint_offset - this.tokens[0].codepoint_offset;

        } else {

            const token = this.tokens[1];

            token.codepoint_length = value;

            token.byte_length = value;
        }

        return index;

    }

    protected goto(instruction: number, index: number) {
        this.stack.push_state(instruction, this.symbol_accumulator);
        while ((this.bytecode[index] & 0xF0000000) == 0x20000000) {
            this.stack.push_state(this.bytecode[index], this.symbol_accumulator);
            index++;
        }
        return index;
    }

    protected repeat(index: number, instruction: number) {

        const origin_offset = 0xFFFFFFF & instruction;

        index -= (origin_offset);

        return index;
    }

    protected push_fail_state(instruction: number) {

        let fail_state_pointer = (instruction) >>> 0;
        const current_state = (this.stack.read_state() & instruction_pointer_mask);
        //Only need to set new failure state if the previous state
        //Is not identical to the pending fail state.
        if (current_state != ((fail_state_pointer >>> 0) & instruction_pointer_mask)) {

            this.stack.push_state(fail_state_pointer >>> 0, this.symbol_accumulator);

        } else {
            this.stack.swap_state(fail_state_pointer);
        }
    }



    protected scanner(
        current_token: KernelToken,
        scanner_start_pointer: number
    ): KernelToken {

        if (true || current_token.type <= 0) {
            const clone = this.reader.clone();
            clone.setCursorTo(current_token);

            const scanner = new StateIterator(
                clone,
                this.bytecode,
                scanner_start_pointer,
                true
            );
            scanner.tokens[1].impersonate(current_token);
            scanner.tokens[0].impersonate(current_token);

            let ACTIVE = true;

            while (ACTIVE) {
                scanner.start(result => {

                    switch (result.type) {

                        case ParseActionType.TOKEN: {

                            const token = result.token;

                            if (token.type == skipped_scan_prod) {

                                current_token.codepoint_offset += token.codepoint_length;
                                current_token.byte_offset += token.byte_length;
                                current_token.line_offset = token.line_offset;
                                current_token.line_number = token.line_number;

                                //Need to reset the state iterator 

                                scanner.stack.reset(scanner_start_pointer);
                                scanner.tokens[0].impersonate(scanner.tokens[1]);

                                return;
                            }

                            current_token.codepoint_length = token.codepoint_length;
                            current_token.byte_length = token.byte_length;
                            current_token.type = token.type;
                            current_token.line_offset = token.line_offset;
                            current_token.line_number = token.line_number;

                            ACTIVE = false;
                        } break;

                        case ParseActionType.FORK: {

                            ACTIVE = false;

                            throw new Error("Scanner Fork Not Implemented");
                        }

                        case ParseActionType.ACCEPT: {
                            ACTIVE = false;
                        }

                        default:
                            ACTIVE = false;

                            throw new Error("Scanner Yielded Invalid Action");
                    }
                }, a => []);
            }
        }

        return current_token;
    }

    protected fork(
        instruction: number,
        index: number,
    ): number {

        this.ACTION_BUFFER_EMPTY = false;

        const buffer = this.fork_handler({
            type: ParseActionType.FORK,
            pointer: index - 1,
        }, this);

        if (buffer.length > 0) {
            for (let action of buffer) {
                this.emit(action);
            }
            return 1;
        } else {
            return 0;
        }



        // Push fork data up to the fork array and return 

        /* let valid = new KernelStateBuffer;
        let invalid = new KernelStateBuffer;
        let process_buffer = new KernelStateBuffer;
        let pointer = origin_kernel_state.stack_pointer;

        let length = (instruction & 0xFFFFFFF);
        let depth = 0;

        //Include the GOTO state and its fail state
        if (origin_kernel_state.state_stack[pointer] & goto_state_mask) {
            depth = 1;
            if (origin_kernel_state.state_stack[pointer - 1] & fail_state_mask) {
                depth = 2;
            }
        }

        while (length-- > 0) {

            let kernel_state = origin_kernel_state.fork(process_buffer, depth);

            kernel_state.push_state(origin_kernel_state.instructions[index]);

            index += 1;
        }

        executeParseProcess(process_buffer, invalid, valid, false);

        if (valid.len() > 0) {

            if (valid.len() == 1) {


                origin_kernel_state.FORKED = true;


                //Continue Parsing from the end of the previous KernelState
                const tip = valid.data[0];

                origin_kernel_state.next.push(tip);

                //Synch tip with the origin_kernel_state
                origin_kernel_state.transfer_state_stack(tip);

                //Set the tip to point to the next set of instructions
                //after the fork.
                tip.push_state(normal_state_mask | index);

                origin_kernel_state_repo.add_state_pointer_and_sort(tip);

                //Set index so that it points to the pass instruction block;
                index = 0;

            } else {

                let furthest_byte = valid.data[0].lexer.byte_offset;

                let furthest_index = 0;

                let furthest_matching_count = 1;

                for (let i = 1; i < valid.len(); i++)

                    if (valid.data[i].lexer.byte_offset != furthest_byte) {
                        // Extract the longest parsers
                        for (let i = 1; i < valid.len(); i++) {
                            let len = valid.data[i].lexer.byte_offset;

                            if (len > furthest_byte) {
                                furthest_byte = len;
                                furthest_index = i;
                                furthest_matching_count = 1;
                            } else if (len == furthest_byte)
                                furthest_matching_count += 1;
                        }

                        if (furthest_matching_count == 1) {

                            origin_kernel_state.FORKED = true;
                            //Continue Parsing from the end of the previous KernelState
                            const tip = valid.data[furthest_index];

                            origin_kernel_state.next.push(tip);

                            //Synch tip with the origin_kernel_state
                            origin_kernel_state.transfer_state_stack(tip);

                            //Set the tip to point to the next set of instructions
                            //after the fork.
                            tip.push_state(index);

                            origin_kernel_state_repo.add_state_pointer_and_sort(tip);

                            //Set index so that it points to the null instruction block;
                            index = 0;

                        } else {
                            throw new Error("Multiple uneven parse paths exist, no resolution mechanism has been implemented for this situation. Exiting");
                        }
                        break;
                    } else if (i == valid.len() - 1) {

                        throw new Error("Multiple even parse paths exist, no resolution mechanism has been implemented for this situation. Exiting");
                    }

            }
        } else {
            const tip = invalid.get_mut_state(0);

            origin_kernel_state.next.push(tip);

            origin_kernel_state.transfer_state_stack(tip);

            tip.push_state(normal_state_mask | fail_state_mask | 1);

            origin_kernel_state_repo.add_state(tip);

            return 0;
        }

        // Link valid states to the origin state
        // Then create a new forward state the linked states reference.

        return index;
    } */
    }
}


export interface DebugState {
    state: number;
    ACTIVE: boolean,
    stack: KernelStack;
    tokens: KernelToken[];

    symbol_accumulator: number;

    production_id: number;

    fail_mode: boolean;

    last_good_state: number;

    byte_reader: ByteReader,

    actions: ParseAction[ParseActionType][];
}

export class DebugIterator extends StateIterator {

    active_debug_state: DebugState;

    constructor(
        base_byte_reader: ByteReader,
        bytecode_buffer: Uint32Array,
        entry_pointer: number,
    ) {
        super(
            base_byte_reader,
            bytecode_buffer,
            entry_pointer,
            false
        );

        this.active_debug_state = {
            state: entry_pointer & state_index_mask,
            fail_mode: false,
            last_good_state: -1,
            ACTIVE: true,
            production_id: -1,
            actions: [],
            tokens: this.tokens,
            symbol_accumulator: this.symbol_accumulator,
            byte_reader: this.reader,
            stack: this.stack
        };
    }


    protected emit(action: ParseAction[ParseActionType]) {
        if (this.active_debug_state)
            this.active_debug_state.actions.push(action);
    }


    next_state(
        prev_state: DebugState,
        /**
         * The numeric value of the root (goal) production
         */
        goal_production: number = -1
    ): DebugState {

        let fail_mode = prev_state.fail_mode;
        let last_good_state = prev_state.last_good_state;

        this.reader = prev_state.byte_reader.clone();
        this.production_id = prev_state.production_id;
        this.tokens = prev_state.tokens.slice().map(t => t.copy());
        this.stack = prev_state.stack.clone();

        this.active_debug_state = {
            state: 0,
            ACTIVE: true,
            fail_mode: false,
            last_good_state: -1,
            tokens: this.tokens,
            actions: [],
            production_id: -1,
            symbol_accumulator: -1,
            byte_reader: this.reader,
            stack: this.stack
        };

        if (this.stack.pointer < 1) {

            const token = this.tokens[0];
            const advanced = this.tokens[1];

            if (this.SCANNER) {
                //token.byte_length = advanced.byte_offset - token.byte_offset;
                //token.codepoint_length = advanced.codepoint_offset - token.codepoint_offset;
                this.emit({
                    type: ParseActionType.TOKEN,
                    token: token
                });
            } else if (fail_mode) {
                this.emit({
                    type: ParseActionType.ERROR,
                    last_state: last_good_state,
                    tk_type: advanced.type,
                    tk_offset: advanced.codepoint_offset,
                    tk_length: advanced.codepoint_length,
                    production: this.production_id
                });
            } else if (goal_production >= 0 && this.production_id == goal_production) {
                this.emit({
                    type: ParseActionType.ACCEPT
                });
            } else if (this.reader.offset_at_end(this.tokens[1].byte_offset)) {
                this.emit({
                    type: ParseActionType.ACCEPT
                });

            } else
                this.emit({
                    type: ParseActionType.ERROR,
                    last_state: last_good_state,
                    tk_type: advanced.type,
                    tk_offset: advanced.codepoint_offset,
                    tk_length: advanced.codepoint_length,
                    production: this.production_id
                });

            this.active_debug_state.ACTIVE = false;

        } else {


            // Hint to the compiler to inline this section 4 times
            const state = this.stack.pop_state();

            this.active_debug_state.state = state & state_index_mask;

            if (state > 0) {

                const mask_gate = normal_state_mask << +fail_mode;

                if (fail_mode) {
                    if (state & goto_state_mask) {
                        //const production = this.bytecode[(state & state_index_mask) - 1];
                    }
                } else {
                    this.active_debug_state.last_good_state = state;
                }

                if (state & mask_gate) {

                    fail_mode = this.instruction_executor(
                        state,
                        fail_mode
                    );

                    this.active_debug_state.fail_mode = fail_mode;

                }
            }
        }

        this.active_debug_state.production_id = this.production_id;

        this.active_debug_state.stack = this.stack.debug_clone();

        Object.freeze(this.active_debug_state);

        return this.active_debug_state;
    }
}