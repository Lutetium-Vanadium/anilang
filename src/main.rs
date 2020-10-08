use crossterm::Result;

mod history;
mod repl;

fn main() -> Result<()> {
    let mut repl = repl::Repl::new();

    let mut global_scope = anilang::Scope::new();

    loop {
        let line = repl.next("» ", crossterm::style::Color::Green)?;

        let src = anilang::SourceText::new(&line);
        let mut diagnostics = anilang::Diagnostics::new(&src);

        let tokens = anilang::Lexer::lex(&src, &mut diagnostics);
        let root = anilang::Parser::parse(tokens, &src, &mut diagnostics);

        if !diagnostics.any() {
            // root.prt(String::new(), true);
            let value =
                anilang::Evaluator::evaluate_with_global(root, &mut diagnostics, &mut global_scope);
            if value != anilang::Value::Null && !diagnostics.any() {
                println!("{}", value);
            }
        }

        repl.push(line);
    }
}
