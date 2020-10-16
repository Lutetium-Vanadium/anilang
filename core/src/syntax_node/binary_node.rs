use super::SyntaxNode;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crossterm::{queue, style};
use std::io::prelude::*;

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
            span: operator.text_span.clone(),
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
        let marker = if is_last { "└── " } else { "├── " };

        let _ = queue!(
            stdout,
            style::SetForegroundColor(style::Color::Grey),
            style::Print(&indent),
            style::Print(marker),
            style::SetForegroundColor(style::Color::Blue),
            style::Print(format!("{}\n", self)),
            style::ResetColor,
        );

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
