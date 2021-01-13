use anilang::Serialize;
use crossterm::Result;
use std::fs;
use std::path::PathBuf;

pub fn compile(
    input_file: PathBuf,
    output_file: PathBuf,
    show_ast: bool,
    show_bytecode: bool,
) -> Result<()> {
    let input = String::from_utf8(fs::read(input_file)?)?;

    let src = anilang::SourceText::new(&input);
    let diagnostics = anilang::Diagnostics::new(&src);

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);

    if show_ast {
        root.prt();
    }

    let bytecode = anilang::Lowerer::lower(root, &diagnostics, true);

    if show_bytecode {
        anilang::print_bytecode(&bytecode[..])?;
    }

    if !diagnostics.any() {
        let mut output_file = fs::File::create(output_file)?;
        src.serialize(&mut output_file)?;

        // Serialize scopes
        count_scopes(&bytecode[..]).serialize(&mut output_file)?;

        serialize_scopes(&bytecode[..], &mut output_file)?;

        // Serialize the Instructions
        bytecode.serialize(&mut output_file)?;

        println!("Compiled with {} warnings", diagnostics.num_warnings());
    } else {
        println!(
            "Aborted with {} errors and {} warnings",
            diagnostics.num_errors(),
            diagnostics.num_warnings()
        );
    }

    Ok(())
}

use anilang::{Instruction, InstructionKind, Value};

fn count_scopes(bytecode: &[Instruction]) -> usize {
    let mut num_scopes = 0;

    for instr in bytecode {
        match &instr.kind {
            InstructionKind::PushVar { .. } => num_scopes += 1,
            InstructionKind::Push { value } => {
                if let Value::Function(func) = value {
                    num_scopes += count_scopes(&func.body[..])
                }
            }
            _ => {}
        }
    }

    num_scopes
}

fn serialize_scopes(bytecode: &[Instruction], output_file: &mut fs::File) -> Result<()> {
    for instr in bytecode {
        match &instr.kind {
            InstructionKind::PushVar { scope } => {
                if let Some(parent_id) = scope.parent_id() {
                    (parent_id + 1).serialize(output_file)?;
                } else {
                    0usize.serialize(output_file)?;
                }
            }
            InstructionKind::Push { value } => {
                if let Value::Function(func) = value {
                    serialize_scopes(&func.body[..], output_file)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
