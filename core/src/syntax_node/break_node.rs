use crate::text_span::TextSpan;
use std::fmt;

#[derive(Debug, Clone)]
pub struct BreakNode {
    pub span: TextSpan,
}

impl BreakNode {
    pub fn new(span: TextSpan) -> Self {
        Self { span }
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

impl fmt::Display for BreakNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BreakToken")
    }
}
