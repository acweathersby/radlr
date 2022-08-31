use crate::types::GrammarStore;
use crate::types::Item;

pub fn debug_items(comment: &str, items: &[Item], grammar: &GrammarStore)
{
  eprintln!("{} --> ", comment);

  for item in items {
    println!("    {}", item.debug_string(grammar));
  }
}
