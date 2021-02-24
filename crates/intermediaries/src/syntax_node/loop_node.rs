use super::{print_node, BlockNode, BreakNode, IfNode, SyntaxNode, UnaryNode};
use crate::tokens::{Token, TokenKind};
use crossterm::style;
use source::TextSpan;

#[derive(Default, Debug, Clone)]
pub struct LoopNode {
    pub span: TextSpan,
    pub block: Vec<SyntaxNode>,
}

impl LoopNode {
    pub fn new(token: &Token, block: BlockNode) -> Self {
        let (block_span, block) = block.consume();
        Self {
            span: TextSpan::from_spans(&token.text_span, &block_span),
            block,
        }
    }

    pub fn construct_while(token: &Token, cond: SyntaxNode, block: BlockNode) -> Self {
        let (block_span, mut block) = block.consume();
        let span = cond.span().clone();

        let cond = SyntaxNode::UnaryNode(UnaryNode::from_span(
            TokenKind::NotOperator,
            cond,
            span.clone(),
        ));

        block.insert(
            0,
            SyntaxNode::IfNode(IfNode::with_span(
                Box::new(cond),
                BlockNode::new(
                    vec![SyntaxNode::BreakNode(BreakNode::new(span.clone()))],
                    span.clone(),
                ),
                None,
                span,
            )),
        );

        Self {
            span: TextSpan::from_spans(&token.text_span, &block_span),
            block,
        }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.block.len() {
            self.block[i]._prt(indent.clone(), i == self.block.len() - 1, stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for LoopNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LoopStatement")
    }
}
