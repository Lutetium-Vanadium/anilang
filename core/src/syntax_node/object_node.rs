use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;

#[derive(Default, Debug, Clone)]
pub struct ObjectNode {
    pub span: TextSpan,
    // Not stored as a HashMap<SyntaxNode, SyntaxNode>, but instead as a Vec<SyntaxNode> where the
    // (2*n)th element is the key for the (2*n + 1)th element for int n >= 0. Also the keys are
    // stored as SyntaxNode since they don't have to be direct Strings, but can also be expressions
    // that evaluate to strings
    pub elements: Vec<SyntaxNode>,
}

impl ObjectNode {
    pub fn new(open_brace: &Token, elements: Vec<SyntaxNode>, close_brace: &Token) -> Self {
        Self {
            elements,
            span: TextSpan::from_spans(&open_brace.text_span, &close_brace.text_span),
        }
    }

    pub fn from_span(elements: Vec<SyntaxNode>, span: TextSpan) -> Self {
        Self { elements, span }
    }

    pub(crate) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.elements.len() {
            self.elements[i]._prt(indent.clone(), i == self.elements.len() - 1, stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for ObjectNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}
