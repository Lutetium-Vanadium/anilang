use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crossterm::style;

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub span: TextSpan,
    pub ident: String,
}

impl VariableNode {
    pub fn new(span: TextSpan, src: &SourceText) -> Self {
        Self {
            ident: src[&span].to_owned(),
            span,
        }
    }

    pub fn raw(ident: String, span: TextSpan) -> Self {
        Self { ident, span }
    }

    pub(super) fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = super::print_node(style::Color::Green, &indent, self, is_last, stdout);
    }
}

use std::fmt;
impl fmt::Display for VariableNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <IdentToken>", self.ident)
    }
}
