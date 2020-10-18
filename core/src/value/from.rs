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
use super::Value;
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

impl From<Value> for bool {
    fn from(val: Value) -> bool {
        match val {
            Value::String(s) => s.borrow().len() != 0,
            Value::Int(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
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
            Value::Null => false,
        }
    }
}

impl From<Value> for Rc<RefCell<String>> {
    /// Should only be called for Value::String
    fn from(val: Value) -> Rc<RefCell<String>> {
        match val {
            Value::String(r) => r,
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a Value> for Ref<'a, String> {
    /// Should only be called for &Value::String
    fn from(val: &'a Value) -> Ref<'a, String> {
        match val {
            Value::String(ref s) => s.borrow(),
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
        assert_eq!(Rc::<RefCell<String>>::from(s("s")).borrow().as_str(), "s");
        assert_eq!(std::cell::Ref::from(&s("s")).as_str(), "s");
    }
}
