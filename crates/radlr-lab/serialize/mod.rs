// Export and import functions
pub mod bytecode_db {

  use std::{collections::HashMap, hash::Hash};

  use radlr_rust_runtime::types::{BytecodeParserDB, EntryPoint, ParserError};

  /// Import a database from its portable binary format
  pub fn import_bytecode_db(buffer: &[u8]) -> Result<BytecodeParserDB, ParserError> {
    let mut db = BytecodeParserDB::default();
    let mut offset = 0;

    let bc_len = read_primitive_at_offset::<u32>(&buffer, &mut offset) as usize;
    db.bytecode = buffer[offset..offset + bc_len].to_vec();
    offset += bc_len;

    db.address_to_state_name = read_hash_id_str(&buffer, &mut offset);
    db.token_id_to_str = read_hash_id_str(&buffer, &mut offset);
    db.state_name_to_address = read_hash_of_str_id(&buffer, &mut offset);
    db.nonterm_name_to_id = read_hash_of_str_id(&buffer, &mut offset);
    db.state_to_token_ids_map = read_hash_of_id_vecu32(&buffer, &mut offset);
    db.nonterm_id_to_address = read_primitive_hash(&buffer, &mut offset);

    db.nonterm_id_to_name = read_hash_id_str(&buffer, &mut offset);
    db.rule_offsets = read_primitive_hash(&buffer, &mut offset);
    db.rule_diagram = read_hash_id_str(&buffer, &mut offset);

    db.default_entry = read_primitive_at_offset(&buffer, &mut offset);

    Ok(db)
  }

  /// Export the database into a portable binary format
  pub fn export_bytecode_db(db: &BytecodeParserDB) -> Vec<u8> {
    let mut size = db.bytecode.len() + 4;

    // address_to_state_name:     HashMap<u32, String>
    size += 4 + db.address_to_state_name.iter().fold(0, |size, d| size + 8 + d.1.as_bytes().len());

    // token_id_to_str:           HashMap<u32, String>
    size += 4 + db.token_id_to_str.iter().fold(0, |size, d| size + 8 + d.1.as_bytes().len());

    // state_name_to_address:       HashMap<String, u32>
    size += 4 + db.state_name_to_address.iter().fold(0, |size, d| size + 8 + d.0.as_bytes().len());

    // nonterm_name_to_id:        HashMap<String, u32>
    size += 4 + db.nonterm_name_to_id.iter().fold(0, |size, d| size + 8 + d.0.as_bytes().len());

    // state_to_token_ids_map:    HashMap<u32, Vec<u32>>
    size += 4 + db.state_to_token_ids_map.iter().fold(0, |size, d| size + 8 + d.1.len() * 4);

    // rule_diagram:    HashMap<u32, String>
    size += 4 + db.rule_diagram.iter().fold(0, |size, d| size + 8 + d.1.as_bytes().len());

    // nonterm_name:    HashMap<u32, String>
    size += 4 + db.nonterm_id_to_name.iter().fold(0, |size, d| size + 8 + d.1.as_bytes().len());

    // nonterm_offsets:    HashMap<u32, (u32, u32)>
    size += 4 + db.rule_offsets.iter().fold(0, |size, _| size + 4 + size_of::<(u32, u32)>());

    //// ir_token_lookup:           BTreeMap<u32, Token>
    //size += 4 + db.ir_token_lookup.iter().fold(0, |size, _| size + 4 +
    // size_of::<Token>());

    // nonterm_id_to_address:     HashMap<u32, u32>
    size += 4 + db.nonterm_id_to_address.len() * 8;

    // default_entry
    size += size_of::<EntryPoint>();

    let mut buffer = Vec::<u8>::with_capacity(size);

    write_primitive_to_bytes(&mut buffer, db.bytecode.len() as u32);
    write_bytes(&mut buffer, &db.bytecode);
    write_hash_of_id_str(&mut buffer, &db.address_to_state_name);
    write_hash_of_id_str(&mut buffer, &db.token_id_to_str);
    write_hash_of_str_id(&mut buffer, &db.state_name_to_address);
    write_hash_of_str_id(&mut buffer, &db.nonterm_name_to_id);
    write_hash_of_id_vecu32(&mut buffer, &db.state_to_token_ids_map);
    write_primitive_hash(&mut buffer, &db.nonterm_id_to_address);

    write_hash_of_id_str(&mut buffer, &db.nonterm_id_to_name);
    write_primitive_hash(&mut buffer, &db.rule_offsets);
    write_hash_of_id_str(&mut buffer, &db.rule_diagram);

    write_primitive_to_bytes(&mut buffer, db.default_entry);
    debug_assert_eq!(size, buffer.len(), "Error calculating buffer length");

    buffer
  }

  fn write_hash_of_id_str<T: Clone + Copy>(buffer: &mut Vec<u8>, data: &HashMap<T, String>) {
    write_primitive_to_bytes(buffer, data.len() as u32);
    for (id, str) in data {
      write_primitive_to_bytes(buffer, *id);
      write_primitive_to_bytes(buffer, str.len() as u32);
      write_bytes(buffer, str.as_bytes());
    }
  }

