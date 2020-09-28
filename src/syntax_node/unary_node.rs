use super::Node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use std::fmt;

pub struct UnaryNode {
    token_kind: TokenKind,
    span: TextSpan,
    child: Box<dyn Node>,
}

impl UnaryNode {
    pub fn new(token: Token, child: Box<dyn Node>) -> Self {
        Self {
            token_kind: token.kind,
            span: token.text_span,
            child,
        }
    }
}

impl fmt::Display for UnaryNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} => {}", self.token_kind, self.child)
    }
}

impl Node for UnaryNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }
}
