use super::Node;
use crate::text_span::TextSpan;

#[derive(Default)]
pub struct BlockNode {
    span: TextSpan,
    block: Vec<Box<dyn Node>>,
}

impl BlockNode {
    pub fn new(block: Vec<Box<dyn Node>>, span: TextSpan) -> Self {
        Self { span, block }
    }

    pub fn consume(self) -> (TextSpan, Vec<Box<dyn Node>>) {
        (self.span, self.block)
    }
}

use std::fmt;
impl fmt::Display for BlockNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStatement")
    }
}

impl Node for BlockNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }

    fn prt(&self, mut indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_MAGENTA,
            self,
            crate::colour::RESET,
        );

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.block.len() {
            self.block[i].prt(indent.clone(), i == self.block.len() - 1);
        }
    }
}
