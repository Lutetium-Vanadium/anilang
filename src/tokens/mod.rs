mod token_kind;

use crate::text_span::TextSpan;
use std::fmt;

pub use token_kind::TokenKind;

pub struct Token {
    token_kind: TokenKind,
    text_span: TextSpan,
}

impl Token {
    pub fn new(token_kind: TokenKind, start: usize, len: usize) -> Token {
        Token {
            token_kind,
            text_span: TextSpan::new(start, len),
        }
    }

    pub fn prt(&self, f: &mut fmt::Formatter, text: &str) -> fmt::Result {
        write!(
            f,
            "{:?} - ({}, {}) = '{}'",
            self.token_kind,
            self.text_span.start(),
            self.text_span.end(),
            if self.token_kind == TokenKind::Whitespace {
                ""
            } else {
                self.text_span.get_str(text)
            }
        )
    }
}
