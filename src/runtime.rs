use anilang::Serialize;
use crossterm::style::Colorize;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn run(bin_file: PathBuf, show_bytecode: bool) -> io::Result<()> {
    let mut bin = io::BufReader::new(fs::File::open(bin_file)?);
    let src = anilang::SourceText::deserialize(&mut bin)?;
    let diagnostics = anilang::Diagnostics::new(&src);

    let len = usize::deserialize(&mut bin)?;
    let mut bytecode = Vec::with_capacity(len);

    for _ in 0..len {
        bytecode.push(anilang::Instruction::deserialize(&mut bin)?);
    }

    if show_bytecode {
        if let Some(e) = anilang::print_bytecode(&bytecode[..]).err() {
            println!("{} Failed to print bytecode", "ERROR".dark_red());
            println!("Error Message: {}", e);
            return Ok(());
        }
    }

    anilang::Evaluator::evaluate(&bytecode, &diagnostics);

    Ok(())
}

pub fn interpret(file: PathBuf, show_ast: bool, show_bytecode: bool) -> crossterm::Result<()> {
    let input = String::from_utf8(fs::read(file)?)?;

    let src = anilang::SourceText::new(&input);
    let diagnostics = anilang::Diagnostics::new(&src);

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);

    if show_ast {
        root.prt();
    }

    let bytecode = anilang::Lowerer::lower(root, &diagnostics, false);

    if show_bytecode {
        anilang::print_bytecode(&bytecode[..])?;
    }

    if !diagnostics.any() {
        anilang::Evaluator::evaluate(&bytecode, &diagnostics);
    }

    Ok(())
}
