mod token_kind;

#[cfg(test)]
mod tests;

use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use std::fmt;

pub use token_kind::TokenKind;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text_span: TextSpan,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, len: usize) -> Token {
        Token {
            kind,
            text_span: TextSpan::new(start, len),
        }
    }

    pub fn unary_precedence(&self) -> u8 {
        self.kind.unary_precedence()
    }

    pub fn binary_precedence(&self) -> u8 {
        self.kind.binary_precedence()
    }

    pub fn is_calc_assign(&self) -> bool {
        self.kind.is_calc_assign()
    }

    #[allow(dead_code)]
    pub fn prt(&self, src: &SourceText) {
        print!(
            "{:?} - ({}, {}) = '{}'",
            self.kind,
            self.text_span.start(),
            self.text_span.end(),
            if self.kind == TokenKind::Whitespace {
                ""
            } else {
                &src[&self.text_span]
            }
        )
    }

    pub fn fmt(&self, f: &mut fmt::Formatter, src: &SourceText) -> fmt::Result {
        write!(
            f,
            "{:?} - ({}, {}) = '{}'",
            self.kind,
            self.text_span.start(),
            self.text_span.end(),
            if self.kind == TokenKind::Whitespace {
                ""
            } else {
                &src[&self.text_span]
            }
        )
    }
}
