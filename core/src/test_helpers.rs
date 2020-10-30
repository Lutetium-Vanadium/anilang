use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

pub fn i(i: i64) -> Value {
    Value::Int(i)
}

pub fn f(f: f64) -> Value {
    Value::Float(f)
}

pub fn b(b: bool) -> Value {
    Value::Bool(b)
}

pub fn s(s: &str) -> Value {
    Value::String(Rc::new(RefCell::new(s.to_owned())))
}

pub fn l(l: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(l)))
}

pub fn r(s: i64, e: i64) -> Value {
    Value::Range(s, e)
}

pub fn func() -> Value {
    Value::Function(Rc::new(Default::default()))
}

pub fn n() -> Value {
    Value::Null
}