  fn read_hash_id_str<T: Copy + Clone + Default + Eq + Hash>(buffer: &[u8], offset: &mut usize) -> HashMap<T, String> {
    let entry_count = read_primitive_at_offset::<u32>(buffer, offset) as usize;
    let mut hash = HashMap::with_capacity(entry_count);
    for _ in 0..entry_count {
      let k = read_primitive_at_offset::<T>(buffer, offset);
      let str_len = read_primitive_at_offset::<u32>(buffer, offset) as usize;
      let v = unsafe { String::from_utf8_unchecked(buffer[*offset..*offset + str_len].to_vec()) };
      *offset += str_len;
      hash.insert(k, v);
    }
    hash
  }

  fn write_hash_of_str_id<T: Clone + Copy>(buffer: &mut Vec<u8>, data: &HashMap<String, T>) {
    write_primitive_to_bytes(buffer, data.len() as u32);
    for (str, id) in data {
      write_primitive_to_bytes(buffer, *id);
      write_primitive_to_bytes(buffer, str.len() as u32);
      write_bytes(buffer, str.as_bytes());
    }
  }

  fn read_hash_of_str_id<T: Copy + Clone + Default + Eq + Hash>(buffer: &[u8], offset: &mut usize) -> HashMap<String, T> {
    let entry_count = read_primitive_at_offset::<u32>(buffer, offset) as usize;
    let mut hash = HashMap::with_capacity(entry_count);
    for _ in 0..entry_count {
      let v = read_primitive_at_offset::<T>(buffer, offset);
      let str_len = read_primitive_at_offset::<u32>(buffer, offset) as usize;
      let k = unsafe { String::from_utf8_unchecked(buffer[*offset..*offset + str_len].to_vec()) };
      *offset += str_len;
      hash.insert(k, v);
    }
    hash
  }

  fn read_primitive_at_offset<T: Copy + Default>(buffer: &[u8], offset: &mut usize) -> T {
    unsafe {
      let size: usize = size_of::<T>();
      let data: T = Default::default();
      let bytes: *mut u8 = std::mem::transmute(&data);
      buffer.as_ptr().offset(*offset as isize).copy_to(bytes, size);
      *offset += size;
      data
    }
  }

  fn write_primitive_hash<K: Clone + Copy, V: Clone + Copy>(buffer: &mut Vec<u8>, data: &HashMap<K, V>) {
    write_primitive_to_bytes(buffer, data.len() as u32);
    for (k, v) in data {
      write_primitive_to_bytes(buffer, *k);
      write_primitive_to_bytes(buffer, *v);
    }
  }

  fn read_primitive_hash<K: Clone + Copy + Eq + Hash + Default, V: Clone + Copy + Default>(
    buffer: &[u8],
    offset: &mut usize,
  ) -> HashMap<K, V> {
    let entry_count = read_primitive_at_offset::<u32>(buffer, offset) as usize;
    let mut hash = HashMap::with_capacity(entry_count);
    for _ in 0..entry_count {
      let k = read_primitive_at_offset::<K>(buffer, offset);
      let v = read_primitive_at_offset::<V>(buffer, offset);
      hash.insert(k, v);
    }
    hash
  }

  fn write_hash_of_id_vecu32<T: Clone + Copy>(buffer: &mut Vec<u8>, data: &HashMap<T, Vec<u32>>) {
    write_primitive_to_bytes(buffer, data.len() as u32);
    for (k, v) in data {
      write_primitive_to_bytes(buffer, *k);
      write_primitive_to_bytes(buffer, v.len() as u32);
      write_bytes(buffer, v.as_slice());
    }
  }

  fn read_hash_of_id_vecu32<T: Copy + Clone + Default + Eq + Hash>(buffer: &[u8], offset: &mut usize) -> HashMap<T, Vec<u32>> {
    let entry_count = read_primitive_at_offset::<u32>(buffer, offset) as usize;
    let mut hash = HashMap::with_capacity(entry_count);
    for _ in 0..entry_count {
      let k = read_primitive_at_offset::<T>(buffer, offset);
      let vec_size = read_primitive_at_offset::<u32>(buffer, offset) as usize;
      let byte_len = vec_size * size_of::<u32>();

      let mut v = Vec::<u32>::with_capacity(vec_size);
      unsafe {
        v.set_len(vec_size);
        buffer.as_ptr().offset(*offset as isize).copy_to(std::mem::transmute(v.as_mut_ptr()), byte_len);
      }

      *offset += byte_len;

      hash.insert(k, v);
    }
    hash
  }

  fn write_bytes<T: Copy + Clone>(buffer: &mut Vec<u8>, data: &[T]) {
    unsafe {
      let size: usize = size_of::<T>();
      let off: usize = buffer.len();
      let byte_size = data.len() * size;
      buffer.set_len(off + byte_size);
      let ptr: *const u8 = std::mem::transmute(data.as_ptr());
      ptr.copy_to(buffer.as_mut_ptr().offset(off as isize), byte_size);
    }
  }

  fn write_primitive_to_bytes<T: Copy>(buffer: &mut Vec<u8>, data: T) {
    unsafe {
      let size: usize = size_of::<T>();
      let off: usize = buffer.len();
      let bytes: *const u8 = std::mem::transmute(&data);
      buffer.set_len(off + size);
      bytes.copy_to(buffer.as_mut_ptr().offset(off as isize), size);
    }
  }
}
