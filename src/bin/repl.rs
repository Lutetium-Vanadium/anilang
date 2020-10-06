use anilang::{error, evaluator, lexer, parser, source_text, value, Node};

fn main() {
    let source_code = r#"
let x = 10
if x == 10 {
    x = x + 1.
}

if x > 10 {
    x += .22
}

while x < 100 {
    x = x + 10
}
x
"#;

    let src = source_text::SourceText::new(source_code);
    let mut diagnostics = error::Diagnostics::new(&src);

    let tokens = lexer::Lexer::lex(&src, &mut diagnostics);
    let root = parser::Parser::parse(tokens, &src, &mut diagnostics);

    if diagnostics.any() {
        return;
    }

    root.prt(String::new(), true);
    let value = evaluator::Evaluator::evaluate(root, &mut diagnostics);
    if value != value::Value::Null && !diagnostics.any() {
        println!("{}", value);
    }
}
