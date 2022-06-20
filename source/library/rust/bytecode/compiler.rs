use super::constants::default_get_branch_selector;
use super::constants::GetBranchSelector;
use super::constants::DEFAULT_CASE_INDICATOR;
use super::constants::INPUT_TYPE;
use super::constants::INSTRUCTION;
use crate::bytecode::constants::BranchSelector;
use crate::bytecode::constants::GOTO_STATE_MASK;
use crate::bytecode::constants::INSTRUCTION_CONTENT_MASK;
use crate::bytecode::constants::IR_REDUCE_NUMERIC_LEN_ID;
use crate::bytecode::constants::LEXER_TYPE;
use crate::bytecode::constants::NORMAL_STATE_MASK;
use crate::bytecode::constants::STATE_INDEX_MASK;
use crate::grammar::data::ast::ASTNode;
use crate::grammar::data::ast::Consume;
use crate::grammar::data::ast::ForkTo;
use crate::grammar::data::ast::Goto;
use crate::grammar::data::ast::NotInScope;
use crate::grammar::data::ast::Num;
use crate::grammar::data::ast::Pass;
use crate::grammar::data::ast::Reduce;
use crate::grammar::data::ast::ScanUntil;
use crate::grammar::data::ast::SetProd;
use crate::grammar::data::ast::SetScope;
use crate::grammar::data::ast::TokenAssign;
use crate::grammar::data::ast::ASSERT;
use crate::grammar::data::ast::HASH_NAME;
use crate::grammar::data::ast::IR_STATE;
use crate::grammar::parse::compile_ir_ast;
use crate::intermediate::state_construct::generate_production_states;
use crate::primitives::grammar::GrammarStore;
use crate::primitives::IRStateString;
use crate::primitives::ProductionId;
use std::any::Any;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::default;
use std::hash;
use std::iter::Filter;
use std::thread::{self};
use std::vec;

pub fn build_byte_code_buffer(
    states: Vec<&IR_STATE>,
) -> (Vec<u32>, BTreeMap<u32, String>)
{
    let states_iter = states
        .iter()
        .enumerate()
        .map(|(i, s)| (s, s.id.clone(), i as u32));

    let mut goto_bookmarks_to_offset =
        states_iter.clone().map(|_| 0).collect::<Vec<_>>();

    let state_name_to_bookmark = states_iter
        .clone()
        .map(|(_, s, i)| (s, i))
        .collect::<HashMap<_, _>>();

    let mut bytecode = vec![
        INSTRUCTION::I15_FALL_THROUGH,
        INSTRUCTION::I00_PASS,
        INSTRUCTION::I15_FAIL,
        INSTRUCTION::I08_NOOP,
        NORMAL_STATE_MASK,
        INSTRUCTION::I00_PASS,
    ];

    for ((state, name, i)) in states_iter {
        goto_bookmarks_to_offset[i as usize] = bytecode.len() as u32;
        bytecode.append(&mut compile_ir_state_to_bytecode(
            state,
            default_get_branch_selector,
            &state_name_to_bookmark,
        ));
    }

    patch_goto_offsets(&mut bytecode, &goto_bookmarks_to_offset);

    let offset_to_state_name = state_name_to_bookmark
        .into_iter()
        .map(|(name, bookmark)| {
            (goto_bookmarks_to_offset[bookmark as usize], name)
        })
        .collect::<BTreeMap<_, _>>();

    (bytecode, offset_to_state_name)
}

/// Converts Goto location bookmarks to bytecode offsets.
fn patch_goto_offsets(
    bytecode: &mut Vec<u32>,
    goto_bookmarks_to_offset: &Vec<u32>,
)
{
    use crate::bytecode::constants::INSTRUCTION as I;

    let mut index = 0;

    while index < bytecode.len() {
        let instruction = bytecode[index];
        index += match instruction & 0xF000_0000 {
            I::I02_GOTO => {
                let bytecode_bookmark = instruction & STATE_INDEX_MASK;
                let state_header = instruction & (!STATE_INDEX_MASK);
                bytecode[index] = state_header
                    | goto_bookmarks_to_offset[bytecode_bookmark as usize];
                1
            }
            I::I06_FORK_TO => 1,
            I::I09_VECTOR_BRANCH | I::I10_HASH_BRANCH => {
                let table_length = bytecode[index + 2] >> 16 & 0xFFFF;
                table_length as usize + 4
            }
            _ => 1,
        }
    }
}

