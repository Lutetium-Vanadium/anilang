mod lexer;
mod text_span;
mod tokens;

fn main() {
    let s = r#"
        x = 23123
        asd = "asdkadba"
        x += 213
    "#;

    let lexer = lexer::Lexer::new(s);

    println!("{}", lexer);
}
