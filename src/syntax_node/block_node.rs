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
}

use std::fmt;
impl fmt::Display for BlockNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.block.iter() {
            writeln!(f, "\t{}", node)?;
        }
        Ok(())
    }
}

impl Node for BlockNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }
}
