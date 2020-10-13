use super::SyntaxNode;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub struct AssignmentNode {
    pub span: TextSpan,
    pub ident: String,
    pub value: Box<SyntaxNode>,
}

impl AssignmentNode {
    pub fn new(ident_token: &Token, value: SyntaxNode, src: &SourceText) -> Self {
        Self {
            ident: src[&ident_token.text_span].to_owned(),
            span: TextSpan::from_spans(&ident_token.text_span, value.span()),
            value: Box::new(value),
        }
    }

    pub(super) fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{} => {}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_BLUE,
            self,
            self.ident,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        self.value.prt(indent, true);
    }
}

use std::fmt;
impl fmt::Display for AssignmentNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssignmentOperator")
    }
}
