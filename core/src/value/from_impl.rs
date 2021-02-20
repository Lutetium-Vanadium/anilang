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
use super::{Function, List, Object, Value};
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
            Value::List(l) => l.borrow().len() != 0,
            Value::Object(o) => o.borrow().len() != 0,
            // A range is considered truthy as long as it doesn't have a length of zero
            Value::Range(s, e) => s != e,
            Value::Int(i) => i != 0,
            Value::Float(f) => f.abs() > f64::EPSILON,
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
            Value::List(l) => l.borrow().len() != 0,
            Value::Object(o) => o.borrow().len() != 0,
            // A range is considered truthy as long as it doesn't have a length of zero
            Value::Range(s, e) => s != e,
            Value::Int(i) => i != &0,
            // f64 comparisons are not completely accurate, so check if it is within the threshold
            Value::Float(f) => f.abs() > f64::EPSILON,
            Value::Bool(b) => *b,
            Value::Function(_) => true,
            Value::Null => false,
        }
    }
}

use std::ops::Range;
impl From<Value> for Range<i64> {
    fn from(val: Value) -> Range<i64> {
        match val {
            Value::Range(s, e) => s..e,
            _ => unreachable!(),
        }
    }
}

impl From<&Value> for Range<i64> {
    fn from(val: &Value) -> Range<i64> {
        match val {
            Value::Range(s, e) => *s..*e,
            _ => unreachable!(),
        }
    }
}

impl Value {
    pub fn into_rc_str(self) -> Rc<RefCell<String>> {
        match self {
            Value::String(s) => s,
            _ => unreachable!(),
        }
    }

    pub fn into_str(self) -> String {
        Rc::try_unwrap(self.into_rc_str())
            .map(RefCell::into_inner)
            .unwrap_or_else(|rc| rc.borrow().as_str().to_owned())
    }

    pub fn to_ref_str(&self) -> Ref<String> {
        match self {
            Value::String(ref s) => s.borrow(),
            _ => unreachable!(),
        }
    }

    pub fn into_rc_list(self) -> Rc<RefCell<List>> {
        match self {
            Value::List(l) => l,
            _ => unreachable!(),
        }
    }

    pub fn to_ref_list(&self) -> Ref<List> {
        match self {
            Value::List(ref l) => l.borrow(),
            _ => unreachable!(),
        }
    }

    pub fn into_rc_fn(self) -> Rc<Function> {
        match self {
            Value::Function(f) => f,
            _ => unreachable!(),
        }
    }

    pub fn into_rc_obj(self) -> Rc<RefCell<Object>> {
        match self {
            Value::Object(o) => o,
            _ => unreachable!(),
        }
    }

    pub fn to_ref_obj(&self) -> Ref<Object> {
        match self {
            Value::Object(ref o) => o.borrow(),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn val_to_int() {
        assert_eq!(i64::from(i(10)), 10);
    }

    #[test]
    fn val_to_float() {
        assert_eq!(f64::from(i(10)), 10.0);
        assert_eq!(f64::from(f(10.0)), 10.0);
    }

    #[test]
    fn val_to_bool() {
        assert_eq!(bool::from(i(10)), true);
        assert_eq!(bool::from(i(0)), false);

        assert_eq!(bool::from(f(10.0)), true);
        assert_eq!(bool::from(f(0.0)), false);

        assert_eq!(bool::from(b(true)), true);
        assert_eq!(bool::from(b(false)), false);

        assert_eq!(bool::from(r(0, 1)), true);
        assert_eq!(bool::from(r(0, 0)), false);

        assert_eq!(bool::from(s("s")), true);
        assert_eq!(bool::from(s("")), false);

        assert_eq!(bool::from(l(vec![i(0)])), true);
        assert_eq!(bool::from(l(vec![])), false);

        assert_eq!(bool::from(o(vec![("hello", i(0))])), true);
        assert_eq!(bool::from(o(vec![])), false);

        assert_eq!(bool::from(func()), true);
        assert_eq!(bool::from(n()), false);
    }

    #[test]
    fn val_to_range() {
        assert_eq!(Range::from(r(0, 1)), 0..1);
    }

    #[test]
    fn val_to_ref_string() {
        assert_eq!(s("s").into_rc_str().borrow().as_str(), "s");
        assert_eq!(s("s").to_ref_str().as_str(), "s");
    }

    #[test]
    fn val_to_str() {
        assert_eq!(s("string").into_str().as_str(), "string");
    }

    #[test]
    fn val_to_ref_list() {
        assert_eq!(
            l(vec![i(0), i(1), s("s")]).into_rc_list().borrow()[..],
            [i(0), i(1), s("s")]
        );
        assert_eq!(
            l(vec![i(0), i(1), s("s")]).to_ref_list()[..],
            [i(0), i(1), s("s")]
        );
    }

    #[test]
    fn val_to_ref_obj() {
        let obj = o(vec![("a", i(0)), ("b", b(true))]);

        let ref_obj = obj.to_ref_obj();

        assert_eq!(*ref_obj.get("a").unwrap(), i(0));
        assert_eq!(*ref_obj.get("b").unwrap(), b(true));
        drop(ref_obj);

        let rc_obj = obj.into_rc_obj();

        assert_eq!(*rc_obj.borrow().get("a").unwrap(), i(0));
        assert_eq!(*rc_obj.borrow().get("b").unwrap(), b(true));
    }

    #[test]
    fn val_to_ref_fn() {
        let rc_f = Rc::new(Function::anilang_fn(vec!["a".into(), "b".into()], vec![]));
        let f = Value::Function(Rc::clone(&rc_f));
        assert!(Rc::ptr_eq(&rc_f, &f.into_rc_fn()));
    }
}
