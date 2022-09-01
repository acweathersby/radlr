use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;

use crate::types::ParseToken;
use crate::utf8::get_utf8_code_point_from;

use super::reader::ImmutCharacterReader;
use super::reader::SharedSymbolBuffer;
use super::ByteCharacterReader;
use super::LLVMCharacterReader;
use super::MutCharacterReader;
use super::UTF8CharacterReader;

#[derive(Debug, Clone)]
pub struct UTF8StringReader<'a> {
  len:        usize,
  cursor:     usize,
  line_count: usize,
  line_off:   usize,
  string:     &'a [u8],
  word:       u32,
  cp:         u32,
}

impl<'a> LLVMCharacterReader for UTF8StringReader<'a> {}

impl<'a> ByteCharacterReader for UTF8StringReader<'a> {
  fn get_bytes(&self) -> &[u8] {
    self.string
  }
}

impl<'a> UTF8CharacterReader for UTF8StringReader<'a> {}

impl<'a> MutCharacterReader for UTF8StringReader<'a> {
  fn next(&mut self, amount: i32) -> u64 {
    Self::next_utf8(self, amount)
  }

  fn set_cursor(&mut self, cursor: usize) {
    self.cursor = cursor;
  }

  fn set_codepoint(&mut self, code_point: u32) {
    self.cp = code_point;
  }

  fn set_dword(&mut self, dword: u32) {
    self.word = dword;
  }

  fn set_line_count(&mut self, line_count: u32) {
    self.line_count = line_count as usize;
  }

  fn set_line_offset(&mut self, line_offset: u32) {
    self.line_off = line_offset as usize;
  }
}

impl<'a> ImmutCharacterReader for UTF8StringReader<'a> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.len
  }

  #[inline(always)]
  fn byte(&self) -> u32 {
    if (self.cursor() >= self.len) {
      0
    } else {
      self.get_bytes()[self.cursor()] as u32
    }
  }

  #[inline(always)]
  fn dword(&self) -> u32 {
    self.word
  }

  #[inline(always)]
  fn line_offset(&self) -> u32 {
    self.line_off as u32
  }

  #[inline(always)]
  fn line_count(&self) -> u32 {
    self.line_count as u32
  }

  #[inline(always)]
  fn codepoint(&self) -> u32 {
    self.cp
  }

  #[inline(always)]
  fn cursor(&self) -> usize {
    self.cursor
  }

  #[inline(always)]
  fn get_source(&self) -> SharedSymbolBuffer {
    let vec = self.string.clone();

    SharedSymbolBuffer::new(Vec::from(vec))
  }

  #[inline(always)]
  fn get_line_data(&self) -> u64 {
    ((self.line_count as u64) << 32) | self.line_off as u64
  }

  #[inline(always)]
  fn get_length_data(&self) -> u64 {
    ((self.codepoint_byte_length() as u64) << 32) | self.codepoint_length() as u64
  }

  #[inline(always)]
  fn set_cursor_to(&mut self, token: &ParseToken) -> u64 {
    let ParseToken { byte_offset, line_number, line_offset, .. } = *token;

    if self.cursor != byte_offset as usize {
      let diff = byte_offset as i32 - self.cursor as i32;

      self.line_count = line_number as usize;

      self.line_off = line_offset as usize;

      self.next(diff)
    } else {
      self.get_type_info()
    }
  }
}

impl<'a> UTF8StringReader<'a> {
  pub fn from_string(string: &'a str) -> Self {
    Self::new(string)
  }

  pub fn new(string: &'a str) -> UTF8StringReader<'a> {
    let mut reader = UTF8StringReader {
      string:     string.as_bytes(),
      len:        string.len(),
      cursor:     0,
      word:       0,
      line_count: 0,
      line_off:   0,
      cp:         0,
    };

    Self::next(&mut reader, 0);

    reader
  }
}
