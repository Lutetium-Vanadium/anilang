use super::{print_node, SyntaxNode};
use crate::tokens::Token;
use crossterm::style;
use source::TextSpan;

#[derive(Default, Debug, Clone)]
pub struct ListNode {
    pub span: TextSpan,
    pub elements: Vec<SyntaxNode>,
}

impl ListNode {
    pub fn new(open_bracket: &Token, elements: Vec<SyntaxNode>, close_bracket: &Token) -> Self {
        Self {
            elements,
            span: TextSpan::from_spans(&open_bracket.text_span, &close_bracket.text_span),
        }
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
impl fmt::Display for ListNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List")
    }
}
