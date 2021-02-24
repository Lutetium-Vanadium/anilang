use super::{print_node, SyntaxNode};
use crate::tokens::Token;
use crossterm::style;
use source::TextSpan;

#[derive(Debug, Clone)]
pub struct FnCallNode {
    pub span: TextSpan,
    pub child: Box<SyntaxNode>,
    pub args: Vec<SyntaxNode>,
}

impl FnCallNode {
    pub fn new(value: SyntaxNode, args: Vec<SyntaxNode>, end_paran: &Token) -> Self {
        Self {
            span: TextSpan::from_spans(value.span(), &end_paran.text_span),
            child: Box::new(value),
            args,
        }
    }

    pub fn with_span(child: Box<SyntaxNode>, args: Vec<SyntaxNode>, span: TextSpan) -> Self {
        Self { child, args, span }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };
        self.child._prt(indent.clone(), false, stdout);

        for (i, arg) in self.args.iter().enumerate() {
            arg._prt(indent.clone(), i + 1 == self.args.len(), stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for FnCallNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnCall")
    }
}
