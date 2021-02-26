use super::{print_node, SyntaxNode};
use crate::tokens::{Token, TokenKind};
use crossterm::style;
use source::TextSpan;

#[derive(Debug, Clone)]
pub struct BinaryNode {
    pub span: TextSpan,
    pub operator: TokenKind,
    pub left: Box<SyntaxNode>,
    pub right: Box<SyntaxNode>,
}

impl BinaryNode {
    pub fn new(operator: &Token, left: SyntaxNode, right: SyntaxNode) -> Self {
        Self {
            operator: operator.kind.clone(),
            span: TextSpan::from_spans(left.span(), right.span()),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn with_span(
        operator: TokenKind,
        left: SyntaxNode,
        right: SyntaxNode,
        span: TextSpan,
    ) -> Self {
        Self {
            operator,
            span,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        self.left._prt(indent.clone(), false, stdout);
        self.right._prt(indent, true, stdout);
    }
}

use std::fmt;
impl fmt::Display for BinaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.operator)
    }
}
