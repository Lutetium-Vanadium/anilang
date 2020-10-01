use super::{Node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use std::fmt;

pub struct UnaryNode {
    token_kind: TokenKind,
    span: TextSpan,
    child: Box<SyntaxNode>,
}

impl UnaryNode {
    pub fn new(token: Token, child: SyntaxNode) -> Self {
        Self {
            token_kind: token.kind,
            span: token.text_span,
            child: Box::new(child),
        }
    }
}

impl fmt::Display for UnaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.token_kind)
    }
}

impl Node for UnaryNode {
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

        self.child.prt(indent, true);
    }
}
