use super::Node;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;

pub struct AssignmentNode {
    ident: String,
    span: TextSpan,
    value: Box<dyn Node>,
}

impl AssignmentNode {
    pub fn new(ident_token: Token, value: Box<dyn Node>, src: &SourceText) -> Self {
        Self {
            ident: src[&ident_token.text_span].to_owned(),
            span: TextSpan::from_spans(&ident_token.text_span, value.span()),
            value,
        }
    }
}

use std::fmt;
impl fmt::Display for AssignmentNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssignmentOperator")
    }
}

impl Node for AssignmentNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }

    fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{} => {}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_MAGENTA,
            self,
            self.ident,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        self.value.prt(indent, true);
    }
}
