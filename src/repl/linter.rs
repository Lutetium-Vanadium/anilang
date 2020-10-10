use crossterm::{queue, style};
use std::io::{self, prelude::*};

pub fn print_linted(stdout: &mut io::Stdout, line: &str) -> crossterm::Result<()> {
    match line.trim() {
        "exit" | "clear" => {
            queue!(
                stdout,
                style::SetForegroundColor(style::Color::DarkGreen),
                style::Print(line)
            )?;
        }
        _ => {
            let src = anilang::SourceText::new(line);
            let mut diagnostics = anilang::Diagnostics::new(&src).no_print();
            let tokens = anilang::Lexer::lex(&src, &mut diagnostics);

            for token in tokens {
                print_token(&src[&token.text_span], &token.kind, stdout)?;
            }
        }
    }

    queue!(stdout, style::ResetColor)
}

const PURPLE: Color = Color::Rgb {
    r: 174,
    g: 129,
    b: 255,
};
const YELLOW: Color = Color::Rgb {
    r: 230,
    g: 219,
    b: 116,
};
const RED: Color = Color::Rgb {
    r: 249,
    g: 38,
    b: 114,
};

use anilang::TokenKind;
use style::Color;
fn print_token(
    text: &str,
    token_kind: &TokenKind,
    stdout: &mut io::Stdout,
) -> crossterm::Result<()> {
    let colour = match token_kind {
        TokenKind::Number | TokenKind::Boolean => PURPLE,
        TokenKind::String(_) => YELLOW,

        TokenKind::IfKeyword
        | TokenKind::ElseKeyword
        | TokenKind::BreakKeyword
        | TokenKind::WhileKeyword
        | TokenKind::LoopKeyword
        | TokenKind::LetKeyword => RED,

        TokenKind::AssignmentOperator
        | TokenKind::PlusOperator
        | TokenKind::MinusOperator
        | TokenKind::StarOperator
        | TokenKind::SlashOperator
        | TokenKind::ModOperator
        | TokenKind::CaretOperator
        | TokenKind::PlusPlusOperator
        | TokenKind::MinusMinusOperator
        | TokenKind::OrOperator
        | TokenKind::AndOperator
        | TokenKind::NotOperator
        | TokenKind::NEOperator
        | TokenKind::EqOperator
        | TokenKind::LTOperator
        | TokenKind::GTOperator
        | TokenKind::LEOperator
        | TokenKind::GEOperator => RED,

        TokenKind::Bad => {
            return queue!(
                stdout,
                style::SetForegroundColor(Color::DarkRed),
                style::SetAttribute(style::Attribute::Underlined),
                style::Print(text),
                style::SetAttribute(style::Attribute::NoUnderline),
            );
        }

        _ => Color::Reset,
    };

    queue!(
        stdout,
        style::SetForegroundColor(colour),
        style::Print(text),
    )
}
