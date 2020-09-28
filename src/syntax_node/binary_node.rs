use super::Node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

pub struct BinaryNode {
    token_kind: TokenKind,
    span: TextSpan,
    left: Box<dyn Node>,
    right: Box<dyn Node>,
}

impl BinaryNode {
    pub fn new(token: Token, left: Box<dyn Node>, right: Box<dyn Node>) -> Self {
        Self {
            token_kind: token.kind,
            span: token.text_span,
            left,
            right,
        }
    }
}

use std::fmt;
impl fmt::Display for BinaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} => {}, {}", self.token_kind, self.left, self.right)
    }
}

impl Node for BinaryNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }
}
