use super::SyntaxNode;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crossterm::{queue, style};
use std::fmt;
use std::io::prelude::*;

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

        self.child._prt(indent, true, stdout);
    }
}

impl fmt::Display for UnaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.operator)
    }
}
