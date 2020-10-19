/// Contains the From declarations for Value to Primitive
///
/// NOTE: it intentionaly crashes when wrong type is trying to be converted. This is only meant for
/// easy conversion between determined types. For example if you know a value is a Value::Int, it
/// is annoying to get the int value using:
/// ```compile_fail
/// match value {
///     Value::Int(i) => i,
///     _ => unreachable!()
/// }
/// ```
use super::{Function, Value};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

impl From<Value> for i64 {
    /// Should only be called for Value::Int
    fn from(val: Value) -> i64 {
        match val {
            Value::Int(i) => i,
            _ => unreachable!(),
        }
    }
}

impl From<&Value> for i64 {
    /// Should only be called for &Value::Int
    fn from(val: &Value) -> i64 {
        match val {
            Value::Int(i) => *i,
            _ => unreachable!(),
        }
    }
}

impl From<Value> for f64 {
    /// Should only be called for Value::Int or Value::Float
    fn from(val: Value) -> f64 {
        match val {
            Value::Int(i) => i as f64,
            Value::Float(f) => f,
            _ => unreachable!(),
        }
    }
}

impl From<&Value> for f64 {
    /// Should only be called for &Value::Int or &Value::Float
    fn from(val: &Value) -> f64 {
        match val {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => unreachable!(),
        }
    }
}

// NOTE while the boolean conversion exists from all types, it is not exposed as a implicit
// conversion, these conversions are only there for simplicity in the comparison operators, i.e.
// '||', '>' etc
impl From<Value> for bool {
    fn from(val: Value) -> bool {
        match val {
            Value::String(s) => s.borrow().len() != 0,
            Value::Int(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
            // Function objects are truthy, but the returned after calling a Function need not be
            // Value::Function refers to the function object itself, and not the return type of the
            // function
            Value::Function(_) => true,
            Value::Null => false,
        }
    }
}

impl From<&Value> for bool {
    fn from(val: &Value) -> bool {
        match val {
            Value::String(s) => s.borrow().len() != 0,
            Value::Int(i) => i != &0,
            Value::Float(f) => f != &0.0,
            Value::Bool(b) => *b,
            Value::Function(_) => true,
            Value::Null => false,
        }
    }
}

impl Value {
    pub fn as_rc_str(self) -> Rc<RefCell<String>> {
        match self {
            Value::String(s) => s,
            _ => unreachable!(),
        }
    }

    pub fn as_ref_str(&self) -> Ref<String> {
        match self {
            Value::String(ref s) => s.borrow(),
            _ => unreachable!(),
        }
    }

    pub fn as_rc_fn(self) -> Rc<Function> {
        match self {
            Value::Function(f) => f,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(s: &str) -> Value {
        Value::String(Rc::new(RefCell::new(s.to_owned())))
    }

    #[test]
    fn val_to_int() {
        assert_eq!(i64::from(Value::Int(10)), 10);
    }

    #[test]
    fn val_to_float() {
        assert_eq!(f64::from(Value::Int(10)), 10.0);
        assert_eq!(f64::from(Value::Float(10.0)), 10.0);
    }

    #[test]
    fn val_to_bool() {
        assert_eq!(bool::from(Value::Int(10)), true);
        assert_eq!(bool::from(Value::Int(0)), false);

        assert_eq!(bool::from(Value::Float(10.0)), true);
        assert_eq!(bool::from(Value::Float(0.0)), false);

        assert_eq!(bool::from(Value::Bool(true)), true);
        assert_eq!(bool::from(Value::Bool(false)), false);

        assert_eq!(bool::from(s("s")), true);
        assert_eq!(bool::from(s("")), false);

        assert_eq!(bool::from(Value::Null), false);
    }

    #[test]
    fn val_to_ref_string() {
        assert_eq!(s("s").as_rc_str().borrow().as_str(), "s");
        assert_eq!(s("s").as_ref_str().as_str(), "s");
    }
}
