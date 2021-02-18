use anilang::{Deserialize, DeserializeCtx};
use crossterm::style::Colorize;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn run(bin_file: PathBuf, show_bytecode: bool) -> io::Result<()> {
    let mut bin = io::BufReader::new(fs::File::open(bin_file)?);
    let src = anilang::SourceText::deserialize(&mut bin)?;
    let diagnostics = anilang::Diagnostics::new(&src);

    let std = crate::stdlib::make_std();

    let num_scopes = usize::deserialize(&mut bin)?;
    let num_idents = usize::deserialize(&mut bin)?;

    let mut ctx = anilang::DeserializationContext::new(num_scopes, num_idents, Some(std));
    let mut i = 0;

    for _ in 0..(num_scopes + num_idents) {
        let is_ident = bool::deserialize(&mut bin)?;
        if is_ident {
            let id = usize::deserialize(&mut bin)?;
            let ident = Deserialize::deserialize(&mut bin)?;
            ctx.add_ident(id, ident);
        } else {
            let parent_id = usize::deserialize(&mut bin)?;

            let parent_id = if parent_id != usize::MAX {
                Some(parent_id)
            } else {
                None
            };

            ctx.add_scope(i, parent_id);
            i += 1;
        }
    }

    let bytecode = Vec::deserialize_with_context(&mut bin, &mut ctx)?;

    if show_bytecode {
        if let Some(e) = anilang::print_bytecode(&bytecode[..]).err() {
            println!("{} Failed to print bytecode", "ERROR".dark_red());
            println!("Error Message: {}", e);
            return Ok(());
        }
    }

    anilang::Evaluator::evaluate(&bytecode[..], &diagnostics);

    Ok(())
}

pub fn interpret(file: PathBuf, show_ast: bool, show_bytecode: bool) -> crossterm::Result<()> {
    let input = String::from_utf8(fs::read(file)?)?;

    let std = crate::stdlib::make_std();

    let src = anilang::SourceText::new(input.as_str());
    let diagnostics = anilang::Diagnostics::new(&src);

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);

    if show_ast {
        root.prt();
    }

    let bytecode = anilang::Lowerer::lower_with_global(root, &diagnostics, std, false);

    if show_bytecode {
        anilang::print_bytecode(&bytecode[..])?;
    }

    if !diagnostics.any() {
        anilang::Evaluator::evaluate(&bytecode[..], &diagnostics);
    }

    Ok(())
}
