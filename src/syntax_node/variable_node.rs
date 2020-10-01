use super::Node;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub ident: String,
    span: TextSpan,
}

impl VariableNode {
    pub fn new(token: Token, src: &SourceText) -> Self {
        Self {
            ident: src[&token.text_span].to_owned(),
            span: token.text_span,
        }
    }
}

use std::fmt;
impl fmt::Display for VariableNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <IdentToken>", self.ident)
    }
}

impl Node for VariableNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }

    fn prt(&self, indent: String, is_last: bool) {
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
