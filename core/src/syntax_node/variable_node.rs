use crate::text_span::TextSpan;
use crossterm::style;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub span: TextSpan,
    pub ident: Rc<str>,
}

impl VariableNode {
    pub fn new(ident: Rc<str>, span: TextSpan) -> Self {
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
