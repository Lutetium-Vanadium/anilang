use crate::text_span::TextSpan;
use crossterm::{queue, style};
use std::fmt;
use std::io::prelude::*;

#[derive(Debug, Clone)]
pub struct BreakNode {
    pub span: TextSpan,
}

impl BreakNode {
    pub fn new(span: TextSpan) -> Self {
        Self { span }
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

impl fmt::Display for BreakNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BreakToken")
    }
}
