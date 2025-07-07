use std::{ops::Add, str::FromStr};

use super::{ParserInput, SharedSymbolBuffer, Token};

#[derive(Clone, Debug)]
pub struct Range {
  /// The line number at which the range starts
  pub start_line:   u32,
  /// The line number at which the range ends
  pub end_line:     u32,
  /// The column of the first character in the range
  pub start_column: u32,
  /// The column following the last character in the range
  pub end_column:   u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct TokenRange {
  pub len:      u32,
  pub off:      u32,
  pub line_num: u32,
  pub line_off: u32,
}

#[test]
fn token_range_size_is_16() {
  assert_eq!(std::mem::size_of::<TokenRange>(), 16)
}

impl PartialOrd for TokenRange {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TokenRange {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.off == other.off {
      true => match self.len == other.len {
        true => std::cmp::Ordering::Equal,
        false => match self.len < other.len {
          true => std::cmp::Ordering::Less,
          false => std::cmp::Ordering::Greater,
        },
      },
      false => match self.off < other.off {
        true => std::cmp::Ordering::Less,
        false => std::cmp::Ordering::Greater,
      },
    }
  }
}

impl Add for TokenRange {
  type Output = TokenRange;

  fn add(self, rhs: Self) -> Self::Output {
    let (min, max) = if self < rhs { (self, rhs) } else { (rhs, self) };
    TokenRange {
      len:      max.off - min.off + max.len,
      off:      min.off,
      line_num: min.line_num,
      line_off: min.line_off,
    }
  }
}

impl Into<Range> for TokenRange {
  fn into(self) -> Range {
    self.get_range()
  }
}

impl Add<TokenRange> for u32 {
  type Output = TokenRange;

  /// Shifts the offset of token right by `rhs`.
  ///
  /// Note: This may recalculate the line offset, which
  /// may be more expensive than one would expect.
  #[inline(always)]
  fn add(self, mut rhs: TokenRange) -> Self::Output {
    rhs.off += self;
    TokenRange { ..rhs }
  }
}

impl Add<u32> for TokenRange {
  type Output = TokenRange;

  /// Extends the size of token by `rhs`
  fn add(self, rhs: u32) -> Self::Output {
    TokenRange { len: self.len + rhs, ..self }
  }
}

impl TokenRange {
  pub fn from_range(start: TokenRange, end: TokenRange) -> Self {
    TokenRange {
      len:      end.off - start.off + end.len,
      off:      start.off,
      line_num: start.line_num,
      line_off: start.line_off,
    }
  }

  #[inline(always)]
  pub const fn empty() -> Self {
    TokenRange { len: 0, off: 0, line_num: u32::MAX - 1, line_off: 0 }
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.len as usize
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  #[inline(always)]
  pub fn to_length(&self, length: usize) -> Self {
    Self {
      len:      length as u32,
      off:      self.off,
      line_num: self.line_num,
      line_off: self.line_off,
    }
  }

  pub fn trim(&self, start_trim: u32, end_trim: u32) -> Self {
    let new_len = (self.len as i32 - start_trim as i32 - end_trim as i32).max(0);
    let new_off = (self.off + start_trim).min(self.off + self.len);

    Self {
      len:      new_len as u32,
      off:      new_off,
      line_num: self.line_num,
      line_off: self.line_off,
    }
  }

  #[inline(always)]
  pub fn get_range(&self) -> Range {
    Range {
      end_column:   (self.off - self.line_off).max(1) + self.len,
      start_column: (self.off - self.line_off).max(1),
      end_line:     self.line_num + 1,
      start_line:   self.line_num + 1,
    }
  }

  pub fn to_slice<'a>(&self, source: &'a str) -> &'a str {
    if (self.off + self.len) as usize > source.len() {
      &source[0..0]
    } else {
      &source[self.off as usize..(self.off + self.len) as usize]
    }
  }

  pub fn parse<T: FromStr + Default>(&self, source: &str) -> T {
    self.to_slice(source).parse::<T>().unwrap_or_default()
  }

  #[allow(deprecated)]
  #[deprecated]
  pub fn to_token(&self, reader: &mut dyn crate::deprecate::ByteReader) -> Token {
    let mut tok: Token = (*self).into();
    tok.set_source(reader.get_source());
    tok
  }

  pub fn to_string_from_input<T: ParserInput>(&self, input: &T) -> String {
    if (self.off + self.len) as usize > input.len() {
      Default::default()
    } else {
      input.string_range(self.off as usize..(self.off + self.len) as usize)
    }
  }

  pub fn to_token_from_ref(&self, source: SharedSymbolBuffer) -> Token {
    let mut tok: Token = (*self).into();
    tok.set_source(source);
    tok
  }

  pub fn to_token_with_string(&self, original_string: &str) -> Token {
    let mut tok: Token = (*self).into();
    tok.set_source(original_string.to_string().as_bytes().into());
    tok
  }

  /// The start byte offset of this token range. Equivalent
  /// to `self.off`. Positioned before the
  /// first character in the range.
  pub fn start_offset(&self) -> usize {
    self.off as usize
  }

  /// The end byte offset of this token range. Equivalent
  /// to `self.off + self.len`. Positioned after the
  /// last character in the range.
  pub fn end_offset(&self) -> usize {
    (self.off + self.len) as usize
  }
}
