use crate::value::{Object, Value};
use gc::Gc;
use std::cell::RefCell;

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
    Value::String(Gc::new(RefCell::new(s.to_owned())))
}

pub fn l(l: Vec<Value>) -> Value {
    Value::List(Gc::new(RefCell::new(l)))
}

pub fn o(o: Vec<(&str, Value)>) -> Value {
    let mut obj = Object::new();
    for (k, v) in o {
        obj.insert(k.to_owned(), v);
    }
    Value::Object(Gc::new(RefCell::new(obj)))
}

pub fn r(s: i64, e: i64) -> Value {
    Value::Range(s, e)
}

pub fn func() -> Value {
    Value::Function(std::rc::Rc::new(crate::function::Function::anilang_fn(
        vec![],
        vec![],
    )))
}

pub fn n() -> Value {
    Value::Null
}
