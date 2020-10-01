mod assignment_node;
mod binary_node;
mod block_node;
mod break_node;
mod if_node;
mod literal_node;
mod loop_node;
mod unary_node;
mod variable_node;

use crate::text_span::{self, TextSpan};

pub use assignment_node::*;
pub use binary_node::*;
pub use block_node::*;
pub use break_node::*;
pub use if_node::*;
pub use literal_node::*;
pub use loop_node::*;
pub use unary_node::*;
pub use variable_node::*;

pub trait Node: std::fmt::Display {
    fn span(&self) -> &TextSpan;
    fn prt(&self, indent: String, is_last: bool);
}

pub enum SyntaxNode {
    AssignmentNode(assignment_node::AssignmentNode),
    BinaryNode(binary_node::BinaryNode),
    BlockNode(block_node::BlockNode),
    BreakNode(break_node::BreakNode),
    IfNode(if_node::IfNode),
    LiteralNode(literal_node::LiteralNode),
    LoopNode(loop_node::LoopNode),
    UnaryNode(unary_node::UnaryNode),
    VariableNode(variable_node::VariableNode),
    BadNode,
}

use std::fmt;
impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxNode::AssignmentNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BinaryNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BlockNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BreakNode(ref n) => write!(f, "{}", n),
            SyntaxNode::IfNode(ref n) => write!(f, "{}", n),
            SyntaxNode::LiteralNode(ref n) => write!(f, "{}", n),
            SyntaxNode::LoopNode(ref n) => write!(f, "{}", n),
            SyntaxNode::UnaryNode(ref n) => write!(f, "{}", n),
            SyntaxNode::VariableNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BadNode => write!(f, "BadNode"),
        }
    }
}

impl SyntaxNode {
    fn span(&self) -> &TextSpan {
        match self {
            SyntaxNode::AssignmentNode(ref n) => n.span(),
            SyntaxNode::BinaryNode(ref n) => n.span(),
            SyntaxNode::BlockNode(ref n) => n.span(),
            SyntaxNode::BreakNode(ref n) => n.span(),
            SyntaxNode::IfNode(ref n) => n.span(),
            SyntaxNode::LiteralNode(ref n) => n.span(),
            SyntaxNode::LoopNode(ref n) => n.span(),
            SyntaxNode::UnaryNode(ref n) => n.span(),
            SyntaxNode::VariableNode(ref n) => n.span(),
            SyntaxNode::BadNode => &text_span::DEFAULT,
        }
    }

    fn prt(&self, indent: String, is_last: bool) {
        match self {
            SyntaxNode::AssignmentNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::BinaryNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::BlockNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::BreakNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::IfNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::LiteralNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::LoopNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::UnaryNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::VariableNode(ref n) => n.prt(indent, is_last),
            SyntaxNode::BadNode => {
                let marker = if is_last { "└──" } else { "├──" };

                println!(
                    "{}{}{} {}{}{}",
                    crate::colour::LIGHT_GRAY,
                    indent,
                    marker,
                    crate::colour::BRIGHT_YELLOW,
                    self,
                    crate::colour::RESET,
                );
            }
        }
    }
}
