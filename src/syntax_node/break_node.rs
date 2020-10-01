use super::Node;
use crate::text_span::TextSpan;
use std::fmt;

pub struct BreakNode {
    span: TextSpan,
}

impl BreakNode {
    pub fn new(span: TextSpan) -> Self {
        Self { span }
    }
}

impl fmt::Display for BreakNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BreakToken")
    }
}

impl Node for BreakNode {
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
            crate::colour::BRIGHT_YELLOW,
            self,
            crate::colour::RESET,
        );
    }
}
