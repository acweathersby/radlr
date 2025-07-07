use super::{Range, TokenRange};
use std::{
  fmt::{self},
  hash::Hash,
  ops::Add,
  path::PathBuf,
  sync::Arc,
};

/// Stores color setting code for terminal text coloring of token blame strings.
#[derive(Clone, Copy, Debug)]
pub struct BlameColor {
  highlight: &'static str,
  reset:     &'static str,
}

impl BlameColor {
  /// Sets the color of the highlight to blue
  pub const BLUE: Option<BlameColor> = Some(BlameColor::new("\u{001b}[31m", "\u{001b}[0m"));
  /// Sets the color of the highlight to red
  pub const RED: Option<BlameColor> = Some(BlameColor::new("\u{001b}[31m", "\u{001b}[0m"));

  pub const fn new(highlight: &'static str, reset: &'static str) -> BlameColor {
    Self { highlight, reset }
  }
}

/// Combines a range and a reference to binary data.
#[derive(Clone)]
pub struct Token {
  pub(crate) inner: TokenRange,
  pub(crate) input: Option<Arc<[u8]>>,
}

impl Hash for Token {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.inner.line_num.hash(state);
    self.inner.line_off.hash(state);
    self.inner.off.hash(state);
    self.inner.len.hash(state);
    let _ = state.finish();
  }
}

impl PartialEq for Token {
  fn eq(&self, other: &Self) -> bool {
    matches!(self.inner.cmp(&other.inner), std::cmp::Ordering::Equal)
  }
}

impl PartialOrd for Token {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.inner.cmp(&other.inner))
  }
}

impl Ord for Token {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.inner.cmp(&other.inner)
  }
}

impl Eq for Token {
  fn assert_receiver_is_total_eq(&self) {}
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(&self.slice(0, self.inner.len as i32))
  }
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inner.len < 48 {
      f.write_fmt(format_args!(
        r#"Token{{ off:{}, len:{},  line:{}, l_off:{}, val: "{}" }}"#,
        self.inner.off,
        self.inner.len,
        self.inner.line_num,
        self.inner.line_off,
        self.to_string()
      ))
    } else {
      f.write_fmt(format_args!(
        "Token{{ off:{}, len:{}, line:{}, l_off:{} }}",
        self.inner.off, self.inner.len, self.inner.line_num, self.inner.line_off,
      ))
    }
  }
}

impl From<TokenRange> for Token {
  fn from(value: TokenRange) -> Self {
    Token { inner: value, ..Default::default() }
  }
}

impl Add for &Token {
  type Output = Token;

  fn add(self, rhs: Self) -> Self::Output {
    Token { inner: self.inner + rhs.inner, input: self.input.clone() }
  }
}

impl Add for Token {
  type Output = Token;

  fn add(self, rhs: Self) -> Self::Output {
    Token { inner: self.inner + rhs.inner, input: self.input.clone() }
  }
}

impl Add<Token> for u32 {
  type Output = Token;

  /// Shifts the offset of token right by `rhs`.
  ///
  /// Note: This may recalculate the line offset, which
  /// may be more expensive than one would expect.
  #[inline(always)]
  fn add(self, mut rhs: Token) -> Self::Output {
    rhs.inner = rhs.inner + self;
    rhs.recalculate_line_offset();
    Token { ..rhs }
  }
}

impl Add<u32> for Token {
  type Output = Token;

  /// Extends the size of token by `rhs`
  #[inline(always)]
  fn add(self, rhs: u32) -> Self::Output {
    Token { inner: self.inner + rhs, ..self }
  }
}
impl Add<u32> for &Token {
  type Output = Token;

  /// Extends the `len` of token by `rhs`
  #[inline(always)]
  fn add(self, rhs: u32) -> Self::Output {
    self.clone() + rhs
  }
}

impl Token {
  #[inline(always)]
  pub fn new() -> Token {
    Token { inner: TokenRange::default(), input: None }
  }

