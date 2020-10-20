use crate::syntax_node::BlockNode;

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub body: BlockNode,
}

impl Function {
    pub fn new(args: Vec<String>, body: BlockNode) -> Self {
        Self { args, body }
    }
}

use std::fmt;
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.len() == 0 {
            write!(f, "fn ()")
        } else {
            let mut iter = self.args.iter();
            write!(f, "fn ({}", iter.next().unwrap())?;
            for arg in iter {
                write!(f, ", {}", arg)?;
            }
            write!(f, ")")
        }
    }
}

use crate::value::Value;
impl From<Function> for Value {
    fn from(f: Function) -> Value {
        Value::Function(std::rc::Rc::new(f))
    }
}

impl Default for Function {
    fn default() -> Self {
        Self {
            args: Vec::new(),
            body: BlockNode::new(Vec::new(), Default::default()),
        }
    }
}
