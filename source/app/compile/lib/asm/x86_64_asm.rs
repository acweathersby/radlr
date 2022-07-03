use std::collections::BTreeMap;
use std::io::Result;
use std::io::Write;

use hctk::bytecode::constants::*;

use hctk::grammar::get_exported_productions;
use hctk::grammar::ExportedProduction;
use hctk::types::*;

use crate::writer::x86_64_writer::X8664Writer;

pub fn _undefined<W: Write, T: X8664Writer<W>>(
    _grammar: &GrammarStore,
    _bytecode: &[u32],
    _writer: &mut T,
) -> Result<()>
{
    Ok(())
}

const parse_context_size: usize =
    std::mem::size_of::<ASMParserContext<UTF8StringReader>>();

const state_stack_size: usize = std::mem::size_of::<ASMParserContextStack>();

pub fn write_preamble<W: Write, T: X8664Writer<W>>(
    grammar: &GrammarStore,
    writer: &mut T,
) -> Result<()>
{
    // Disclaimer and constants
    writer
        .comment_line("This is a parser generated by the Hydrocarbon Toolkit.")?
        .comment_line("Any modifications may be overwritten without warning!")?
        .newline()?
        .constant("FAIL_STATE_MASK          ", &FAIL_STATE_MASK.to_string())?
        .constant("NORMAL_STATE_MASK        ", &NORMAL_STATE_MASK.to_string())?
        .constant("PEEK_MODE_FLAG           ", &PEEK_MODE_FLAG.to_string())?
        .constant(
            "STATE_TYPE_MASK        ",
            &(NORMAL_STATE_MASK + FAIL_STATE_MASK).to_string(),
        )?
        .constant(
            "STATE_STACK_START        ",
            &(parse_context_size - state_stack_size).to_string(),
        )?
        .constant("STATE_STACK_BASE_SIZE    ", &state_stack_size.to_string())?
        .constant("foreign_rsp_offset       ", &(8 * 0).to_string())?
        .constant("local_rsp_offset         ", &(8 * 1).to_string())?
        .constant("stack_top_addr_offset    ", &(8 * 2).to_string())?
        .constant("state_u64_data_offset    ", &(8 * 3).to_string())?
        .constant("struct_reader_ptr_offset ", &(8 * 4).to_string())?
        .constant("fn_class_offset          ", &(8 * 5).to_string())?
        .constant("fn_codepoint_offset      ", &(8 * 6).to_string())?
        .constant("fn_word_offset           ", &(8 * 7).to_string())?
        .constant("fn_byte_offset           ", &(8 * 8).to_string())?
        .constant("fn_set_cursor_to_offset  ", &(8 * 9).to_string())?
        .constant("fn_get_line_data_offset  ", &(8 * 10).to_string())?
        .constant("fn_get_length_data_offset", &(8 * 11).to_string())?
        .newline()?
        .newline()?;

    writer.section(".text")?;

    // Entry Points

    writer
        .code("default rel")?
        .commented_code(
            "global construct_context",
            "adjust stack information and initializes context variables",
        )?
        .commented_code(
            "global prime_context",
            "resets context and pops a starting state onto the stack",
        )?
        .commented_code(
            "global destroy_context",
            "freezes any extend stacks that been allocated to the context",
        )?
        .commented_code(
            "global next",
            "continue processing from the last state and yield the next action",
        )?
        .newline()?
        .newline()?;

    writer
        .label("construct_context", false)?
        .commented_code(
            "mov [rdi + foreign_rsp_offset], rsp",
            "Preserve the original stack pointer",
        )?
        .newline()?
        .comment_line("  Reserve the start of our state stack")?
        .code("mov rsp, rdi")?
        .code("add rsp, ( STATE_STACK_START + STATE_STACK_BASE_SIZE )")?
        .newline()?
        .newline()?
        .comment_line(
            "  Calculate the top (rsp, so actually the bottom) of our stack",
        )?
        .code("mov r10, rdi")?
        .code("add r10, STATE_STACK_START")?
        .code("mov [rdi + stack_top_addr_offset], r10")?
        .newline()?
        .comment_line("  Save our state stack")?
        .code("mov [rdi + local_rsp_offset], rsp")?
        .newline()?
        .comment_line("  Restore the original stack pointer")?
        .code("mov rsp, [rdi + foreign_rsp_offset]")?
        .newline()?
        .code("ret")?;

    save_context(
        restore_context(writer.label("prime_context", false)?)?
            .comment_line("  Configure our parse state")?
            .comment_line("  Set the normal mode for our parse state")?
            .code("mov r10, ( NORMAL_STATE_MASK )")?
            .code("mov [rbx + state_u64_data_offset], r10")?
            .code("mov r11, rax")?
            .code("xor eax, eax")?
            .inline_grammar(
                grammar,
                |writer, grammar: &GrammarStore| -> Result<&mut T> {
                    // Create a simple lookup for a production entrypoint
                    let vec = get_exported_productions(grammar);
                    let last = vec.len() - 1;
                    for (
                        i,
                        ExportedProduction {
                            export_name,
                            guid_name,
                            ..
                        },
                    ) in vec.iter().enumerate()
                    {
                        let (is_first, is_last) = (i == 0, last == i);

                        if !is_first {
                            writer.label(&format!("opt_{}", i), true)?;
                        }
                        writer.commented_code(
                            &format!("cmp  r11, {}", i),
                            &format!("{} as {}", guid_name, export_name),
                        )?;
                        if !is_last {
                            writer.code(&format!("jne .opt_{}", i + 1))?;
                        } else {
                            writer.code("jne push_state")?;
                        }
                        writer.code(&format!(
                            "lea rax, [rel state_{}]",
                            guid_name
                        ))?;
                        if !is_last {
                            writer.code("jmp push_state")?;
                        }
                    }
                    Ok(writer)
                },
            )?
            .label("push_state", true)?
            .comment_line("  Add our stack sentinel")?
            .code("xor r10d, r10d")?
            .code("push r10")?
            .code("push r10")?
            .comment_line("  Push our entry state onto the stack")?
            .commented_code("push rcx", "state metadata")?
            .commented_code("push rax", "state address")?,
    )?
    .code("ret")?;

    writer
        .newline()?
        .label("next", false)?
        .inline(restore_context)?
        .comment_line("  Restore our parse state")?
        .code("mov rcx, [rbx + state_u64_data_offset]")?
        .newline()?
        .label("dispatch_loop", false)?
        .commented_code("pop r8", "state address")?
        .commented_code("pop r9", "state metadata")?
        .newline()?
        .comment_line("Test for bottom of stack sentinel")?
        .commented_code("test r9,r9", "test to see if state metadata is 0")?
        .commented_code(
            "JZ end_parse",
            "stop parsing if we have no more actions",
        )?
        .newline()?
        .comment_line("Test for state appropriateness in current context")?
        .commented_code("mov r10, rcx", "get copy of our context state")?
        .commented_code("and r10, STATE_TYPE_MASK", "mask out the mode")?
        .commented_code(
            "test r10, r9",
            "if this is zero then we are not allowed to use this state in the current context",
        )?
        .code("jz dispatch_loop")?
        .newline()?
        .comment_line("Dispatch!")?
        .commented_code("jmp r8", "go to the state")?
        .newline()?
        .label("emit_action", false)?
        .comment_line("  Save our parse state")?
        .code("mov [rbx + state_u64_data_offset], rcx")?
        .inline(save_context)?
        .code("xor eax, eax")?
        .code("inc eax")?
        .code("ret")?
        .label("end_parse", false)?
        .inline(save_context)?
        .code("xor eax, eax")?
        .code("ret")?;

    writer
        .newline()?
        .label("destroy_context", false)?
        .code("ret")?;

    // intentional

    Ok(())
}

