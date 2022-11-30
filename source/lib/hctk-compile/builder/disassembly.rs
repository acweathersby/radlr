use super::pipeline::PipelineTask;
use crate::builder::disclaimer::DISCLAIMER;
use hctk_core::{
  debug::{generate_disassembly, BytecodeGrammarLookups},
  types::HCError,
  writer::code_writer::CodeWriter,
};
use std::io::BufWriter;

/// Generate a disassembly file of the grammar bytecode
pub fn build_bytecode_disassembly() -> PipelineTask {
  PipelineTask {
    fun: Box::new(move |task_ctx| {
      let output_path = task_ctx.get_source_output_dir().clone();
      let grammar = &task_ctx.get_journal().grammar().unwrap();

      if let Ok(parser_data_file) =
        task_ctx.create_file(output_path.join(format!("./{}_dasm.txt", grammar.id.name)))
      {
        let Some(bytecode) = task_ctx.get_bytecode() else {
          return Err(vec![HCError::from("Cannot disassemble Bytecode: Bytecode is not available")]);
        };

        let mut writer = CodeWriter::new(BufWriter::new(parser_data_file));

        writer.write(&DISCLAIMER("Parser Data", "//!", task_ctx)).unwrap();

        writer
          .write(&generate_disassembly(bytecode, Some(&BytecodeGrammarLookups::new(grammar))))
          .unwrap();
      }
      Ok(None)
    }),
    require_ascript: false,
    require_bytecode: true,
  }
}
