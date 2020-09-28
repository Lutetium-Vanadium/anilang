mod error;
mod lexer;
mod parser;
mod source_text;
mod syntax_node;
mod text_span;
mod tokens;

fn main() {
    let source_code = r#"
x = 23123
asd = "asdkadba"
if x == 23123 {
    x + 213
} else {
    x - 123
}
"#;

    let src = source_text::SourceText::new(source_code);
    let mut error_bag = error::ErrorBag::new(&src);

    let tokens = lexer::Lexer::lex(&src, &mut error_bag);
    let root = parser::Parser::parse(tokens, &src, &mut error_bag);

    if error_bag.any() {
        println!("{}", error_bag);
    } else {
        println!("{}", root);
    }
}
