use crate::bytecode::Bytecode;
use gc::Mark;
use std::rc::Rc;

mod anilang_fn;
mod native_fn;

pub mod native {
    pub use super::native_fn::*;
}

pub use anilang_fn::AnilangFn;
pub use native_fn::NativeFn;

pub struct Function {
    fn_type: FunctionType,
    this: Option<Value>,
}

impl Function {
    pub fn new(fn_type: FunctionType) -> Self {
        Self {
            fn_type,
            this: None,
        }
    }

    pub fn anilang_fn(args: Vec<Rc<str>>, body: Bytecode) -> Self {
        Self {
            fn_type: FunctionType::AnilangFn(AnilangFn::new(args, body)),
            this: None,
        }
    }

    pub fn native_fn(native_fn: NativeFn) -> Self {
        Self {
            fn_type: FunctionType::NativeFn(native_fn),
            this: None,
        }
    }

    pub fn as_anilang_fn(&self) -> Option<&AnilangFn> {
        if let FunctionType::AnilangFn(ref f) = self.fn_type {
            Some(f)
        } else {
            None
        }
    }

    pub fn as_native_fn(&self) -> Option<&NativeFn> {
        if let FunctionType::NativeFn(ref f) = self.fn_type {
            Some(f)
        } else {
            None
        }
    }

    pub fn this(&self) -> Option<&Value> {
        self.this.as_ref()
    }

    pub fn with_this(mut self, this: Value) -> Self {
        self.this = Some(this);
        self
    }
}

unsafe impl Mark for Function {
    unsafe fn mark(&self) {
        // NOTE: This implementation does not call mark on all contained `Gc` values, and so
        // violates the safety requirement given for the mark function. The `Gc`s not marked are
        // within the function body. However, this is safe since those `Gc`s will be considered
        // unreachable as they are not updated in update_reachable.
        self.this.mark();
    }

    unsafe fn update_reachable(&self) {
        // NOTE: This implementation does not call update_reachable on all contained `Gc` values,
        // and so violates the safety requirement given for the update_reachable function. The `Gc`s
        // not updated are within the function body. However, this is safe since those `Gc`s will be
        // considered unreachable and will not be cyclic (since cycles can only be created within
        // the Evaluator), so no memory leaks will occur.
        self.this.update_reachable();
    }
}

/// Representation of pointer to function which can be executed
#[derive(Clone)]
pub enum FunctionType {
    /// A function declared within anilang. It contains bytecode and args to be executed.
    AnilangFn(AnilangFn),
    /// A pointer to a rust function that performs some operation on variable amount of args.
    ///
    /// The rust function must be of type
    NativeFn(NativeFn),
}

use std::fmt;
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fn_type)?;
        if let Some(ref this) = self.this {
            write!(f, " on {}", this)?;
        }
        Ok(())
    }
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionType::AnilangFn(func) => write!(f, "{}", func),
            FunctionType::NativeFn(_) => write!(f, "native function"),
        }
    }
}

use crate::value::Value;
impl From<FunctionType> for Value {
    fn from(fn_type: FunctionType) -> Value {
        Value::Function(Rc::new(Function::new(fn_type)))
    }
}

impl From<AnilangFn> for FunctionType {
    fn from(anilang_fn: AnilangFn) -> FunctionType {
        FunctionType::AnilangFn(anilang_fn)
    }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionType::AnilangFn(func) => write!(f, "{:?}", func),
            FunctionType::NativeFn(_) => write!(f, "Native Function"),
        }
    }
}
