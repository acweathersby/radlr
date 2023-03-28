use crate::{
  grammar::new::{
    parser::{
      ASTNode,
      DefaultMatch,
      IntMatch,
      Matches,
      NonTermMatch,
      ProductionMatches,
      Reduce,
      ReduceRaw,
      SetTokenId,
      Statement,
      TermMatch,
      TerminalMatches,
    },
    types::{Array, CachedString, IString, Map, ParseState, ParserDatabase, *},
  },
  Journal,
  SherpaResult,
};
use sherpa_runtime::types::{
  bytecode::{insert_op, Opcode, Opcode as Op, *},
  TableHeaderData,
};
use std::collections::VecDeque;

pub fn compile_bytecode<
  'db,
  T: IntoIterator<Item = (IString, Box<ParseState<'db>>)>,
>(
  db: &'db ParserDatabase,
  states: T,
) -> SherpaResult<(Array<u8>, Map<IString, usize>)> {
  let mut bytecode = Array::new();
  let mut state_name_to_proxy = OrderedMap::new();
  let mut state_name_to_address = Map::new();

  bytecode.append(&mut Array::from(bytecode_header()));

  for (name, state) in states {
    state_name_to_address.insert(name, bytecode.len());
    build_state(db, state.as_ref(), &mut bytecode, &mut state_name_to_proxy)?;
  }

  let proxy_to_address = state_name_to_proxy
    .into_iter()
    .map(|(name, proxy_address)| {
      (proxy_address as u32, *state_name_to_address.get(&name).unwrap() as u32)
    })
    .collect::<OrderedMap<_, _>>()
    .into_values()
    .collect();

  remap_goto_addresses(&mut bytecode, &proxy_to_address);

  SherpaResult::Ok((bytecode, state_name_to_address))
}

/// Converts Goto location bookmarks to bytecode addresses.
fn remap_goto_addresses(bc: &mut Array<u8>, _goto_to_off: &Array<u32>) {
  let mut i = bytecode_header().len();

  while i < bc.len() {
    let instruction = bc[i];
    let op = instruction.into();
    i += match op {
      Op::HashBranch => {
        let i: Instruction = (bc.as_slice(), i).into();
        let TableHeaderData {
          scan_block_instruction: scanner_address,
          parse_block_address,
          ..
        } = i.into();
        let default_delta = parse_block_address - i.address();

        if scanner_address.address() != u32::MAX as usize {
          set_goto_address(bc, _goto_to_off, i.address() + 6);
        }

        default_delta
      }
      Op::Goto | Op::PushGoto | Op::PushExceptionHandler => {
        set_goto_address(bc, _goto_to_off, i + 2);
        op.len()
      }
      op => op.len(),
    }
  }
}

fn set_goto_address(bc: &mut Vec<u8>, _goto_to_off: &[u32], offset: usize) {
  let mut iter: ByteCodeIterator = (bc.as_slice(), offset).into();
  let val = iter.next_u32_le().unwrap();
  set_u32_le(bc, offset, _goto_to_off[val as usize]);
}

fn build_state<'db>(
  db: &'db ParserDatabase,
  state: &ParseState<'db>,
  bytecode: &mut Array<u8>,
  state_name_to_proxy: &mut OrderedMap<IString, usize>,
) -> SherpaResult<()> {
  let state = state.get_ast()?;
  let stmt = &state.statement;
  build_statement(db, stmt.as_ref(), bytecode, state_name_to_proxy);
  SherpaResult::Ok(())
}

