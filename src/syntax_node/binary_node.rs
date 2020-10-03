use super::{Node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct BinaryNode {
    span: TextSpan,
    pub operator: TokenKind,
    pub left: Box<SyntaxNode>,
    pub right: Box<SyntaxNode>,
}

impl BinaryNode {
    pub fn new(operator: Token, left: SyntaxNode, right: SyntaxNode) -> Self {
        Self {
            operator: operator.kind,
            span: operator.text_span,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn with_span(operator: Token, left: SyntaxNode, right: SyntaxNode, span: TextSpan) -> Self {
        Self {
            operator: operator.kind,
            span,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

use std::fmt;
impl fmt::Display for BinaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.operator)
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
            crate::colour::BRIGHT_BLUE,
            self,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        self.left.prt(indent.clone(), false);
        self.right.prt(indent, true);
    }
}
