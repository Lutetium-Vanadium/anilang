use super::{print_node, BlockNode, SyntaxNode};
use crate::tokens::Token;
use crossterm::style;
use source::TextSpan;

#[derive(Debug, Clone)]
pub struct IfNode {
    pub span: TextSpan,
    pub cond: Box<SyntaxNode>,
    pub if_block: BlockNode,
    pub else_block: Option<BlockNode>,
}

impl IfNode {
    pub fn new(
        if_token: &Token,
        cond: SyntaxNode,
        if_block: BlockNode,
        else_block: Option<BlockNode>,
    ) -> Self {
        Self {
            span: TextSpan::from_spans(
                &if_token.text_span,
                match else_block {
                    Some(ref else_block) => &else_block.span,
                    None => &if_block.span,
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

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        self.cond._prt(indent.clone(), false, stdout);
        match self.else_block {
            Some(ref else_block) => {
                self.if_block._prt(indent.clone(), false, stdout);
                else_block._prt(indent, true, stdout);
            }
            None => {
                self.if_block._prt(indent, true, stdout);
            }
        }
    }
}

use std::fmt;
impl fmt::Display for IfNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IfKeyword")
    }
}