fn build_statement<'db>(
  db: &'db ParserDatabase,
  stmt: &Statement,
  bc: &mut Array<u8>,
  state_name_to_proxy: &mut OrderedMap<IString, usize>,
) -> SherpaResult<()> {
  let Statement { branch, non_branch, transitive } = stmt;

  if let Some(transitive) = transitive {
    match transitive {
      ASTNode::Skip(..) => insert_op(bc, Op::SkipToken),
      ASTNode::Pop(..) => insert_op(bc, Op::PopGoto),
      ASTNode::Scan(..) => insert_op(bc, Op::ScanShift),
      ASTNode::Reset(..) => insert_op(bc, Op::PeekReset),
      ASTNode::Shift(..) => insert_op(bc, Op::ShiftToken),
      ASTNode::Peek(..) => insert_op(bc, Op::PeekToken),
      node => {
        #[cfg(debug_assertions)]
        dbg!(node);
        unreachable!();
      }
    }
  }

  for non_branch in non_branch {
    match non_branch {
      ASTNode::ReduceRaw(box ReduceRaw { rule_id, len, prod_id, .. }) => {
        insert_op(bc, Op::Reduce);
        insert_u32_le(bc, *prod_id as u32);
        insert_u32_le(bc, *rule_id as u32);
        insert_u16_le(bc, *len as u16);
      }
      ASTNode::SetTokenId(box SetTokenId { id }) => {
        insert_op(bc, Op::AssignToken);
        insert_u32_le(bc, *id);
      }
      node => {
        #[cfg(debug_assertions)]
        dbg!(node);
        unreachable!();
      }
    }
  }

  if let Some(branch) = branch {
    match branch {
      ASTNode::Pass(..) => insert_op(bc, Op::Pass),
      ASTNode::Fail(..) => insert_op(bc, Op::Fail),
      ASTNode::Accept(..) => insert_op(bc, Op::Accept),
      ASTNode::Gotos(gotos) => {
        for push in &gotos.pushes {
          let proxy_address = get_proxy_address(
            push.prod.to_token().to_string().to_token(),
            state_name_to_proxy,
          );

          insert_op(bc, Op::PushGoto);
          insert_u8(bc, NORMAL_STATE_FLAG as u8);
          insert_u32_le(bc, proxy_address);
        }

        let proxy_address = get_proxy_address(
          gotos.goto.prod.to_token().to_string().to_token(),
          state_name_to_proxy,
        );

        insert_op(bc, Op::Goto);
        insert_u8(bc, NORMAL_STATE_FLAG as u8);
        insert_u32_le(bc, proxy_address);
      }
      matches => {
        build_match(db, matches, bc, state_name_to_proxy)?;
      }
    }
  }

  if let Some(last) = bc.last() {
    // Ensure the last op in this block is a sentinel type.
    match (*last).into() {
      Op::Pass
      | Op::Accept
      | Op::Fail
      | Op::Goto
      | Op::PeekSkipToken
      | Op::SkipToken
      | Op::SkipTokenScanless
      | Op::PeekSkipTokenScanless => {}
      _ => insert_op(bc, Op::Pass),
    }
  }

  SherpaResult::Ok(())
}

