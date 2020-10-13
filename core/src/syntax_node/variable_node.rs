use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;

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

    pub(super) fn prt(&self, indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_GREEN,
            self,
            crate::colour::RESET,
        );
    }
}

use std::fmt;
impl fmt::Display for VariableNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <IdentToken>", self.ident)
    }
}
