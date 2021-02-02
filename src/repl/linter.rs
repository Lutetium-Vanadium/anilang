use crossterm::{queue, style};
use std::io::{self, prelude::*};

pub fn print_linted(
    stdout: &mut io::Stdout,
    lines: &[String],
    index: usize,
) -> crossterm::Result<()> {
    match lines[0].trim() {
        "exit" | ".tree" | ".bytecode" | "clear" if lines.len() == 1 => {
            queue!(
                stdout,
                style::SetForegroundColor(style::Color::DarkGreen),
                style::Print(lines[0].as_str())
            )?;
        }
        _ => {
            let src = anilang::SourceText::new(lines);
            let diagnostics = anilang::Diagnostics::new(&src).no_print();
            let mut tokens = anilang::Lexer::lex(&src, &diagnostics)
                .into_iter()
                .peekable();

            let (line_start, line_end) = src.line(index);

            while let Some(token) = tokens.next() {
                if token.text_span.end() > line_start {
                    print_token(
                        // min in case the whole line is a single token
                        &src[line_start..token.text_span.end().min(line_end)],
                        &token.kind,
                        tokens.peek().map(|t| &t.kind),
                        stdout,
                    )?;

                    break;
                }
            }

            while let Some(token) = tokens.next() {
                if token.text_span.end() > line_end {
                    if !token.text_span.is_empty() && token.kind != TokenKind::Whitespace {
                        print_token(
                            &src[token.text_span.start()..line_end],
                            &token.kind,
                            tokens.peek().map(|t| &t.kind),
                            stdout,
                        )?;
                    }

                    break;
                }

                if !token.text_span.is_empty() {
                    print_token(
                        &src[&token.text_span],
                        &token.kind,
                        tokens.peek().map(|t| &t.kind),
                        stdout,
                    )?;
                }
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
    next_token: Option<&TokenKind>,
    stdout: &mut io::Stdout,
) -> crossterm::Result<()> {
    let colour = match token_kind {
        TokenKind::Number | TokenKind::Boolean => PURPLE,
        TokenKind::String => YELLOW,

        TokenKind::IfKeyword
        | TokenKind::ElseKeyword
        | TokenKind::FnKeyword
        | TokenKind::InterfaceKeyword
        | TokenKind::BreakKeyword
        | TokenKind::WhileKeyword
        | TokenKind::LoopKeyword
        | TokenKind::ReturnKeyword
        | TokenKind::LetKeyword => RED,

        TokenKind::AssignmentOperator
        | TokenKind::PlusOperator
        | TokenKind::MinusOperator
        | TokenKind::StarOperator
        | TokenKind::SlashOperator
        | TokenKind::ModOperator
        | TokenKind::CaretOperator
        | TokenKind::OrOperator
        | TokenKind::AndOperator
        | TokenKind::NotOperator
        | TokenKind::NEOperator
        | TokenKind::EqOperator
        | TokenKind::LTOperator
        | TokenKind::GTOperator
        | TokenKind::LEOperator
        | TokenKind::GEOperator
        | TokenKind::ColonColonOperator => RED,

        TokenKind::Comment => Color::DarkGrey,

        TokenKind::Bad => {
            return queue!(
                stdout,
                style::SetForegroundColor(Color::DarkRed),
                style::SetAttribute(style::Attribute::Underlined),
                style::Print(text),
                style::SetAttribute(style::Attribute::NoUnderline),
            );
        }

        TokenKind::Ident => {
            if let Some(TokenKind::OpenParan) = next_token {
                Color::DarkGreen
            } else {
                Color::Reset
            }
        }

        TokenKind::DotOperator
        | TokenKind::CommaOperator
        | TokenKind::ColonOperator
        | TokenKind::RangeOperator
        | TokenKind::OpenParan
        | TokenKind::OpenBracket
        | TokenKind::OpenBrace
        | TokenKind::CloseParan
        | TokenKind::CloseBracket
        | TokenKind::CloseBrace
        | TokenKind::Whitespace
        | TokenKind::EOF => Color::Reset,
    };

    queue!(
        stdout,
        style::SetForegroundColor(colour),
        style::Print(text),
    )
}
