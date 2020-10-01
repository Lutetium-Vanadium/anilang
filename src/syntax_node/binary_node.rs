use super::Node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

pub struct BinaryNode {
    token_kind: TokenKind,
    span: TextSpan,
    left: Box<dyn Node>,
    right: Box<dyn Node>,
}

impl BinaryNode {
    pub fn new(token: Token, left: Box<dyn Node>, right: Box<dyn Node>) -> Self {
        Self {
            token_kind: token.kind,
            span: token.text_span,
            left,
            right,
        }
    }
}

use std::fmt;
impl fmt::Display for BinaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.token_kind)
    }
}

impl Node for BinaryNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }

    fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_MAGENTA,
            self,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        self.left.prt(indent.clone(), false);
        self.right.prt(indent, true);
    }
}
