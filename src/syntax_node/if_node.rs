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

    pub fn with_span(
        cond: Box<dyn Node>,
        if_block: BlockNode,
        else_block: Option<BlockNode>,
        span: TextSpan,
    ) -> Self {
        Self {
            cond,
            if_block,
            else_block,
            span,
        }
    }
}

use std::fmt;
impl fmt::Display for IfNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IfKeyword")
    }
}

impl Node for IfNode {
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

        self.cond.prt(indent.clone(), false);
        match self.else_block {
            Some(ref else_block) => {
                self.if_block.prt(indent.clone(), false);
                else_block.prt(indent, true);
            }
            None => {
                self.if_block.prt(indent, true);
            }
        }
    }
}
