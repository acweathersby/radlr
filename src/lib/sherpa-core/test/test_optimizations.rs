use super::utils::build_parse_states_from_multi_sources;
use crate::{
  optimize,
  test::utils::{build_parse_states_from_source_str as build, TestPackage as Tp},
  DBProdKey,
  ParseStatesVec,
  SherpaResult as R,
};

#[test]
fn basic_optimize_unknown() -> R<()> {
  build_parse_states_from_multi_sources(
    &[r##"
    IGNORE { c:sp c:nl }
    
    <> json > '{'  value(*',') '}'
    
    <> value > tk:string ':' tk:string
    
        | "\"test\"" ':' c:num
    
    <> string > '"' ( c:sym | c:num | c:sp | c:id | escape )(*) '\"'
        
    <> escape > "\\"   ( c:sym | c:num | c:sp | c:id | c:nl)
    "##],
    "/".into(),
    Default::default(),
    &|Tp { states, db, .. }| {
      //for state in &states {
      //  println!("A: store{:#}\n", state.1.source_string(db.string_store()))
      // }

      let states = optimize::<ParseStatesVec>(db, states, false)?;

      println!("AFTER -------------------");

      for state in states {
        println!("B: {} {:#}\n", state.1.get_canonical_hash(db), state.1.print(db, true)?)
      }

      R::Ok(())
    },
  )
}
