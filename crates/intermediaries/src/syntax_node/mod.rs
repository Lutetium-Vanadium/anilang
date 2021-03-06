mod assignment_node;
mod binary_node;
mod block_node;
mod break_node;
mod declaration_node;
mod fn_call_node;
mod fn_declaration_node;
mod if_node;
mod index_node;
mod interface_node;
mod list_node;
mod loop_node;
mod object_node;
mod return_node;
mod unary_node;
mod variable_node;

// Public for access to `Parse` and `ErrorKind`
pub(crate) mod literal_node;

use crossterm::{queue, style};
use source::TextSpan;
use std::io::prelude::*;

pub mod node {
    pub use super::assignment_node::AssignmentNode;
    pub use super::binary_node::BinaryNode;
    pub use super::block_node::BlockNode;
    pub use super::break_node::BreakNode;
    pub use super::declaration_node::DeclarationNode;
    pub use super::fn_call_node::FnCallNode;
    pub use super::fn_declaration_node::FnDeclarationNode;
    pub use super::if_node::IfNode;
    pub use super::index_node::IndexNode;
    pub use super::interface_node::InterfaceNode;
    pub use super::list_node::ListNode;
    pub use super::literal_node::LiteralNode;
    pub use super::loop_node::LoopNode;
    pub use super::object_node::ObjectNode;
    pub use super::return_node::ReturnNode;
    pub use super::unary_node::UnaryNode;
    pub use super::variable_node::VariableNode;
}

use node::*;

#[inline]
fn print_node<T: std::fmt::Display>(
    colour: style::Color,
    indent: &str,
    node: &T,
    is_last: bool,
    stdout: &mut std::io::Stdout,
) -> crossterm::Result<()> {
    let marker = if is_last { "└── " } else { "├── " };

    queue!(
        stdout,
        style::SetForegroundColor(style::Color::Grey),
        style::Print(indent),
        style::Print(marker),
        style::SetForegroundColor(colour),
        style::Print(format!("{}\n", node)),
        style::ResetColor,
    )
}

#[derive(Debug, Clone)]
pub enum SyntaxNode {
    AssignmentNode(AssignmentNode),
    BinaryNode(BinaryNode),
    BlockNode(BlockNode),
    BreakNode(BreakNode),
    DeclarationNode(DeclarationNode),
    FnCallNode(FnCallNode),
    FnDeclarationNode(FnDeclarationNode),
    IfNode(IfNode),
    IndexNode(IndexNode),
    InterfaceNode(InterfaceNode),
    ListNode(ListNode),
    LiteralNode(LiteralNode),
    LoopNode(LoopNode),
    ObjectNode(ObjectNode),
    ReturnNode(ReturnNode),
    UnaryNode(UnaryNode),
    VariableNode(VariableNode),
    BadNode(TextSpan),
}

use std::fmt;
impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxNode::AssignmentNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BinaryNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BlockNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BreakNode(ref n) => write!(f, "{}", n),
            SyntaxNode::DeclarationNode(ref n) => write!(f, "{}", n),
            SyntaxNode::FnCallNode(ref n) => write!(f, "{}", n),
            SyntaxNode::FnDeclarationNode(ref n) => write!(f, "{}", n),
            SyntaxNode::IfNode(ref n) => write!(f, "{}", n),
            SyntaxNode::IndexNode(ref n) => write!(f, "{}", n),
            SyntaxNode::InterfaceNode(ref n) => write!(f, "{}", n),
            SyntaxNode::ListNode(ref n) => write!(f, "{}", n),
            SyntaxNode::LiteralNode(ref n) => write!(f, "{}", n),
            SyntaxNode::LoopNode(ref n) => write!(f, "{}", n),
            SyntaxNode::ObjectNode(ref n) => write!(f, "{}", n),
            SyntaxNode::ReturnNode(ref n) => write!(f, "{}", n),
            SyntaxNode::UnaryNode(ref n) => write!(f, "{}", n),
            SyntaxNode::VariableNode(ref n) => write!(f, "{}", n),
            SyntaxNode::BadNode(_) => write!(f, "BadNode"),
        }
    }
}

impl SyntaxNode {
    pub fn span(&self) -> &TextSpan {
        match self {
            SyntaxNode::AssignmentNode(ref n) => &n.span,
            SyntaxNode::BinaryNode(ref n) => &n.span,
            SyntaxNode::BlockNode(ref n) => &n.span,
            SyntaxNode::BreakNode(ref n) => &n.span,
            SyntaxNode::DeclarationNode(ref n) => &n.span,
            SyntaxNode::FnCallNode(ref n) => &n.span,
            SyntaxNode::FnDeclarationNode(ref n) => &n.span,
            SyntaxNode::IfNode(ref n) => &n.span,
            SyntaxNode::IndexNode(ref n) => &n.span,
            SyntaxNode::InterfaceNode(ref n) => &n.span,
            SyntaxNode::ListNode(ref n) => &n.span,
            SyntaxNode::LiteralNode(ref n) => &n.span,
            SyntaxNode::LoopNode(ref n) => &n.span,
            SyntaxNode::ObjectNode(ref n) => &n.span,
            SyntaxNode::ReturnNode(ref n) => &n.span,
            SyntaxNode::UnaryNode(ref n) => &n.span,
            SyntaxNode::VariableNode(ref n) => &n.span,
            SyntaxNode::BadNode(ref span) => span,
        }
    }

    pub fn prt(&self) {
        let mut stdout = std::io::stdout();
        self._prt(String::new(), true, &mut stdout);
    }

    fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        match self {
            SyntaxNode::AssignmentNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::BinaryNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::BlockNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::BreakNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::DeclarationNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::FnCallNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::FnDeclarationNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::IfNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::IndexNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::InterfaceNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::ListNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::LiteralNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::LoopNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::ObjectNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::ReturnNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::UnaryNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::VariableNode(ref n) => n._prt(indent, is_last, stdout),
            SyntaxNode::BadNode(_) => {
                let _ = print_node(style::Color::Red, &indent, self, is_last, stdout);
            }
        }
    }

    pub fn can_const_eval(&self) -> bool {
        match self {
            SyntaxNode::BinaryNode(ref n) => n.left.can_const_eval() && n.right.can_const_eval(),
            SyntaxNode::BlockNode(ref n) => n.can_const_eval(),
            SyntaxNode::IfNode(ref n) => {
                n.cond.can_const_eval()
                    && n.if_block.can_const_eval()
                    && if let Some(ref block) = n.else_block {
                        block.can_const_eval()
                    } else {
                        true
                    }
            }
            SyntaxNode::IndexNode(ref n) => n.child.can_const_eval() && n.index.can_const_eval(),
            SyntaxNode::ListNode(ref n) => n.elements.iter().all(|n| n.can_const_eval()),
            SyntaxNode::ObjectNode(ref n) => n.elements.iter().all(|n| n.can_const_eval()),
            SyntaxNode::UnaryNode(ref n) => n.child.can_const_eval(),
            SyntaxNode::LiteralNode(_) => true,

            SyntaxNode::AssignmentNode(_) => false,
            SyntaxNode::BreakNode(_) => false,
            SyntaxNode::DeclarationNode(_) => false,
            SyntaxNode::FnDeclarationNode(_) => false,
            SyntaxNode::FnCallNode(_) => false,
            SyntaxNode::InterfaceNode(_) => false,
            SyntaxNode::LoopNode(_) => false,
            SyntaxNode::ReturnNode(_) => false,
            SyntaxNode::VariableNode(_) => false,
            SyntaxNode::BadNode(_) => false,
        }
    }
}
