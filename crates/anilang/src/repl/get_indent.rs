const INDENT_WIDTH: usize = 4;

/// Counts the number of open braces, brackets and parans
pub fn get_indent(lines: &[String]) -> usize {
    let mut indent = 0;

    let src = anilang::SourceText::new(lines);
    let diagnostics = anilang::Diagnostics::new(&src).no_print();
    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    // The last token is an EOF token. Since source is made from &[String], the last character is
    // guaranteed to be a '\n', the only case when second last token (which will contain the '\n')
    // is a string or a comment, is when the '\n' is part of the token, and so has no closing
    // delimiter
    if tokens.len() > 1
        && matches!(
            tokens[tokens.len() - 2].kind,
            anilang::TokenKind::String | anilang::TokenKind::Comment
        )
    {
        indent += 1;
    }

    for i in tokens {
        match i.kind {
            anilang::TokenKind::OpenBrace => indent += 1,
            anilang::TokenKind::CloseBrace => indent -= 1,
            anilang::TokenKind::OpenBracket => indent += 1,
            anilang::TokenKind::CloseBracket => indent -= 1,
            anilang::TokenKind::OpenParan => indent += 1,
            anilang::TokenKind::CloseParan => indent -= 1,
            _ => {}
        }
    }

    indent * INDENT_WIDTH
}