fn compile_ir_states_worker(
    grammar: &GrammarStore,
    productions: &[ProductionId],
) -> Vec<IRStateString>
{
    productions
        .into_iter()
        .map(|p| generate_production_states(p, grammar))
        .flatten()
        .collect::<Vec<_>>()
}

/// Builds ir states for every standard production in
/// a grammar.

fn compile_ir_states(
    grammar: &GrammarStore,
    work_group: &[ProductionId],
    number_of_threads: usize,
) -> BTreeMap<u64, IRStateString>
{
    let production_id_chunks = work_group
        .chunks(number_of_threads)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    let mut ir_map = BTreeMap::new();

    for ir_state in thread::scope(|s| {
        production_id_chunks
            .iter()
            .map(|work| s.spawn(|| compile_ir_states_worker(grammar, work)))
            // Collect now to actually generate the threads
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(move |s| s.join().unwrap())
            .collect::<Vec<_>>()
    }) {
        let key = ir_state.get_hash();

        if !ir_map.contains_key(&key) {
            ir_map.insert(key, ir_state);
        }
    }

    ir_map
}

pub fn compile_ir_state_to_bytecode(
    state: &IR_STATE,
    get_branch_selector: GetBranchSelector,
    state_name_to_bookmark: &HashMap<String, u32>,
) -> Vec<u32>
{
    // Determine if we are dealing with a branch state or a single line
    // state. Branch states will always have more than one assert
    // statement.

    if is_branch_state(state) {
        // We are dealing with a branching state

        // For each branch we compile new vectors separately
        // Then we derive offset for each branch, build a suitable
        // storage structure (either HASH or JUMP) and combine all
        // that data into a single file

        // We first order the statements into blocks based on
        // on the assertion type. The order is
        // 1. Production
        // 2. Token
        // 3. Byte
        // 4. Class
        // 5. CodePoint

        build_branching_bytecode(
            &state.instructions,
            get_branch_selector,
            state_name_to_bookmark,
        )
    } else {
        // We are dealing with a standard non-branching state
        build_branchless_bytecode(&state.instructions, state_name_to_bookmark)
    }
}

fn is_branch_state(state: &IR_STATE) -> bool
{
    state.instructions.iter().all(|i| match i {
        ASTNode::ASSERT(_) => true,
        _ => false,
    })
}

