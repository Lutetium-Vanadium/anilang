use crate::function::{native, Function, NativeFn};
use crate::value::{ErrorKind, Ref, Result, Value};
use std::rc::Rc;

impl Value {
    pub fn get_property(self, p: Ref<String>) -> Result<Value> {
        let err = |val, property| Err(ErrorKind::InvalidProperty { val, property });

        let property = p.borrow();

        match &self {
            Value::String(s) => match property.as_str() {
                "len" => Ok(Value::Int(s.borrow().len() as i64)),
                _ => err(self, Rc::clone(&p)),
            },
            Value::List(l) => match property.as_str() {
                "len" => Ok(Value::Int(l.borrow().len() as i64)),
                "push" => Ok(make_fn(self, native::push)),
                "pop" => Ok(make_fn(self, native::pop)),
                _ => err(self, Rc::clone(&p)),
            },
            Value::Object(o) => {
                if let Some(val) = o.borrow().get(property.as_str()) {
                    return Ok(val.clone());
                }

                err(self, Rc::clone(&p))
            }
            Value::Range(s, e) => match property.as_str() {
                "start" => Ok(Value::Int(*s)),
                "end" => Ok(Value::Int(*e)),
                _ => err(self, Rc::clone(&p)),
            },
            Value::Function(_) => match property.as_str() {
                "call" => Ok(self),
                _ => err(self, Rc::clone(&p)),
            },
            _ => unreachable!(),
        }
    }

    pub fn set_property(self, p: Ref<String>, value: Value) -> Result<Value> {
        let err_invalid = |val, property| Err(ErrorKind::InvalidProperty { val, property });
        let err_readonly = |val, property| Err(ErrorKind::ReadonlyProperty { val, property });

        let property = p.borrow();

        match &self {
            Value::String(_) => match property.as_str() {
                "len" => err_readonly(self, Rc::clone(&p)),
                _ => err_invalid(self, Rc::clone(&p)),
            },
            Value::List(_) => match property.as_str() {
                "len" | "push" | "pop" => err_readonly(self, Rc::clone(&p)),
                _ => err_invalid(self, Rc::clone(&p)),
            },
            Value::Object(o) => {
                drop(property);
                o.borrow_mut().insert(copy_str(p), value.clone());
                Ok(value)
            }
            Value::Range(..) => match property.as_str() {
                "start" | "end" => err_readonly(self, Rc::clone(&p)),
                _ => err_invalid(self, Rc::clone(&p)),
            },
            Value::Function(_) => match property.as_str() {
                "call" => err_readonly(self, Rc::clone(&p)),
                _ => err_invalid(self, Rc::clone(&p)),
            },
            _ => unreachable!(),
        }
    }
}

fn make_fn(this: Value, native_fn: NativeFn) -> Value {
    Value::Function(Rc::new(Function::native_fn(native_fn).with_this(this)))
}

fn copy_str(string: Ref<String>) -> String {
    Rc::try_unwrap(string)
        .map(std::cell::RefCell::into_inner)
        .unwrap_or_else(|string| string.borrow().as_str().to_owned())
}
