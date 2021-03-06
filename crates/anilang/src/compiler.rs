use anilang::Serialize;
use crossterm::Result;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub fn compile(
    input_file: PathBuf,
    output_file: PathBuf,
    show_ast: bool,
    show_bytecode: bool,
) -> Result<()> {
    let input = String::from_utf8(fs::read(input_file)?)?;

    let src = anilang::SourceText::new(input.as_str());
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

        let mut idents = HashSet::new();

        // Serialize scopes
        count_scopes(&bytecode[..], &mut idents).serialize(&mut output_file)?;
        idents.len().serialize(&mut output_file)?;

        idents.clear();

        serialize_scopes(&bytecode[..], &mut output_file, &mut idents)?;

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

fn count_scopes(bytecode: &[Instruction], idents: &mut HashSet<usize>) -> usize {
    let mut num_scopes = 0;

    for instr in bytecode {
        match &instr.kind {
            InstructionKind::PushVar { .. } => num_scopes += 1,
            InstructionKind::Push {
                value: Value::Function(func),
            } => {
                if let Some(func) = func.as_anilang_fn() {
                    num_scopes += count_scopes(&func.body[..], idents)
                }
            }
            InstructionKind::Load { ident, .. } | InstructionKind::Store { ident, .. } => {
                let ident = to_usize(ident);
                if !idents.contains(&ident) {
                    idents.insert(ident);
                }
            }
            _ => {}
        }
    }

    num_scopes
}

fn serialize_scopes(
    bytecode: &[Instruction],
    output_file: &mut fs::File,
    idents: &mut HashSet<usize>,
) -> Result<()> {
    for instr in bytecode {
        match &instr.kind {
            InstructionKind::PushVar { scope } => {
                false.serialize(output_file)?;
                if let Some(parent_id) = scope.parent_id() {
                    parent_id.serialize(output_file)?;
                } else {
                    usize::MAX.serialize(output_file)?;
                }
            }
            InstructionKind::Push {
                value: Value::Function(func),
            } => {
                if let Some(func) = func.as_anilang_fn() {
                    serialize_scopes(&func.body[..], output_file, idents)?;
                }
            }
            InstructionKind::Store { ident, .. } | InstructionKind::Load { ident } => {
                let ident_usize = to_usize(ident);
                if !idents.contains(&ident_usize) {
                    true.serialize(output_file)?;
                    ident_usize.serialize(output_file)?;
                    ident[..].serialize(output_file)?;
                    idents.insert(ident_usize);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn to_usize(rc: &Rc<str>) -> usize {
    Rc::as_ptr(rc) as *const u8 as usize
}
