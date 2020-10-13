use super::SyntaxNode;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use std::fmt;

#[derive(Debug, Clone)]
pub struct UnaryNode {
    pub span: TextSpan,
    pub operator: TokenKind,
    pub child: Box<SyntaxNode>,
}

impl UnaryNode {
    pub fn new(operator: &Token, child: SyntaxNode) -> Self {
        Self {
            operator: operator.kind.clone(),
            span: operator.text_span.clone(),
            child: Box::new(child),
        }
    }

    pub fn from_span(operator: TokenKind, child: SyntaxNode, span: TextSpan) -> Self {
        Self {
            operator,
            span,
            child: Box::new(child),
        }
    }

    pub(super) fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_BLUE,
            self,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        self.child.prt(indent, true);
    }
}

impl fmt::Display for UnaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.operator)
    }
}
