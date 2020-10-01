use super::{Node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

pub struct BinaryNode {
    token_kind: TokenKind,
    span: TextSpan,
    left: Box<SyntaxNode>,
    right: Box<SyntaxNode>,
}

impl BinaryNode {
    pub fn new(token: Token, left: SyntaxNode, right: SyntaxNode) -> Self {
        Self {
            token_kind: token.kind,
            span: token.text_span,
            left: Box::new(left),
            right: Box::new(right),
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
