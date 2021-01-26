use super::Value;

/// When printing we want to only show the inner value, which is what the user expects
/// for example for an integer 1, when printing, the user expects for it to be printed as
/// `1` and not Value::Int(1)
use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s.borrow()),
            Value::List(ref l) => {
                let l = l.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if l.len() < 8 {
                    write!(f, "{:?}", l)
                } else {
                    write!(f, "{:#?}", l)
                }
            }
            Value::Object(ref o) => {
                let o = o.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if o.len() < 3 {
                    write!(f, "{:?}", o)
                } else {
                    write!(f, "{:#?}", o)
                }
            }
            Value::Range(s, e) => write!(f, "{} -> {}", s, e),
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => {
                let s = &s.borrow();
                // while printing quotes must be escaped to avoid confusion
                if s.contains('\'') && !s.contains('"') {
                    write!(f, "\"{}\"", s)
                } else {
                    write!(f, "'")?;
                    for i in s.chars() {
                        if i == '\'' {
                            write!(f, "\\{}", i)?;
                        } else {
                            write!(f, "{}", i)?;
                        }
                    }
                    write!(f, "'")
                }
            }
            Value::List(_) | Value::Object(_) => write!(f, "{}", self),
            Value::Range(s, e) => write!(f, "{}..{}", s, e),
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{:?}", i),
            Value::Float(fl) => write!(f, "{:?}", fl),
            Value::Bool(b) => write!(f, "{:?}", b),
            Value::Null => write!(f, "null"),
        }
    }
}
