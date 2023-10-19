use sherpa_ascript_beta::*;
use sherpa_core::*;
use sherpa_formatter::*;

const SCRIPT: &'static str = include_str!("format_script.form");

#[test]
fn constructs_ascipt_build_database() -> SherpaResult<()> {
  let source = r###"
  <> A > B+^test :ast { t_B, type: str($test), dang: false, name: $test }

  <> B > "A" | "B" | "C" 
  
  "###;

  let db = SherpaGrammar::new().add_source_from_string(source, "", false)?.build_db("", Default::default())?;

  let adb = AscriptDatabase::from(db);

  let f = FormatterResult::from(SCRIPT).into_result()?;

  let mut ctx: FormatterContext<'_> = FormatterContext::new_with_values(&adb, Default::default());

  ctx.max_width = 20;

  let output = f.write_to_string(&mut ctx, 1024)?;

  println!("{output}");

  Ok(())
}
