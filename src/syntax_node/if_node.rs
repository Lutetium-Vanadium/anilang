use super::{BlockNode, Node, SyntaxNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;

#[derive(Debug, Clone)]
pub struct IfNode {
    span: TextSpan,
    pub cond: Box<SyntaxNode>,
    pub if_block: BlockNode,
    pub else_block: Option<BlockNode>,
}

impl IfNode {
    pub fn new(
        if_token: Token,
        cond: SyntaxNode,
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
            cond: Box::new(cond),
            if_block,
            else_block,
        }
    }

    pub fn with_span(
        cond: Box<SyntaxNode>,
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
            crate::colour::BRIGHT_BLUE,
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