fn build_branching_bytecode(
    instructions: &Vec<ASTNode>,
    get_branch_selector: GetBranchSelector,
    state_name_to_bookmark: &HashMap<String, u32>,
) -> Vec<u32>
{
    let branches = instructions
        .iter()
        .filter_map(|n| match n {
            ASTNode::ASSERT(assert) => {
                if let ASTNode::Num(box Num { val }) = assert.ids[0] {
                    if val as u32 == DEFAULT_CASE_INDICATOR {
                        None
                    } else {
                        Some(assert)
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let default_branches = instructions
        .iter()
        .filter_map(|n| match n {
            ASTNode::ASSERT(assert) => {
                if let ASTNode::Num(box Num { val }) = assert.ids[0] {
                    if val as u32 == DEFAULT_CASE_INDICATOR {
                        Some(assert)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    // Extract the default branch if it exists.

    let output = if default_branches.len() > 0 {
        build_branchless_bytecode(
            &default_branches[0].instructions,
            state_name_to_bookmark,
        )
    } else {
        vec![INSTRUCTION::I15_FAIL]
    };

    let output = make_table(
        branches
            .iter()
            .cloned()
            .filter(|p| p.mode == "BYTE")
            .collect::<Vec<_>>(),
        INPUT_TYPE::T05_BYTE,
        output,
        get_branch_selector,
        state_name_to_bookmark,
    );

    let output = make_table(
        branches
            .iter()
            .cloned()
            .filter(|p| p.mode == "CODEPOINT")
            .collect::<Vec<_>>(),
        INPUT_TYPE::T04_CODEPOINT,
        output,
        get_branch_selector,
        state_name_to_bookmark,
    );

    let output = make_table(
        branches
            .iter()
            .cloned()
            .filter(|p| p.mode == "CLASS")
            .collect::<Vec<_>>(),
        INPUT_TYPE::T03_CLASS,
        output,
        get_branch_selector,
        state_name_to_bookmark,
    );

    let output = make_table(
        branches
            .iter()
            .cloned()
            .filter(|p| p.mode == "TOKEN")
            .collect::<Vec<_>>(),
        INPUT_TYPE::T02_TOKEN,
        output,
        get_branch_selector,
        state_name_to_bookmark,
    );

    let output = make_table(
        branches
            .iter()
            .cloned()
            .filter(|p| p.mode == "PRODUCTION")
            .collect::<Vec<_>>(),
        INPUT_TYPE::T01_PRODUCTION,
        output,
        get_branch_selector,
        state_name_to_bookmark,
    );

    output
}

fn make_table(
    branches: Vec<&Box<ASSERT>>,
    input_type_key: u32,
    mut default: Vec<u32>,
    get_branch_selector: GetBranchSelector,
    state_name_to_bookmark: &HashMap<String, u32>,
) -> Vec<u32>
{
    if branches.is_empty() {
        return default;
    }

    use super::constants::INSTRUCTION as I;

    let lexer_type: u32 = if branches[0].is_peek {
        LEXER_TYPE::PEEK
    } else {
        LEXER_TYPE::ASSERT
    };
    let scanner_pointer = 0;

    let mut val_offset_map = branches
        .iter()
        .enumerate()
        .flat_map(|(index, assert)| {
            assert.ids.iter().filter_map(move |i| match i {
                ASTNode::Num(box Num { val }) => {
                    Some((*val as u32, index as u32))
                }
                _ => None,
            })
        })
        .collect::<BTreeMap<_, _>>();

    let max_span = {
        let mut val = 0;
        let mut prev = *val_offset_map.first_key_value().unwrap().0;
        for id in val_offset_map.keys().cloned() {
            val = u32::max(id - prev, val);
            prev = id;
        }
        val
    };

    let mut branch_instructions = vec![];
    let mut branch_instructions_length = 0;

    for branch in branches {
        if let ASTNode::Skip(_) = &branch.instructions[0] {
            for id in &branch.ids {
                if let ASTNode::Num(box Num { val }) = id {
                    val_offset_map.insert(*val as u32, 0xFFFF_FFFF as u32);
                }
            }
        } else {
            let mut instructions = build_branchless_bytecode(
                &branch.instructions,
                state_name_to_bookmark,
            );

            let offset = branch_instructions_length;
            branch_instructions_length += instructions.len() as u32;
            branch_instructions.push(instructions);

            for id in &branch.ids {
                if let ASTNode::Num(box Num { val }) = id {
                    val_offset_map.insert(*val as u32, offset as u32);
                }
            }
        }
    }

    let mut output = vec![];

    match get_branch_selector(
        &val_offset_map.keys().cloned().collect(),
        max_span,
        &branch_instructions,
    ) {
        BranchSelector::Hash => {
            let values = val_offset_map.keys().cloned().collect::<Vec<_>>();

            let offset_lookup_table_length = values.len() as u32;

            let instruction_field_start = 4 + offset_lookup_table_length;

            let default_offset =
                branch_instructions_length + instruction_field_start;

            let mut pending_pairs = val_offset_map
                .clone()
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        if v == 0xFFFF_FFFF {
                            0x7FF
                        } else {
                            v + instruction_field_start
                        },
                    )
                })
                .collect::<VecDeque<_>>();

            let mod_base = f64::log2(val_offset_map.len() as f64) as u32;

            let mod_mask = (1 << mod_base) - 1;

            let mut hash_entries = (0..pending_pairs.len())
                .into_iter()
                .map(|_| 0)
                .collect::<Vec<_>>();

            let mut leftover_pairs = vec![];

            // Distribute keys-vals with unique hashes into hash table
            // slots.
            while let Some(pair) = pending_pairs.pop_front() {
                let (val, offset) = pair;
                let hash_index = ((val as u32) & mod_mask) as usize;
                if hash_entries[hash_index] == 0 {
                    hash_entries[hash_index] = ((val as u32) & 0x7FF)
                        | ((offset & 0x7FF) << 11)
                        | (512 << 22);
                } else {
                    leftover_pairs.push(pair);
                }
            }

            // What remains are hash collisions. We use simple linear
            // probing to find the next available slot, and
            // attach it to the probing chain using a signed
            // delta index.
            for (val, offset) in leftover_pairs {
                let mut pointer = 0;
                let mut node = (val & mod_mask) as usize;

                loop {
                    pointer =
                        (((hash_entries[node] >> 22) & 0x3FF) as i32) - 512;

                    if pointer == 0 {
                        break;
                    } else {
                        node += pointer as usize;
                    }
                }

                for i in 0..hash_entries.len() {
                    if hash_entries[i] == 0 {
                        // Update the previous node in the chain with the
                        // diff pointer to the new node.
                        hash_entries[node] =
                            ((((i as i32 - node as i32) + 512) as u32 & 0x3FF)
                                << 22)
                                | (hash_entries[node] & ((1 << 22) - 1));
                        // Add data for the new node.
                        hash_entries[i] = ((val) & 0x7FF)
                            | ((offset & 0x7FF) << 11)
                            | (512 << 22);

                        break;
                    }
                }
            }

            // First word header
            output.push(
                I::I10_HASH_BRANCH
                    | (input_type_key << 22)
                    | (lexer_type << 26),
            );

            // Second word header
            output.push(scanner_pointer);

            // Third word header
            output.push((offset_lookup_table_length << 16) | mod_base);

            output.push(default_offset);

            output.append(&mut hash_entries);
        }
        BranchSelector::Vector => {
            let values = val_offset_map.keys().cloned().collect::<Vec<_>>();
            let (start, end) =
                (*values.first().unwrap(), *values.last().unwrap());
            let value_offset = start;
            let offset_lookup_table_length = values.len() as u32;
            let instruction_field_start = 4 + offset_lookup_table_length;
            let default_offset =
                branch_instructions_length + instruction_field_start;

            // First word header
            output.push(
                I::I09_VECTOR_BRANCH
                    | (input_type_key << 22)
                    | (lexer_type << 26),
            );

            // Second word header
            output.push(scanner_pointer);

            // Third word header
            output.push((offset_lookup_table_length << 16) | value_offset);

            // Default Location
            output.push(default_offset);

            for branch in start..=end {
                if let Some(offset) = val_offset_map.get(&branch) {
                    if *offset == 0xFFFF_FFFF {
                        output.push(*offset);
                    } else {
                        output.push(*offset + instruction_field_start);
                    }
                } else {
                    output.push(default_offset)
                }
            }
        }
    }

    output.append(&mut branch_instructions.into_iter().flatten().collect());

    output.append(&mut default);

    output
}

fn build_branchless_bytecode(
    instructions: &Vec<ASTNode>,
    state_name_to_bookmark: &HashMap<String, u32>,
) -> Vec<u32>
{
    let mut byte_code: Vec<u32> = vec![];
    use super::constants::INSTRUCTION as I;
    for instruction in instructions {
        match instruction {
            ASTNode::TokenAssign(box TokenAssign { ids }) => {
                if let ASTNode::Num(id) = &ids[0] {
                    byte_code.push(
                        I::I05_TOKEN_ASSIGN | ((id.val as u32) & 0x00FF_FFFF),
                    )
                }
            }
            ASTNode::Consume(box Consume { EMPTY }) => {
                byte_code.push(I::I01_CONSUME | *EMPTY as u32)
            }
            ASTNode::Goto(box Goto { state }) => {
                if let ASTNode::HASH_NAME(box HASH_NAME { val }) = state {
                    let state_pointer_val =
                        if let Some(v) = state_name_to_bookmark.get(val) {
                            *v
                        } else {
                            0
                        };
                    byte_code.push(
                        I::I02_GOTO | NORMAL_STATE_MASK | state_pointer_val,
                    );
                } else {
                    panic!("Invalid state type in goto instruction");
                }
            }
            ASTNode::ScanUntil(box ScanUntil {
                ids,
                SCAN_BACKWARDS,
            }) => {}
            ASTNode::ForkTo(box ForkTo {
                states,
                production_id,
            }) => {
                byte_code.push(
                    I::I06_FORK_TO
                        | ((states.len() << 16) as u32)
                        | (production_id.val as u32),
                );
                for state in states {
                    if let ASTNode::HASH_NAME(box HASH_NAME { val }) = state {
                        let state_pointer_val =
                            if let Some(v) = state_name_to_bookmark.get(val) {
                                *v
                            } else {
                                0
                            };
                        byte_code.push(
                            I::I02_GOTO | NORMAL_STATE_MASK | state_pointer_val,
                        );
                    } else {
                        panic!("Invalid state type in goto instruction");
                    }
                }
            }
            ASTNode::Skip(box Skip) => {}
            ASTNode::Pass(_) => byte_code.push(I::I00_PASS),
            ASTNode::Fail(_) => byte_code.push(I::I15_FAIL),
            ASTNode::NotInScope(box NotInScope { ids }) => {}
            ASTNode::SetScope(box SetScope { scope }) => {}
            ASTNode::SetProd(box SetProd { id }) => {
                if let ASTNode::Num(box Num { val }) = id {
                    byte_code.push(
                        I::I03_SET_PROD
                            | (*val as u32 & INSTRUCTION_CONTENT_MASK),
                    )
                }
            }
            ASTNode::Reduce(box Reduce { body_id, len, .. }) => byte_code.push(
                I::I04_REDUCE
                    | if *len as u32 == IR_REDUCE_NUMERIC_LEN_ID {
                        0xFFFF0000 & INSTRUCTION_CONTENT_MASK
                    } else {
                        (*len as u32) << 16
                    }
                    | (*body_id as u32),
            ),

            _ => {}
        }
    }

    if let Some(last) = byte_code.last() {
        if *last != I::I00_PASS && *last != I::I15_FAIL {
            byte_code.push(I::I00_PASS);
        }
    }

    byte_code
}

#[cfg(test)]
mod byte_code_creation_tests
{

    use std::collections::HashMap;

    use crate::bytecode::compiler::compile_ir_state_to_bytecode;
    use crate::bytecode::compiler::BranchSelector;
    use crate::bytecode::constants::default_get_branch_selector;
    use crate::debug::compile_test_grammar;
    use crate::grammar::data::ast::ASTNode;
    use crate::grammar::get_production_by_name;
    use crate::grammar::parse::compile_ir_ast;
    use crate::intermediate::state_construct::generate_production_states;

    #[test]
    pub fn test_produce_a_single_ir_ast_from_a_single_state_of_a_trivial_production(
    )
    {
        let grammar = compile_test_grammar("<> A > \\h");

        let prod_id = get_production_by_name("A", &grammar).unwrap();

        let result = generate_production_states(&prod_id, &grammar);

        println!("{:#?}", result);

        let state = result[0].get_code();

        let result = compile_ir_ast(Vec::from(state.as_bytes()));

        println!("{:#?}", result);

        assert!(result.is_ok());

        let result = compile_ir_state_to_bytecode(
            &result.unwrap(),
            default_get_branch_selector,
            &HashMap::new(),
        );

        println!("{:#?}", result);
    }
}
