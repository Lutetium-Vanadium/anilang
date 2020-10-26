/// Counts the number of open braces, brackets and parans
pub fn get_indent(lines: &[String]) -> u32 {
    let mut brace = 0;
    let mut bracket = 0;
    let mut paran = 0;

    for line in lines {
        let src = anilang::SourceText::new(line);
        let diagnostics = anilang::Diagnostics::new(&src).no_print();
        let tokens = anilang::Lexer::lex(&src, &diagnostics);
        for i in tokens {
            match i.kind {
                anilang::TokenKind::OpenBrace => brace += 1,
                anilang::TokenKind::CloseBrace => brace -= 1,
                anilang::TokenKind::OpenBracket => bracket += 1,
                anilang::TokenKind::CloseBracket => bracket -= 1,
                anilang::TokenKind::OpenParan => paran += 1,
                anilang::TokenKind::CloseParan => paran -= 1,
                _ => {}
            }
        }
    }

    brace + bracket + paran
}