fn get_proxy_address(
  name: IString,
  state_name_to_proxy: &mut OrderedMap<IString, usize>,
) -> u32 {
  let val = state_name_to_proxy.len();
  let proxy_address = (*state_name_to_proxy.entry(name).or_insert(val)) as u32;
  proxy_address
}
fn build_match<'db>(
  db: &'db ParserDatabase,
  matches: &ASTNode,
  bc: &mut Array<u8>,
  state_name_to_proxy: &mut OrderedMap<IString, usize>,
) -> SherpaResult<()> {
  let mut default = None;
  let mut match_branches = Array::new();
  let mut input_type_key = InputType::Default as u32;
  let mut scanner_address = u32::MAX;

  let mut mode = match matches {
    ASTNode::Matches(box Matches { matches, mode, meta }) => {
      input_type_key = match mode.as_str() {
        InputType::PRODUCTION_STR => InputType::Production,
        InputType::TOKEN_STR => {
          scanner_address =
            get_proxy_address(IString::from_u64(*meta), state_name_to_proxy);
          InputType::Token
        }
        InputType::CLASS_STR => InputType::Class,
        InputType::CODEPOINT_STR => InputType::Codepoint,
        InputType::BYTE_STR => InputType::Byte,
        InputType::END_OF_FILE_STR => InputType::EndOfFile,
        _ => unreachable!(),
      } as u32;
      for m in matches.iter().rev() {
        match m {
          ASTNode::DefaultMatch(box DefaultMatch { statement, .. }) => {
            default = Some(statement.as_ref());
          }
          ASTNode::IntMatch(box IntMatch { statement, vals }) => {
            match_branches.push((vals, statement.as_ref()))
          }
          _ => {}
        }
      }
    }
    node => {
      #[cfg(debug_assertions)]
      dbg!(node);
      unreachable!();
    }
  };

  let mut offset = 0;
  let mut val_offset_map = Map::new();
  let mut sub_bcs = Array::new();

  for (ids, stmt) in match_branches {
    for id in ids {
      val_offset_map.insert(*id as u32, offset);
    }

    let mut sub_bc = Array::new();
    build_statement(db, stmt, &mut sub_bc, state_name_to_proxy)?;
    offset += sub_bc.len() as u32;
    sub_bcs.push(sub_bc);
  }

  let offset_lookup_table_length = val_offset_map.len() as u32;
  let instruction_field_start = 18 + offset_lookup_table_length * 4;
  let default_offset = offset + instruction_field_start;

  let mut pending_pairs = val_offset_map
    .clone()
    .into_iter()
    .map(|(k, v)| (k, v + instruction_field_start))
    .collect::<VecDeque<_>>();

  let mod_base = f64::log2(val_offset_map.len() as f64) as u32;
  let mod_mask = (1 << mod_base) - 1;

  let mut hash_entries =
    (0..pending_pairs.len()).into_iter().map(|_| 0).collect::<Vec<_>>();

  let mut leftover_pairs = vec![];

  // Distribute keys-values with unique hashes into hash table
  // slots.

  while let Some(pair) = pending_pairs.pop_front() {
    let (val, offset) = pair;
    let hash_index = (val & mod_mask) as usize;
    if hash_entries[hash_index] == 0 {
      hash_entries[hash_index] =
        (val & 0x7FF) | ((offset & 0x7FF) << 11) | (512 << 22);
    } else {
      leftover_pairs.push(pair);
    }
  }

  // What remains are hash collisions. We use simple linear
  // probing to find the next available slot, and
  // attach it to the probing chain using a signed
  // delta index.
  for (val, offset) in leftover_pairs {
    let mut pointer;
    let mut prev_node = (val & mod_mask) as usize;

    loop {
      pointer = (((hash_entries[prev_node] >> 22) & 0x3FF) as i32) - 512;

      if pointer == 0 {
        break;
      } else {
        prev_node = (prev_node as i32 + pointer as i32) as usize;
      }
    }

    for i in 0..hash_entries.len() {
      if hash_entries[i] == 0 {
        // Update the previous node in the chain with the
        // diff pointer to the new node.
        hash_entries[prev_node] =
          ((((i as i32 - prev_node as i32) + 512) as u32 & 0x3FF) << 22)
            | (hash_entries[prev_node] & ((1 << 22) - 1));
        // Add data for the new node.
        hash_entries[i] =
          ((val) & 0x7FF) | ((offset & 0x7FF) << 11) | (512 << 22);
        break;
      }
    }
  }

  insert_op(bc, Op::HashBranch); // 1
  insert_u8(bc, input_type_key as u8); // 2
  insert_u32_le(bc, default_offset); // 6
  insert_u32_le(bc, scanner_address); // 10
  insert_u32_le(bc, offset_lookup_table_length); // 14
  insert_u32_le(bc, mod_base); // 18

  for instruction in hash_entries {
    insert_u32_le(bc, instruction)
  }

  for mut sub_bc in sub_bcs {
    bc.append(&mut sub_bc)
  }

  if let Some(stmt) = default {
    build_statement(db, stmt, bc, state_name_to_proxy)?;
  } else {
    insert_op(bc, Op::Fail)
  }

  SherpaResult::Ok(())
}

const fn bytecode_header() -> [u8; 8] {
  [
    0,
    ('S' as u8) | 0x80,
    ('H' as u8) | 0x80,
    ('E' as u8) | 0x80,
    ('R' as u8) | 0x80,
    ('P' as u8) | 0x80,
    ('A' as u8) | 0x80,
    Op::Fail as u8,
  ]
}
