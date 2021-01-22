use super::*;

fn lex(text: &str) -> Vec<Token> {
    let src = SourceText::new(text);
    let diagnostics = Diagnostics::new(&src).no_print();
    Lexer::lex(&src, &diagnostics)
}

fn lex_one(text: &str) -> Token {
    let tokens = lex(text);
    assert!(tokens.len() == 2, "Expected 2 tokens, got {:#?}", tokens);
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

    assert_eq!(lex_one("// random comment").kind, TokenKind::Comment);
    assert_eq!(lex_one("/* random comment */").kind, TokenKind::Comment);
    assert_eq!(lex_one("/* random \n comment */").kind, TokenKind::Comment);

    assert_eq!(lex_one(".").kind, TokenKind::DotOperator);
    assert_eq!(lex_one("..").kind, TokenKind::RangeOperator);
    assert_eq!(lex_one(",").kind, TokenKind::CommaOperator);
    assert_eq!(lex_one(":").kind, TokenKind::ColonOperator);
    assert_eq!(lex_one("=").kind, TokenKind::AssignmentOperator);
    assert_eq!(lex_one("+").kind, TokenKind::PlusOperator);
    assert_eq!(lex_one("-").kind, TokenKind::MinusOperator);
    assert_eq!(lex_one("*").kind, TokenKind::StarOperator);
    assert_eq!(lex_one("/").kind, TokenKind::SlashOperator);
    assert_eq!(lex_one("%").kind, TokenKind::ModOperator);
    assert_eq!(lex_one("^").kind, TokenKind::CaretOperator);
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
    assert_eq!(lex_one("return").kind, TokenKind::ReturnKeyword);
    assert_eq!(lex_one("while").kind, TokenKind::WhileKeyword);
    assert_eq!(lex_one("loop").kind, TokenKind::LoopKeyword);
    assert_eq!(lex_one("let").kind, TokenKind::LetKeyword);
    assert_eq!(lex_one("fn").kind, TokenKind::FnKeyword);

    assert_eq!(lex_one(";").kind, TokenKind::Bad);
}

#[test]
fn ignores_singleline_comment() {
    let mut tokens = lex("1 + 2// + 3").into_iter();

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
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Comment, 5, 5));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 11, 0));
    assert_eq!(tokens.next(), None);

    let mut tokens = lex("1 + 2 // random comment\n + 3").into_iter();

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
        Token::new(TokenKind::Comment, 6, 17)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 24, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::PlusOperator, 25, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 26, 1)
    );
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 27, 1));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 28, 0));
    assert_eq!(tokens.next(), None);
}

#[test]
fn ignores_multiline_comment() {
    let mut tokens = lex("1 + 2/* + 3*/").into_iter();

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
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Comment, 5, 8));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 13, 0));
    assert_eq!(tokens.next(), None);

    let mut tokens = lex("1 + 2 /* random comment */ + 3").into_iter();

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
        Token::new(TokenKind::Comment, 6, 20)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 26, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::PlusOperator, 27, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 28, 1)
    );
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 29, 1));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 30, 0));
    assert_eq!(tokens.next(), None);

    let mut tokens = lex("1 + 2 /* random\n comment */ + 3").into_iter();

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
        Token::new(TokenKind::Comment, 6, 21)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 27, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::PlusOperator, 28, 1)
    );
    assert_eq!(
        tokens.next().unwrap(),
        Token::new(TokenKind::Whitespace, 29, 1)
    );
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::Number, 30, 1));
    assert_eq!(tokens.next().unwrap(), Token::new(TokenKind::EOF, 31, 0));
    assert_eq!(tokens.next(), None);
}