  #[inline(always)]
  pub fn from_vals(len: u32, off: u32, line_num: u32, line_off: u32) -> Token {
    Token { inner: TokenRange { len, off, line_num, line_off }, input: None }
  }

  #[inline(always)]
  pub fn from_range(start: &Token, end: &Token) -> Token {
    Token { inner: start.inner + end.inner, input: start.input.clone() }
  }

  #[inline(always)]
  pub const fn empty() -> Token {
    Token { inner: TokenRange::empty(), input: None }
  }

  /// Defines the source string for this token. Certain Token
  /// methods will not work correctly if the Token has not been
  /// attached to its source.
  #[inline(always)]
  pub fn set_source(&mut self, source: Arc<[u8]>) {
    self.input = Some(source);
  }

  pub fn trim(&self, trim_head: u32, trim_tail: u32) -> Token {
    Token {
      inner: self.inner.trim(trim_head, trim_tail),
      input: self.input.clone(),
    }
  }

  /// Returns the line number of the token location. This may be
  /// zero if the token is not attached to its source.
  pub fn get_line(&mut self) -> usize {
    let TokenRange { off, line_num, .. } = self.inner;
    if line_num == u32::MAX {
      if let Some(source) = self.input.clone() {
        let root = off as usize;
        let mut i = 0;
        let mut lines = 0;

        while i < root {
          if source[i] == 10 {
            lines += 1
          }

          i += 1
        }

        self.inner.line_num = lines;

        self.get_line()
      } else {
        0
      }
    } else {
      line_num as usize
    }
  }

  #[inline(always)]
  fn get_slice_range(&self, mut start: i32, mut end: i32) -> (usize, usize) {
    let TokenRange { len, off, .. } = self.inner;
    use std::cmp::{max, min};

    if start < 0 {
      start = max(off as i32, (off + len) as i32 + start)
    } else {
      start = min(off as i32 + start, (off + len) as i32)
    }

    if end < 0 {
      end = max(off as i32, (off + len) as i32 + end)
    } else {
      end = min(off as i32 + end, (off + len) as i32)
    }

    if end < start {
      end = off as i32;

      start = off as i32;
    }

    (start as usize, end as usize)
  }

  #[inline(always)]
  pub fn get_start(&self) -> usize {
    self.inner.off as usize
  }

