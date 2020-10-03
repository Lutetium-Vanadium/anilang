/// Contains the From declarations for Value to Primitive
///
/// NOTE: it intentionaly crashes when wrong type is trying to be converted. This is only meant for
/// easy conversion between determined types. For example if you know a value is a Value::Int, it
/// is annoying to get the int value using:
/// ```
/// match value {
///     Value::Int(i) => i,
///     _ => unreachable!()
/// }
/// ```
use super::Value;

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
            Value::String(ref s) => s.len() != 0,
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
            Value::String(ref s) => s.len() != 0,
            Value::Int(i) => i != &0,
            Value::Float(f) => f != &0.0,
            Value::Bool(b) => *b,
            Value::Null => false,
        }
    }
}

impl From<Value> for String {
    /// Should only be called for Value::String
    fn from(val: Value) -> String {
        match val {
            Value::String(s) => s,
            _ => unreachable!(),
        }
    }
}

impl Value {
    /// Should only be called for &Value::String
    pub fn as_str(&self) -> &str {
        match self {
            Value::String(ref s) => s,
            _ => unreachable!(),
        }
    }
}
