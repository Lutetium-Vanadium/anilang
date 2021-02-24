use super::SyntaxNode;
use crate::tokens::Token;
use crossterm::style;
use source::TextSpan;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ReturnNode {
    pub span: TextSpan,
    pub value: Option<Box<SyntaxNode>>,
}

impl ReturnNode {
    pub fn new(value: Option<Box<SyntaxNode>>, return_token: &Token) -> Self {
        let span = value
            .as_ref()
            .map(|node| TextSpan::from_spans(&return_token.text_span, node.span()))
            .unwrap_or_else(|| return_token.text_span.clone());

        Self { value, span }
    }

    pub(super) fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = super::print_node(style::Color::Green, &indent, self, is_last, stdout);
    }
}

impl fmt::Display for ReturnNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReturnToken")
    }
}
