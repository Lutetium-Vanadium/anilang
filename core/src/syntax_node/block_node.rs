use super::SyntaxNode;
use crate::text_span::TextSpan;

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
    pub fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_BLUE,
            self,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.block.len() {
            self.block[i].prt(indent.clone(), i == self.block.len() - 1);
        }
    }
}

use std::fmt;
impl fmt::Display for BlockNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStatement")
    }
}
