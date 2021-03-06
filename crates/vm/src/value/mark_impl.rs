use super::*;
use gc::{Gc, Mark};

unsafe impl Mark for Value {
    fn mark(&self) {
        match self {
            Value::String(s) => Gc::mark(s),
            Value::List(l) => Gc::mark(l),
            Value::Object(o) => Gc::mark(o),
            Value::Function(f) => (*f).mark(),
            _ => {}
        }
    }

    fn update_reachable(&self) {
        match self {
            Value::String(s) => Gc::update_reachable(s),
            Value::List(l) => Gc::update_reachable(l),
            Value::Object(o) => Gc::update_reachable(o),
            Value::Function(f) => (*f).update_reachable(),
            _ => {}
        }
    }
}
