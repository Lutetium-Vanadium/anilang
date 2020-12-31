use anilang::Serialize;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn run(bin_file: PathBuf) -> io::Result<()> {
    let mut bin = io::BufReader::new(fs::File::open(bin_file)?);
    let src = anilang::SourceText::deserialize(&mut bin)?;
    let diagnostics = anilang::Diagnostics::new(&src);

    let len = usize::deserialize(&mut bin)?;
    let mut bytecode = Vec::with_capacity(len);

    for _ in 0..len {
        bytecode.push(anilang::Instruction::deserialize(&mut bin)?);
    }

    anilang::Evaluator::evaluate(&bytecode, &diagnostics);

    Ok(())
}

pub fn interpret(file: PathBuf) -> crossterm::Result<()> {
    let input = String::from_utf8(fs::read(file)?)?;

    let src = anilang::SourceText::new(&input);
    let diagnostics = anilang::Diagnostics::new(&src);

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);

    let bytecode = anilang::Lowerer::lower(root, &diagnostics, false);

    if !diagnostics.any() {
        anilang::Evaluator::evaluate(&bytecode, &diagnostics);
    }

    Ok(())
}
