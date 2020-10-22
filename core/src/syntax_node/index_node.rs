use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;
use std::fmt;

#[derive(Debug, Clone)]
pub struct IndexNode {
    pub span: TextSpan,
    pub child: Box<SyntaxNode>,
    pub index: Box<SyntaxNode>,
}

impl IndexNode {
    pub fn new(child: SyntaxNode, index: SyntaxNode, close_bracket: &Token) -> Self {
        Self {
            span: TextSpan::from_spans(child.span(), &close_bracket.text_span),
            child: Box::new(child),
            index: Box::new(index),
        }
    }

    pub fn from_span(child: SyntaxNode, index: SyntaxNode, span: TextSpan) -> Self {
        Self {
            span,
            child: Box::new(child),
            index: Box::new(index),
        }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        self.child._prt(indent.clone(), true, stdout);
        self.index._prt(indent, true, stdout);
    }
}

impl fmt::Display for IndexNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IndexOperator")
    }
}
