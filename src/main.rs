mod get_indent;
mod linter;

use crossterm::Result;
use shelp::LangInterface;
use std::io;
use std::path::PathBuf;

pub fn get_persistant_file_path() -> Option<PathBuf> {
    match app_dirs2::data_root(app_dirs2::AppDataType::UserCache) {
        Ok(mut p) => {
            p.push(".anilang-history");
            Some(p)
        }
        Err(_) => None,
    }
}

struct AnilangLangInterface;
impl LangInterface for AnilangLangInterface {
    fn print_line(stdout: &mut io::Stdout, line: &str) -> shelp::Result<()> {
        linter::print_linted(stdout, line)
    }

    fn get_indent(lines: &[String]) -> usize {
        get_indent::get_indent(lines)
    }
}

fn main() -> Result<()> {
    let repl = shelp::Repl::<AnilangLangInterface>::new("» ", "· ", get_persistant_file_path())
        .iter(shelp::Color::Green);

    let mut global_scope = anilang::Scope::new();
    let mut show_ast = false;
    let mut show_bytecode = false;

    for line in repl {
        if line == ".tree" {
            show_ast = !show_ast;
            if show_ast {
                println!("Showing Abstract Syntax Tree")
            } else {
                println!("Hiding Abstract Syntax Tree")
            }
            continue;
        } else if line == ".bytecode" {
            show_bytecode = !show_bytecode;
            if show_ast {
                println!("Showing Bytecode")
            } else {
                println!("Hiding Bytecode")
            }
            continue;
        }

        let src = anilang::SourceText::new(&line);
        let diagnostics = anilang::Diagnostics::new(&src);

        let tokens = anilang::Lexer::lex(&src, &diagnostics);
        let root = anilang::Parser::parse(tokens, &src, &diagnostics);
        if show_ast {
            root.prt();
        }

        let bytecode = anilang::Lowerer::lower(root, &diagnostics, false);

        if !diagnostics.any() {
            let value = anilang::Evaluator::evaluate_with_global(
                &bytecode,
                &diagnostics,
                &mut global_scope,
            );
            match value {
                anilang::Value::Null => {}
                value if !diagnostics.any() => println!("{:?}", value),
                _ => {}
            }
        }
    }

    Ok(())
}
