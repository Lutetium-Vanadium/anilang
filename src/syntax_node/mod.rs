mod assignment_node;
mod binary_node;
mod block_node;
mod break_node;
mod if_node;
mod literal_node;
mod loop_node;
mod unary_node;

use crate::text_span::{self, TextSpan};

pub use assignment_node::*;
pub use binary_node::*;
pub use block_node::*;
pub use break_node::*;
pub use if_node::*;
pub use literal_node::*;
pub use loop_node::*;
pub use unary_node::*;

pub trait Node: std::fmt::Display {
    fn span(&self) -> &TextSpan;
    fn prt(&self, indent: String, is_last: bool);
}

pub struct BadNode();

use std::fmt;
impl fmt::Display for BadNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BadNode")
    }
}

impl Node for BadNode {
    fn span(&self) -> &TextSpan {
        &text_span::DEFAULT
    }

    fn prt(&self, indent: String, is_last: bool) {
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
