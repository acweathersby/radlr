use std::path::PathBuf;

use crate::{types::AscriptDatabase, AscriptAggregateType, AscriptMultis, AscriptRule, AscriptScalarType, AscriptType, StringId};
use radlr_core::{CachedString, RadlrGrammar, RadlrResult};

#[test]
fn parse_errors_when_struct_prop_type_is_redefined() -> RadlrResult<()> {
  let source = r##"

  <> a > "a" :ast { t_TestA, apple: u32 } |  "b"  :ast { t_TestA, apple: i64 }

          "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  assert!(adb.errors.len() > 0);

  println!("{}", adb.errors[0]);

  Ok(())
}

#[test]
fn group_rules_as_vectors() -> RadlrResult<()> {
  let source = r#"<> A > ( "1" :ast u32($1) )(+"|") "#;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  let AscriptRule::ListInitial(_, init) = &adb.rules[2] else {
    panic!("Expected AscriptRule::ListInitial - got {:?}", &adb.rules[2])
  };

  assert!(matches!(init.ty, AscriptType::Aggregate(AscriptAggregateType::Vec { .. })));

  Ok(())
}

#[test]
fn prop_is_made_optional_when_not_present_or_introduced_in_subsequent_definitions() -> RadlrResult<()> {
  let source = r#" <> start > A | B

  <> A > "1234" :ast { t_R, d:str($1) }

  <> B > "1234" :ast { t_R, o: u32 }"#;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  let entry = adb.structs.0.first_key_value().unwrap().1;

  assert!(entry.properties.get(&StringId::from("d")).unwrap().is_optional);
  assert!(entry.properties.get(&StringId::from("o")).unwrap().is_optional);

  Ok(())
}

#[test]
fn handles_numeric_expressions() -> RadlrResult<()> {
  let source = r#" IGNORE { c:sp c:nl }

  <> statement > e            :ast { t_Result, v:$1 }
  
  <> e > e "+"{1} e{2}        :ast $1 + $3  
       | e "*"{3} e{4}        :ast $1 * $3 
       | term                   
  
  <> term > tk:num            :ast f64($1)  
          | "(" e ")"         :ast $2      
          
  <> num > c:num(+)"#;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_json() -> RadlrResult<()> {
  let source = r#"IGNORE {c:sp c:nl}

  <> json 
  
    > entry                 :ast { t_JSON, body: $1, tok }
  
  <> entry 
  
    > obj | array
    
  <> obj 
    
    > "{" key_val(+",") "}" :ast { t_Object, values: $2, tok }
  
  <> array
    
    > "[" val(+",") "]"     :ast { t_Array, values: $2, tok }
  
  
  <> key_val 
  
    > key ":" val           :ast map($1, $3)
  
  
  <> key 
  
    > tk:string             :ast str(tok<1,1>)
  
  
  <> val 
    > tk:string :ast str(tok<1,1>)
    | c:num     :ast f64($1)
    | obj
    | array
    | "true"    :ast bool($1)
    | "false"   :ast bool
    | "null"    :ast {t_Null}
  
  
  <> string 
    > "\""  ( c:id | c:sp |  c:num | c:sym )(*)  "\""
   "#;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_rum_lang_2() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > ( [ ( R | B )(*)^s T?^eoi ]  :ast [$s, $eoi] )^s 

  <> T > "tt" :ast { t_T }

  <> R > "test" :ast { t_R }

  <> B >  "nest" :ast { t_B }
   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_rum_langd() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > B Ca? :ast { t_A, B, Ca }

  <> B > "test"

  <> Ca > "test" :ast f32($1)

   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_template() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> B > A::<t_Test>^A  :ast { t_R, A }
  
  <t_TypeNamePrefix:ast> A > "Cat" :ast { t_A, B, Ca }

   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_values() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > B | D

  <> B > X :ast { t_B, X }

  <> D > Y :ast { t_D, X }


  <> X > "A" :ast { t_X } 
       | "B" :ast { t_Y } 

  
  <> Y > "C" :ast { t_X } 
       | "D" :ast { t_Y } 
  

   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(&adb);

  for i in 0..adb.multi_types.len() {
    for j in 0..adb.multi_types.len() {
      if j == i {
        continue;
      }

      assert_ne!(adb.multi_type_lu[i], adb.multi_type_lu[j]);
    }
  }

  Ok(())
}

#[test]
fn builds_values_2() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > C | B 

  <> C > B | "R" :ast  { t_R }

  <> B > "X" :ast {t_X} | "Y" :ast {t_Y} | "Z" :ast {t_Z} | W

  <> W > "W" :ast { t_W }
  

   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(&adb.multi_types);

  Ok(())
}

#[test]
fn aliased_struct_properties_are_tracked() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> A > "begin" D?^test_prop

        :ast { t_TestStruct, test_prop, other_prop: $2 }

  <> D > "test"
"##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let mut adb: AscriptDatabase = db.into();

  assert_eq!(adb.structs.len(), 1);

  let (_, strct) = adb.structs.pop_first().expect("should have that one struct here");

  assert_eq!(strct.properties.len(), 2);

  dbg!(strct);

  Ok(())
}