fn restore_context<W: Write, T: X8664Writer<W>>(
    writer: &mut T,
) -> Result<&mut T>
{
    writer
        .comment_line("Restoring context")?
        .commented_code("push rbx", "preserve the base pointer")?
        .commented_code(
            "mov rbx, rdi",
            "make our offsets relative to the parse context",
        )?
        .commented_code(
            "mov [rbx + foreign_rsp_offset], rsp",
            "preserve the outside stack",
        )?
        .commented_code(
            "mov rsp, [rbx + local_rsp_offset]",
            "restore our local state stack",
        )
}

fn save_context<W: Write, T: X8664Writer<W>>(writer: &mut T) -> Result<&mut T>
{
    writer
        .comment_line("Saving context")?
        .commented_code(
            "mov [rbx + local_rsp_offset], rsp",
            "preserve our local stack",
        )?
        .commented_code(
            "mov rsp, [rbx + foreign_rsp_offset]",
            "restore the outside stack",
        )?
        .commented_code("pop rbx", "restore the base pointer")
}

pub fn compile_from_bytecode<W: Write, T: X8664Writer<W>>(
    grammar: &GrammarStore,
    bytecode: &[u32],
    writer: &mut T,
    offset_to_name: &BTreeMap<u32, String>,
) -> Result<()>
{
    write_preamble(grammar, writer);
    let mut offset = FIRST_STATE_OFFSET;

    while offset < bytecode.len() as u32 {
        offset =
            write_state(grammar, bytecode, writer, offset_to_name, offset)?;
    }

    Ok(())
}

pub fn write_state<W: Write, T: X8664Writer<W>>(
    grammar: &GrammarStore,
    bytecode: &[u32],
    writer: &mut T,
    offset_to_name: &BTreeMap<u32, String>,
    mut offset: u32,
) -> Result<u32>
{
    offset += 1;

    if let Some(name) = offset_to_name.get(&offset) {
        writer
            .label(&format!("state_{}", name), false)?
            .code("jmp dispatch_loop")?;
    } else {
        writer
            .label(&format!("state_{:X}", offset), false)?
            .code("jmp dispatch_loop")?;
    }

    Ok(offset)
}

#[cfg(test)]
mod test_x86_generation
{
    use std::collections::BTreeMap;

    use hctk::bytecode::compile_bytecode;
    use hctk::grammar::get_production_id_by_name;
    use hctk::grammar::parse::compile_ir_ast;
    use hctk::intermediate::state::generate_production_states;

    use crate::asm::x86_64_asm::compile_from_bytecode;
    use crate::writer::nasm_writer::NasmWriter;
    use crate::writer::x86_64_writer::X8664Writer;

    #[test]
    fn test_nasm_output_on_trivial_grammar()
    {
        use hctk::debug::compile_test_grammar;

        let grammar = compile_test_grammar("<> A > \\h ? \\e ? \\l \\l \\o");

        let output = compile_bytecode(&grammar, 1);

        let mut writer = NasmWriter::new(Vec::<u8>::new());

        let result = compile_from_bytecode(
            &grammar,
            &output.bytecode,
            &mut writer,
            &output.get_inverted_state_lookup(),
        );

        assert!(result.is_ok());

        println!(
            "\n\n{}\n\n",
            String::from_utf8(writer.into_writer()).unwrap()
        );
    }
}
