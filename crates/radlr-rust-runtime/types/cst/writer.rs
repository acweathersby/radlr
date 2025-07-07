use super::super::*;
use std::io::Write;

pub struct Printer<'node, 'db> {
  node:          &'node CSTNode,
  db:            &'db dyn RuntimeDatabase,
  write_missing: bool,
}

impl<'node, 'db> Printer<'node, 'db> {
  pub fn new<'a, 'b>(node: &'a CSTNode, write_missing: bool, db: &'b dyn RuntimeDatabase) -> Printer<'a, 'b> {
    Printer { node, db, write_missing }
  }

  pub fn new_node(&self, node: &'node CSTNode) -> Self {
    Self { node, db: self.db, write_missing: self.write_missing }
  }

  pub fn to_string(&self) -> String {
    let mut vec = vec![];
    self.write(&mut vec).unwrap();
    unsafe { String::from_utf8_unchecked(vec) }
  }

  pub fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
    use CSTNode::*;
    match self.node {
      Token(token) => match token.ty() {
        NodeType::Missing => {
          if self.write_missing {
            let id = token.tok_id() as u32;
            if let Some(token_str) = self.db.token_id_to_str(id) {
              w.write(token_str.as_bytes())?;
            } else {
              w.write(format!("[TK:{id}]").as_bytes())?;
            }
          }
        }
        NodeType::Errata => {}
        _ => {
          w.write(token.str().as_bytes())?;
        }
      },
      NonTerm(non_term) => {
        for node in &non_term.symbols {
          self.new_node(node).write(w)?;
        }
      }
      PlaceholderNonTerm(non_term) => {
        if let Some(name) = self.db.get_nonterminal_name_from_id(*non_term) {
          w.write(format!("<NT:{name}>").as_bytes())?;
        } else {
          w.write(format!("<NT:{non_term}>").as_bytes())?;
        }
      }
      Alts(multi) => {
        if let Some(first) = multi.alternatives.first() {
          for node in &first.symbols {
            self.new_node(node).write(w)?;
          }
        }
      }
    }
    Ok(())
  }

  pub fn print(&self) {
    let mut stdout = std::io::stderr();
    self.write(&mut stdout).unwrap();
    stdout.flush().unwrap();
  }

  pub fn print_all(&self) {
    let mut stdout = std::io::stdout();
    self.write_all(&mut stdout, true).unwrap();
  }

  fn write_all<W: Write>(&self, w: &mut W, root: bool) -> std::io::Result<()> {
    use CSTNode::*;

    match self.node {
      Token(_) => {
        self.new_node(self.node).write(w)?;
      }

      NonTerm(non_term) => {
        for node in &non_term.symbols {
          if root {
            w.write(b"\n\n")?;
          }
          self.new_node(node).write_all(w, false)?;
        }
      }
      PlaceholderNonTerm(non_term) => {
        w.write(format!("placeholder [{non_term}]").as_str().as_bytes())?;
      }
      Alts(multi) => {
        for alt in &multi.alternatives {
          for node in &alt.symbols {
            self.new_node(node).write_all(w, root)?;
          }
        }
      }
    }
    Ok(())
  }
}
