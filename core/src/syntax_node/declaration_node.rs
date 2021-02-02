use super::{print_node, SyntaxNode};
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;

#[derive(Debug, Clone)]
pub struct DeclarationNode {
    pub span: TextSpan,
    pub ident: String,
    pub value: Box<SyntaxNode>,
}

impl DeclarationNode {
    pub fn new(
        declaration_token: &Token,
        ident_token: &Token,
        value: SyntaxNode,
        src: &SourceText,
    ) -> Self {
        Self {
            ident: src[&ident_token.text_span].to_owned(),
            span: TextSpan::from_spans(&declaration_token.text_span, value.span()),
            value: Box::new(value),
        }
    }

    pub fn from_span(ident: String, value: Box<SyntaxNode>, span: TextSpan) -> Self {
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
