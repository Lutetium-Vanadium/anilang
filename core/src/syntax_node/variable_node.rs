use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::{queue, style};
use std::io::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub span: TextSpan,
    pub ident: String,
}

impl VariableNode {
    pub fn new(token: &Token, src: &SourceText) -> Self {
        Self {
            ident: src[&token.text_span].to_owned(),
            span: token.text_span.clone(),
        }
    }

    pub fn raw(ident: String, span: TextSpan) -> Self {
        Self { ident, span }
    }

    pub(super) fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let marker = if is_last { "└── " } else { "├── " };

        let _ = queue!(
            stdout,
            style::SetForegroundColor(style::Color::Grey),
            style::Print(&indent),
            style::Print(marker),
            style::SetForegroundColor(style::Color::Green),
            style::Print(format!("{}\n", self)),
            style::ResetColor,
        );
    }
}

use std::fmt;
impl fmt::Display for VariableNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <IdentToken>", self.ident)
    }
}
