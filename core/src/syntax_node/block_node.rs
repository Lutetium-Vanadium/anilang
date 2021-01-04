use super::{print_node, SyntaxNode};
use crate::text_span::TextSpan;
use crossterm::style;

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
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.block.len() {
            self.block[i]._prt(indent.clone(), i == self.block.len() - 1, stdout);
        }
    }

    pub(super) fn can_const_eval(&self) -> bool {
        self.block.iter().all(|n| n.can_const_eval())
    }
}

use std::fmt;
impl fmt::Display for BlockNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStatement")
    }
}
