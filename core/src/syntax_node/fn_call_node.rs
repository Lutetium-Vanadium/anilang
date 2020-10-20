use super::{print_node, SyntaxNode};
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;

#[derive(Debug, Clone)]
pub struct FnCallNode {
    pub span: TextSpan,
    pub ident: String,
    pub args: Vec<SyntaxNode>,
}

impl FnCallNode {
    pub fn new(ident: &Token, args: Vec<SyntaxNode>, end_paran: &Token, src: &SourceText) -> Self {
        Self {
            span: TextSpan::from_spans(&ident.text_span, &end_paran.text_span),
            ident: src[&ident.text_span].to_owned(),
            args,
        }
    }

    pub fn with_span(ident: String, args: Vec<SyntaxNode>, span: TextSpan) -> Self {
        Self { ident, args, span }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };
        for (i, arg) in self.args.iter().enumerate() {
            arg._prt(indent.clone(), i + 1 == self.args.len(), stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for FnCallNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnCall {}", self.ident)
    }
}
