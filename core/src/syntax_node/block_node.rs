use super::SyntaxNode;
use crate::text_span::TextSpan;
use crossterm::{queue, style};
use std::io::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct BlockNode {
    pub span: TextSpan,
    pub block: Vec<SyntaxNode>,
}

impl BlockNode {
    pub fn new(block: Vec<SyntaxNode>, span: TextSpan) -> Self {
        Self { span, block }
    }

    pub fn consume(self) -> (TextSpan, Vec<SyntaxNode>) {
        (self.span, self.block)
    }

    /// Must be completely pub for `BlockNode` only, because parser returns a `BlockNode` directly,
    /// and not a `SyntaxNode`
    pub fn prt(&self) {
        self._prt(String::new(), true, &mut std::io::stdout());
    }

    pub(crate) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let marker = if is_last { "└── " } else { "├── " };

        let _ = queue!(
            stdout,
            style::SetForegroundColor(style::Color::Grey),
            style::Print(&indent),
            style::Print(marker),
            style::SetForegroundColor(style::Color::Blue),
            style::Print(format!("{}\n", self)),
            style::ResetColor,
        );

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.block.len() {
            self.block[i]._prt(indent.clone(), i == self.block.len() - 1, stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for BlockNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStatement")
    }
}