  #[inline(always)]
  pub fn get_end(&self) -> usize {
    (self.inner.off + self.inner.len) as usize
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.inner.len as usize
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  #[inline(always)]
  pub fn to_length(&self, length: usize) -> Self {
    Self { inner: self.inner.to_length(length), input: self.input.clone() }
  }

  /// Returns a string containing the starting line and col
  /// token in the form `"{line num}:{col num}"`
  pub fn loc_stub(&self) -> String {
    let Range { start_line, start_column, .. } = self.get_range();
    format!("{}:{}", start_line, start_column)
  }

  /// Returns a string that appends the Token's `loc_stub` to
  /// the end of a path.
  ///
  /// # Example
  ///
  /// ```
  /// # use radlr_rust_runtime::types::Token;
  /// # use std::path::PathBuf;
  /// # use std::str::FromStr;
  /// let tok = Token::from_vals(5, 20, 9, 11);
  ///
  /// let path = PathBuf::from_str("/my/temp/file.txt").unwrap();
  ///
  /// assert_eq!(tok.path_ref(&path), "/my/temp/file.txt:10:9")
  /// ```
  #[inline(always)]
  pub fn path_ref(&self, path: &PathBuf) -> String {
    match path.to_str() {
      Some(string) => string.to_owned() + ":" + &self.loc_stub(),
      _ => Default::default(),
    }
  }

  #[inline(always)]
  pub fn get_range(&self) -> Range {
    self.inner.into()
  }

  #[inline(always)]
  pub fn get_tok_range(&self) -> TokenRange {
    self.inner
  }

  #[inline(always)]
  pub fn slice(&self, start: i32, end: i32) -> String {
    if let Some(input) = &self.input {
      let (adjusted_start, adjusted_end) = self.get_slice_range(start, end);

      unsafe { String::from_utf8_unchecked(Vec::from(&input[adjusted_start..adjusted_end])) }
    } else {
      String::default()
    }
  }

  #[inline(always)]
  pub fn to_f32(&self) -> f32 {
    self.to_numeric_or_length() as f32
  }

  #[inline(always)]
  pub fn to_f64(&self) -> f64 {
    self.to_numeric_or_length() as f64
  }

  #[inline(always)]
  pub fn to_i8(&self) -> i8 {
    self.to_numeric_or_length() as i8
  }

  #[inline(always)]
  pub fn to_i16(&self) -> i16 {
    self.to_numeric_or_length() as i16
  }

  #[inline(always)]
  pub fn to_i32(&self) -> i32 {
    self.to_numeric_or_length() as i32
  }

  #[inline(always)]
  pub fn to_i64(&self) -> i64 {
    self.to_numeric_or_length() as i64
  }

  #[inline(always)]
  pub fn to_u8(&self) -> u8 {
    self.to_numeric_or_length() as u8
  }

  #[inline(always)]
  pub fn to_u16(&self) -> u16 {
    self.to_numeric_or_length() as u16
  }

  #[inline(always)]
  pub fn to_u32(&self) -> u32 {
    self.to_numeric_or_length() as u32
  }

  #[inline(always)]
  pub fn to_u64(&self) -> u64 {
    self.to_numeric_or_length() as u64
  }

  #[inline(always)]
  pub fn to_numeric_or_length(&self) -> f64 {
    match self.to_string().parse::<f64>() {
      Ok(num) => num,
      Err(_) => self.inner.len as f64,
    }
  }

  #[inline(always)]
  fn find_next_line(source: &[u8], line: i64) -> i64 {
    let mut i = line;
    let len = source.len() as i64;
    while i < len && (i < 0 || source[i as usize] as char != '\n' || i == line) {
      i += 1;
    }
    i as i64
  }

  #[inline(always)]
  fn find_prev_line(source: &[u8], mut line: i64) -> i64 {
    line -= 1;
    while line >= 0 && source[line as usize] as char != '\n' {
      line -= 1;
    }

    line
  }

  #[inline(always)]
  fn recalculate_line_offset(&mut self) {
    if let Some(source) = self.input.clone() {
      let prev_line = Self::find_prev_line(&source, (self.inner.off + 1) as i64);
      self.inner.line_off = prev_line.max(0).min(u32::MAX as i64) as u32;
    }
  }

  /// Creates a diagram of the position of this token within the
  /// source string.
  ///
  /// ### Arguments:
  /// - `max_pre` - The maximum number of lines to render before the token
  ///   line(s).
  /// - `max_post` - The maximum number of lines to render after the token
  ///   line(s).
  ///
  /// ### Returns:
  /// - `Option<String>` - A `String` of the blame diagram or `None` if source
  ///   is
  /// not defined.
  pub fn blame(&self, max_pre: usize, max_post: usize, inline_comment: &str, colors: Option<BlameColor>) -> String {
    fn create_line(source: &Arc<[u8]>, prev_line: i64, next_line: i64, line_number: usize) -> String {
      if let Ok(utf_string) = String::from_utf8(Vec::from(&source[(prev_line + 1) as usize..next_line as usize])) {
        format!("{: >4}: {}\n", line_number, utf_string,)
      } else {
        String::from("")
      }
    }

    if let Some(source) = self.input.clone() {
      if source.len() == 0 {
        return format!("[Source Invalid] \n{:^>16} {inline_comment}", "");
      }

      let mut string = String::from("");
      let mut prev_line = Self::find_prev_line(&source, self.inner.off as i64) as i64;
      let mut line_num = (self.inner.line_num + 1) as usize;
      let mut next_line;
      let mut col_diff = (self.inner.off as i64 - prev_line - (prev_line != 0) as i64).max(0) as usize;

      if source[0] as char == '\n' {
        line_num -= 1;
      } else if prev_line == 0 {
        prev_line -= 1;
      }

      if prev_line > 0 && line_num > 0 {
        let mut prev_line = prev_line;
        for a in 1..=max_pre {
          if prev_line <= 0 {
            break;
          }
          let next_line = prev_line;
          prev_line = Self::find_prev_line(&source, prev_line);
          string = create_line(&source, prev_line, next_line, line_num - a) + &string;
        }
      }

      loop {
        let max_size = source.len() as i64;
        next_line = Self::find_next_line(&source, prev_line);

        if prev_line >= max_size {
          break;
        }

        if let Ok(utf_string) =
          String::from_utf8(Vec::from(&source[(prev_line + 1).min(max_size) as usize..next_line.min(max_size) as usize]))
        {
          let leading_spaces = utf_string.len() - utf_string.trim_start().len();
          let diff = usize::max(leading_spaces, col_diff);
          let highlight_len = ((utf_string.len() as i64)
            - (diff as i64)
            - i64::max(0, next_line as i64 - (self.inner.off + self.inner.len) as i64))
          .max(1) as usize;

          let lines_str = format!("{: >4}", line_num);
          string += &format!(
            "{}: {}\n{}",
            &lines_str,
            utf_string,
            String::from(" ").repeat(4 + 1 + diff)
              + &if let Some(BlameColor { highlight, .. }) = colors {
                " ".to_string() + highlight + &String::from("^").repeat(highlight_len as usize) + " "
              } else {
                " ".to_string() + &String::from("^").repeat(highlight_len as usize) + " "
              },
          );

          line_num += 1;
          prev_line = next_line;
          col_diff = 0;

          match (next_line >= (self.inner.off + self.inner.len) as i64, colors) {
            (true, Some(BlameColor { reset, .. })) => {
              string += &format!("{}{}\n", inline_comment, reset);
              break;
            }
            (true, None) => {
              string += &(inline_comment.to_owned() + "\n");
              break;
            }
            (false, Some(BlameColor { reset, .. })) => {
              string += &(reset.to_owned() + "\n");
            }
            (false, None) => {
              string += "\n";
            }
          }
        } else {
          break;
        }
      }

      for a in 0..max_post {
        if next_line as usize >= source.len() {
          break;
        }
        let prev_line = next_line;
        next_line = Self::find_next_line(&source, next_line);
        string += &create_line(&source, prev_line as i64, next_line, line_num + a);
      }

      string
    } else {
      "[Token Source Not Valid]\n   ".to_string() + inline_comment
    }
  }
}

impl Default for Token {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod test {
  use crate::types::{Token, TokenRange};

