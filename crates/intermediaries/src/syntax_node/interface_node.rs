use super::{print_node, SyntaxNode};
use crate::tokens::Token;
use crossterm::{queue, style};
use source::TextSpan;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct InterfaceNode {
    pub ident: Rc<str>,
    pub span: TextSpan,
    /// The default values present in the object produced from interface.
    ///
    /// They are of the following kind.
    /// <ident> = <statement>
    /// ^^^^^^^^^^^^^^^^^^^^^-- Declare regular properties which have some particular value
    ///
    /// fn <ident>(<args>...) { <-----------.
    ///     ...                             | Declare functions which exist on the object, or can
    /// }                                   | be used statically from the interface
    /// ^-----------------------------------'
    ///
    /// <interface-name>(<args>...) { <-----.
    ///     ...                             | Special function which acts as a constructor
    /// }                                   |
    /// ^-----------------------------------'
    pub values: Vec<(String, SyntaxNode)>,
}

impl InterfaceNode {
    pub fn new(
        interface_token: &Token,
        ident: Rc<str>,
        values: Vec<(String, SyntaxNode)>,
        close_brace: &Token,
    ) -> Self {
        Self {
            ident,
            values,
            span: TextSpan::from_spans(&interface_token.text_span, &close_brace.text_span),
        }
    }

    pub(crate) fn _prt(&self, mut indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = print_node(style::Color::Blue, &indent, self, is_last, stdout);

        indent += if is_last { "   " } else { "│  " };

        for i in 0..self.values.len() {
            let _ = queue!(
                stdout,
                style::SetForegroundColor(style::Color::Grey),
                style::Print(&indent),
                style::Print("├── "),
                style::SetForegroundColor(style::Color::Yellow),
                style::Print(&self.values[i].0),
                style::ResetColor,
                style::Print('\n'),
            );
            self.values[i]
                .1
                ._prt(indent.clone(), i == self.values.len() - 1, stdout);
        }
    }
}

use std::fmt;
impl fmt::Display for InterfaceNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interface")
    }
}
