mod colour;
mod error;
mod evaluator;
mod lexer;
mod parser;
mod source_text;
mod syntax_node;
mod text_span;
mod tokens;
mod types;
mod value;

use syntax_node::Node;

fn main() {
    let source_code = r#"
x = 10
if x == 10 {
    x = x + 1
}

if x > 10 {
    x += 2.2
}

while x < 100 {
    x = x + 10
}
x
"#;

    let src = source_text::SourceText::new(source_code);
    let mut error_bag = error::ErrorBag::new(&src);

    let tokens = lexer::Lexer::lex(&src, &mut error_bag);
    let root = parser::Parser::parse(tokens, &src, &mut error_bag);

    if error_bag.any() {
        return;
    }

    root.prt(String::new(), true);
    let value = evaluator::Evaluator::evaluate(root, &mut error_bag);
    println!("{}", value);
}
