use crate::bytecode::Bytecode;

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Vec<String>,
    pub body: Bytecode,
}

impl Function {
    pub fn new(args: Vec<String>, body: Bytecode) -> Self {
        Self { args, body }
    }
}

use std::fmt;
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.is_empty() {
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
            body: Default::default(),
        }
    }
}
