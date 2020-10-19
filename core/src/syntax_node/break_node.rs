use crate::text_span::TextSpan;
use crossterm::style;
use std::fmt;

#[derive(Debug, Clone)]
pub struct BreakNode {
    pub span: TextSpan,
}

impl BreakNode {
    pub fn new(span: TextSpan) -> Self {
        Self { span }
    }

    pub(super) fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = super::print_node(style::Color::Green, &indent, self, is_last, stdout);
    }
}

impl fmt::Display for BreakNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BreakToken")
    }
}
