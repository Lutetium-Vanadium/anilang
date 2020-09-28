use super::{BlockNode, Node};
use crate::text_span::TextSpan;
use crate::tokens::Token;

pub struct IfNode {
    span: TextSpan,
    cond: Box<dyn Node>,
    if_block: BlockNode,
    else_block: Option<BlockNode>,
}

impl IfNode {
    pub fn new(
        if_token: Token,
        cond: Box<dyn Node>,
        if_block: BlockNode,
        else_block: Option<BlockNode>,
    ) -> Self {
        Self {
            span: TextSpan::from_spans(
                &if_token.text_span,
                match else_block {
                    Some(ref else_block) => else_block.span(),
                    None => if_block.span(),
                },
            ),
            cond,
            if_block,
            else_block,
        }
    }
}

use std::fmt;
impl fmt::Display for IfNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if {} then \n{}", self.cond, self.if_block)?;
        if let Some(ref else_block) = self.else_block {
            write!(f, "else\n{}", else_block)?;
        }
        Ok(())
    }
}

impl Node for IfNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }
}
