use super::{print_node, SyntaxNode};
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;

#[derive(Debug, Clone)]
pub struct AssignmentNode {
    pub span: TextSpan,
    pub ident: String,
    /// For an assignment `<variable>[<index>] = <value>`, index refers to the <index>, and *not*
    /// a `IndexNode`
    pub indices: Option<Vec<SyntaxNode>>,
    pub value: Box<SyntaxNode>,
}

impl AssignmentNode {
    pub fn new(ident_token: &Token, value: SyntaxNode, src: &SourceText) -> Self {
        Self {
            ident: src[&ident_token.text_span].to_owned(),
            span: TextSpan::from_spans(&ident_token.text_span, value.span()),
            indices: None,
            value: Box::new(value),
        }
    }

    pub fn new_index(
        ident_token: &Token,
        indices: Vec<SyntaxNode>,
        value: SyntaxNode,
        src: &SourceText,
    ) -> Self {
        Self {
            ident: src[&ident_token.text_span].to_owned(),
            span: TextSpan::from_spans(&ident_token.text_span, value.span()),
            indices: Some(indices),
            value: Box::new(value),
        }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        if let Some(ref indices) = self.indices {
            for index in indices {
                index._prt(indent.clone(), false, stdout);
            }
        }

        self.value._prt(indent, true, stdout);
    }
}

use std::fmt;
impl fmt::Display for AssignmentNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssignmentOperator: {}", self.ident)
    }
}
