mod get_indent;
mod linter;

use crossterm::style::Colorize;
use shelp::LangInterface;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

fn get_persistant_file_path() -> Option<PathBuf> {
    match app_dirs2::data_root(app_dirs2::AppDataType::UserCache) {
        Ok(mut p) => {
            p.push(".anilang-history");
            Some(p)
        }
        Err(_) => None,
    }
}

pub struct AnilangLangInterface;
impl LangInterface for AnilangLangInterface {
    fn print_line(stdout: &mut io::Stdout, lines: &[String], index: usize) -> shelp::Result<()> {
        linter::print_linted(stdout, lines, index)
    }

    fn get_indent(lines: &[String]) -> usize {
        get_indent::get_indent(lines)
    }
}

pub fn run(mut show_ast: bool, mut show_bytecode: bool) {
    let repl = shelp::Repl::<AnilangLangInterface>::new("» ", "· ", get_persistant_file_path())
        .iter(shelp::Color::Green);

    let std = crate::stdlib::make_std();

    let global_scope = Rc::new(anilang::Scope::new(1, Some(std)));

    for line in repl {
        if line.trim() == ".tree" {
            show_ast = !show_ast;
            if show_ast {
                println!("Showing Abstract Syntax Tree")
            } else {
                println!("Hiding Abstract Syntax Tree")
            }
            continue;
        } else if line.trim() == ".bytecode" {
            show_bytecode = !show_bytecode;
            if show_bytecode {
                println!("Showing Bytecode")
            } else {
                println!("Hiding Bytecode")
            }
            continue;
        }

        let src = anilang::SourceText::new(line.as_str());
        let diagnostics = anilang::Diagnostics::new(&src);

        let tokens = anilang::Lexer::lex(&src, &diagnostics);
        let root = anilang::Parser::parse(tokens, &src, &diagnostics);
        if show_ast {
            root.prt();
        }

        let bytecode = anilang::Lowerer::lower_with_global(
            root,
            &diagnostics,
            Rc::clone(&global_scope),
            false,
        );
        if show_bytecode {
            anilang::print_bytecode(&bytecode[..]).unwrap_or_else(|e| {
                println!("{} Failed to print bytecode", "ERROR".dark_red());
                println!("Error Message: {}", e)
            });
        }

        if !diagnostics.any() {
            let value = anilang::Evaluator::evaluate(&bytecode[..], &diagnostics);
            match value {
                anilang::Value::Null => {}
                value if !diagnostics.any() => println!("{:?}", value),
                _ => {}
            }
        }
    }
}
