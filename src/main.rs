use crossterm::Result;

mod repl;

fn main() -> Result<()> {
    let mut repl = repl::Repl::new("» ", "· ", repl::get_persistant_file_path());

    let mut global_scope = anilang::Scope::new();

    loop {
        let line = repl.next(crossterm::style::Color::Green)?;

        let src = anilang::SourceText::new(&line);
        let diagnostics = anilang::Diagnostics::new(&src);

        let tokens = anilang::Lexer::lex(&src, &diagnostics);
        let root = anilang::Parser::parse(tokens, &src, &diagnostics);

        if !diagnostics.any() {
            if repl.show_tree {
                root.prt();
            }
            let value =
                anilang::Evaluator::evaluate_with_global(root, &diagnostics, &mut global_scope);
            match value {
                anilang::Value::Null => {}
                value if !diagnostics.any() => println!("{:?}", value),
                _ => {}
            }
        }
    }
}
