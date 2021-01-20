use crate::bytecode::Bytecode;
use std::rc::Rc;

mod anilang_fn;
pub(crate) mod native_fn;

pub use anilang_fn::AnilangFn;
pub use native_fn::NativeFn;

/// Representation of pointer to function which can be executed
#[derive(Clone)]
pub enum Function {
    /// A function declared within anilang. It contains bytecode and args to be executed.
    AnilangFn(Rc<anilang_fn::AnilangFn>),
    /// A pointer to a rust function that performs some operation on variable amount of args.
    ///
    /// The rust function must be of type
    NativeFn(native_fn::NativeFn),
}

impl Function {
    pub fn new(args: Vec<String>, body: Bytecode) -> Self {
        Self::AnilangFn(Rc::new(AnilangFn::new(args, body)))
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        match (self, other) {
            (Function::AnilangFn(f1), Function::AnilangFn(f2)) => Rc::ptr_eq(f1, f2),
            // Not a great solution, but the only one I found to equate if 2 functions are the same
            //
            // FIXME: Hacky and possibly erroneous way to check if functions are the same
            (Function::NativeFn(f1), Function::NativeFn(f2)) => *f1 as usize == *f2 as usize,
            _ => false,
        }
    }
}

use std::fmt;
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::AnilangFn(func) => write!(f, "{}", func),
            Function::NativeFn(_) => write!(f, "Native function"),
        }
    }
}

use super::Value;
impl From<Function> for Value {
    fn from(f: Function) -> Value {
        Value::Function(f)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::AnilangFn(func) => write!(f, "{:?}", func),
            Function::NativeFn(_) => write!(f, "Native Function"),
        }
    }
}
