mod error;
mod lexer;
mod source_text;
mod text_span;
mod tokens;

fn main() {
    let s = r#"
x = 23123
asd = "asdkadba"
x += 213"#;

    let src = source_text::SourceText::new(s);

    let mut error_bag = error::ErrorBag::new(&src);

    let tokens = lexer::Lexer::lex(&src, &mut error_bag);

    if error_bag.any() {
        print!("{}", error_bag);
    } else {
        println!("[");
        for token in tokens.iter() {
            print!("    ");
            token.prt(&src);
            println!(",");
        }
        println!("]\n")
    }
}
