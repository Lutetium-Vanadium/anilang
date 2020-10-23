use super::Value;
use std::rc::Rc;

/// Only `PartialEq` can be implemented, since `f32` does not support `Eq`, and Null is not equal
/// to anything
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        let l = match self.try_cast(other.type_()) {
            Ok(l) => l,
            // No implicit cast means it cannot be compared
            Err(_) => return false,
        };

        let r = match other.try_cast(l.type_()) {
            Ok(r) => r,
            // No implicit cast means it cannot be compared
            Err(_) => return false,
        };

        match l {
            Value::Int(l) => l == r.into(),
            Value::Float(l) => l == r.into(),
            Value::Bool(l) => l == r.into(),
            Value::String(ref l_rc) => {
                // Easy to check if both are references to the same string, otherwise check if the
                // actual strings are equal
                Rc::ptr_eq(&l_rc, &r.clone().as_rc_str())
                    || l_rc.borrow().as_str() == r.as_ref_str().as_str()
            }
            // Functions are only equal if they are references to the same definition, the actual
            // args and function body are not considered.
            Value::Function(ref l) => Rc::ptr_eq(l, &r.as_rc_fn()),
            Value::Null => false,
        }
    }
}

/// Only `PartialOrd` can be implemented, since `f32` does not support `Ord`, non implicitly
/// castable type cannot be compared and Null is cannot be compared
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        let l = match self.try_cast(other.type_()) {
            Ok(l) => l,
            // No implicit cast means it cannot be compared
            Err(_) => return None,
        };

        let r = match other.try_cast(l.type_()) {
            Ok(r) => r,
            // No implicit cast means it cannot be compared
            Err(_) => return None,
        };

        match l {
            Value::Int(l) => l.partial_cmp(&r.into()),
            Value::Float(l) => l.partial_cmp(&r.into()),
            Value::Bool(l) => l.partial_cmp(&r.into()),
            Value::String(ref l) => l.borrow().as_str().partial_cmp(r.as_ref_str().as_str()),
            // Functions have no ordering as they are just a container for a `BlockNode`
            Value::Function(_) => None,
            Value::Null => None,
        }
    }
}