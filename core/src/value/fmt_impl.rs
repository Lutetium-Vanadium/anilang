use super::Value;

/// When printing we want to only show the inner value, which is what the user expects
/// for example for an integer 1, when printing, the user expects for it to be printed as
/// `1` and not Value::Int(1)
use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s.borrow()),
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}
