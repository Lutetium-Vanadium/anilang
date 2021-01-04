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
        bytecode.len().serialize(&mut output_file)?;

        for instr in bytecode.into_iter() {
            instr.serialize(&mut output_file)?;
        }
    }

    Ok(())
}
