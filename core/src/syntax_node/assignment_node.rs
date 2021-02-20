use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct AssignmentNode {
    pub span: TextSpan,
    pub ident: Rc<str>,
    /// For an assignment `<variable>[<index>] = <value>`, index refers to the <index>, and *not*
    /// a `IndexNode`
    pub indices: Option<Vec<SyntaxNode>>,
    pub value: Box<SyntaxNode>,
}

impl AssignmentNode {
    pub fn new(
        ident: Rc<str>,
        ident_token: &Token,
        indices: Option<Vec<SyntaxNode>>,
        value: SyntaxNode,
    ) -> Self {
        Self {
            ident,
            span: TextSpan::from_spans(&ident_token.text_span, value.span()),
            indices,
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