#[test]
fn test_temp() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp }  

  <> goal > fn | strct
      
  <> fn > tk:(c:id+) "(" ")" ":"  :ast  { t_FN }
  
  <> strct > tk:(c:id+) "{" "}" :ast  { t_FN }
"##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let mut adb: AscriptDatabase = db.into();

  assert_eq!(adb.structs.len(), 1);

  let (_, strct) = adb.structs.pop_first().expect("should have that one struct here");

  assert_eq!(strct.properties.len(), 2);

  dbg!(strct);

  Ok(())
}

#[test]
fn token_vector() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp }  

  <> goal > tk:( "test" )+
"##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let mut adb: AscriptDatabase = db.into();

  dbg!(&adb);

  let (..) = adb.structs.pop_first().expect("should have that one struct here");
  assert_eq!(adb.structs.len(), 1);

  //assert_eq!(strct.properties.len(), 2);

  Ok(())
}

#[test]
fn builds_rum_lang() -> RadlrResult<()> {
  let source = r##"
  IGNORE { c:sp c:nl }

  <> RS 
     
     > ( table_definition | Block )(+)
  
     :ast { t_Rs, val:$1 }
  
  <> table_definition 
     
     > table_identifier table_row_aliases(*) table_row_types

     :ast { t_Table }
  
  <> table_identifier 
  
     > "[" id sub_table_id primary_key_table? table_attributes? "]"
  
  <> table_attributes 
  
     > ":" id(+)
  
  <> primary_key_table 
     
     > "#" id
  
  <> sub_table_id 
     
     > "." id
  
  <> table_row_aliases 
     
     > "[" (id(+))(+"-") "]"
  
  <> table_row_types 
     
     > "(" (id table_attributes? )(*",") ")"
  
  // Statements
  
  <> for_in_table_stmt > 
  
    "for" id "in" ( id | select_expression ) stmt_block 
  
  <> stmt_block > "{" "}"
  
  
  
  // Expressions
  
  <> select_expression > id "{" "}"
  
  
  <> t_expr 
     > t_expr "|" t_expr 
  
  <> row_method > "todo" 
  
  <> table_method > "todo" 
  
  <> expr > expr "+" expr
          | expr "-" expr
          | expr "*" expr
          | expr "/" expr
          | expr "^" expr
          | expr "-" expr
          | expr "&" expr
          | expr "|" expr
          | expr "&&" expr
          | expr "||" expr
          | "-" expr
          | "~" expr
  
  
  <> num > tk:( "-"? c:num(+) ( "." ( c:num(+) )? ( ("e" | "E") c:num(*) )? ) )
  
  <> id > tk:(  ( c:id | "_" ) ( c:id | "_" | c:num )(*) | c:num(+) ( c:id ) ( c:id | "_" | c:num )(*)  )
  
  
  <> Block > "{" "}"
  
    :ast { t_Block }
  
    
   "##;

  let db = RadlrGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb: AscriptDatabase = db.into();

  dbg!(adb);

  Ok(())
}

#[test]
fn builds_radlr_grammar_ast() -> RadlrResult<()> {
  let resolved_root_path = RadlrGrammar::resolve_to_grammar_file(
    &PathBuf::from("/home/work/projects/lib_radlr/grammars/radlr/3.0.0-pre-bootstrap/radlr.radlr")
      .canonicalize()
      .expect("Grammar not found"),
  )?;

  let db = RadlrGrammar::new().add_source(&resolved_root_path)?.build_db(resolved_root_path, Default::default())?;

  let adb: AscriptDatabase = db.clone().into();

  dbg!(&adb);

  std::hint::black_box(adb);

  Ok(())
}

#[test]
fn builds_rum_lang_grammar_ast() -> RadlrResult<()> {
  let resolved_root_path = RadlrGrammar::resolve_to_grammar_file(
    &PathBuf::from("/home/work/projects/lib_rum_common/crates/language/grammar/low_level.radlr")
      .canonicalize()
      .expect("Grammar not found"),
  )?;
  let db = RadlrGrammar::new().add_source(&resolved_root_path)?.build_db(resolved_root_path, Default::default())?;

  let adb: AscriptDatabase = db.into();
  let multis = AscriptMultis::new(&adb);
  let value = multis.types.iter().find(|v| v.name == "block_list_Value").unwrap();
  dbg!(&multis.types.iter().zip(adb.multi_types.iter()).zip(adb.multi_type_lu.iter()).enumerate().collect::<Vec<_>>());

  let name = StringId("LL_Num".to_string().to_token());
  let ty = AscriptType::Scalar(AscriptScalarType::Struct(name, false));

  dbg!(
    AscriptType::Scalar(AscriptScalarType::Struct(name, false)),
    value.types.0.iter().any(|d| {
      dbg!(*d == ty);
      *d == ty
    })
  );

  assert!(value.types.iter().any(|d| { *d == ty }));

  //dbg!(&adb);

  std::hint::black_box(adb);

  Ok(())
}
