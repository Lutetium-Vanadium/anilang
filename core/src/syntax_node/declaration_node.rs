use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DeclarationNode {
    pub span: TextSpan,
    pub ident: Rc<str>,
    pub value: Box<SyntaxNode>,
}

impl DeclarationNode {
    pub fn new(declaration_token: &Token, ident: Rc<str>, value: SyntaxNode) -> Self {
        Self {
            ident,
            span: TextSpan::from_spans(&declaration_token.text_span, value.span()),
            value: Box::new(value),
        }
    }

    pub fn from_span(ident: Rc<str>, value: Box<SyntaxNode>, span: TextSpan) -> Self {
        Self { ident, value, span }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        self.value._prt(indent, true, stdout);
    }
}

use std::fmt;
impl fmt::Display for DeclarationNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeclarationOperator: {}", self.ident)
    }
}
