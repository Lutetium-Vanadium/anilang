use super::*;

fn lex(text: &str) -> Vec<Token> {
    let src = SourceText::new(text);
    let diagnostics = Diagnostics::new(&src).no_print();
    Lexer::lex(&src, &diagnostics)
}

fn lex_one(text: &str) -> Token {
    let mut tokens = lex(text);
    assert_eq!(tokens.len(), 2);
    tokens.into_iter().next().unwrap()
}

#[test]
fn lexes_properly() {
    let mut tokens = lex("1 + 2 + 3").into_iter();

    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 0, 1));
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 1, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::PlusOperator, 2, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 3, 1)
    );
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 4, 1));
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 5, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::PlusOperator, 6, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 7, 1)
    );
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 8, 1));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 9, 0));
    assert_eq!(tokens.next(), None);
}

#[test]
fn lexes_kind_properly() {
    assert_eq!(lex_one(" ").kind, TokenKind::Whitespace);
    assert_eq!(lex_one("1").kind, TokenKind::Number);
    assert_eq!(lex_one("true").kind, TokenKind::Boolean);
    assert_eq!(lex_one("'str'").kind, TokenKind::String);
    assert_eq!(lex_one("ident").kind, TokenKind::Ident);

    assert_eq!(lex_one(".").kind, TokenKind::DotOperator);
    assert_eq!(lex_one(",").kind, TokenKind::CommaOperator);
    assert_eq!(lex_one("=").kind, TokenKind::AssignmentOperator);
    assert_eq!(lex_one("+").kind, TokenKind::PlusOperator);
    assert_eq!(lex_one("-").kind, TokenKind::MinusOperator);
    assert_eq!(lex_one("*").kind, TokenKind::StarOperator);
    assert_eq!(lex_one("/").kind, TokenKind::SlashOperator);
    assert_eq!(lex_one("%").kind, TokenKind::ModOperator);
    assert_eq!(lex_one("^").kind, TokenKind::CaretOperator);
    assert_eq!(lex_one("++").kind, TokenKind::PlusPlusOperator);
    assert_eq!(lex_one("--").kind, TokenKind::MinusMinusOperator);
    assert_eq!(lex_one("||").kind, TokenKind::OrOperator);
    assert_eq!(lex_one("&&").kind, TokenKind::AndOperator);
    assert_eq!(lex_one("!").kind, TokenKind::NotOperator);
    assert_eq!(lex_one("!=").kind, TokenKind::NEOperator);
    assert_eq!(lex_one("==").kind, TokenKind::EqOperator);
    assert_eq!(lex_one("<").kind, TokenKind::LTOperator);
    assert_eq!(lex_one(">").kind, TokenKind::GTOperator);
    assert_eq!(lex_one("<=").kind, TokenKind::LEOperator);
    assert_eq!(lex_one(">=").kind, TokenKind::GEOperator);

    assert_eq!(lex_one("(").kind, TokenKind::OpenParan);
    assert_eq!(lex_one(")").kind, TokenKind::CloseParan);
    assert_eq!(lex_one("{").kind, TokenKind::OpenBrace);
    assert_eq!(lex_one("}").kind, TokenKind::CloseBrace);
    assert_eq!(lex_one("[").kind, TokenKind::OpenBracket);
    assert_eq!(lex_one("]").kind, TokenKind::CloseBracket);

    assert_eq!(lex_one("if").kind, TokenKind::IfKeyword);
    assert_eq!(lex_one("else").kind, TokenKind::ElseKeyword);
    assert_eq!(lex_one("break").kind, TokenKind::BreakKeyword);
    assert_eq!(lex_one("while").kind, TokenKind::WhileKeyword);
    assert_eq!(lex_one("loop").kind, TokenKind::LoopKeyword);
    assert_eq!(lex_one("let").kind, TokenKind::LetKeyword);

    assert_eq!(lex_one(";").kind, TokenKind::Bad);
}