  #[test]
  pub fn blame_string_places_cursor_in_correct_position() {
    let tok = Token {
      input: Some("\n start \n\n test \n final ".to_string().as_bytes().into()),
      inner: TokenRange { len: 4, off: 11, line_num: 3, line_off: 9 },
    };

    let blame_string = tok.blame(2, 1, "test", None);
    let lines = blame_string.split("\n").collect::<Vec<_>>();

    eprintln!("{}", blame_string);

    assert_eq!(lines.len(), 6);

    assert_eq!(lines[0], "   1:  start ");
    assert_eq!(lines[1], "   2: ");
    assert_eq!(lines[2], "   3:  test ");
    assert_eq!(lines[3], "       ^^^^ test");
    assert_eq!(lines[4], "   4:  final ");
  }

  #[test]
  pub fn blame_string_places_cursor_in_correct_position2() {
    let tok = Token {
      input: Some(" start \n\n test \n final ".to_string().as_bytes().into()),
      inner: TokenRange { len: 5, off: 1, line_num: 0, line_off: 0 },
    };

    let blame_string = tok.blame(2, 3, "start", None);
    let lines = blame_string.split("\n").collect::<Vec<_>>();

    eprintln!("{}", blame_string);

    assert_eq!(lines.len(), 6);

    assert_eq!(lines[0], "   1:  start ");
    assert_eq!(lines[1], "       ^^^^^ start");
    assert_eq!(lines[2], "   2: ");
    assert_eq!(lines[3], "   3:  test ");
    assert_eq!(lines[4], "   4:  final ");
  }
}
