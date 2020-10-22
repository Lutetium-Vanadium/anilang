use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crossterm::style;
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
            span: TextSpan::from_spans(&operator.text_span, child.span()),
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

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        self.child._prt(indent, true, stdout);
    }
}

impl fmt::Display for UnaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.operator)
    }
}
