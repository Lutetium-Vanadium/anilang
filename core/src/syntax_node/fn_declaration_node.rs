use super::{print_node, BlockNode};
use crate::text_span::TextSpan;
use crate::tokens::Token;
use crossterm::style;

#[derive(Debug, Clone)]
pub struct FnDeclarationNode {
    pub span: TextSpan,
    pub ident: String,
    pub args: Vec<String>,
    pub block: BlockNode,
}

impl FnDeclarationNode {
    pub fn new(fn_token: &Token, ident: String, args: Vec<String>, block: BlockNode) -> Self {
        Self {
            span: TextSpan::from_spans(&fn_token.text_span, &block.span),
            ident,
            args,
            block,
        }
    }

    pub fn with_span(ident: String, args: Vec<String>, block: BlockNode, span: TextSpan) -> Self {
        Self {
            ident,
            args,
            block,
            span,
        }
    }

    pub(super) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        println!("{}├── [", indent);
        for arg in self.args.iter() {
            println!("{}│  {}", indent, arg);
        }
        println!("{}│ ]", indent);
        self.block._prt(indent, true, stdout);
    }
}

use std::fmt;
impl fmt::Display for FnDeclarationNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnKeyword -> {}", self.ident)
    }
}
