type Result<T> = std::result::Result<T, ErrorKind>;
pub enum ErrorKind {
    IncorrectLeftType,
    IncorrectRightType,
    OutOfBounds(i64, i64),
    DivideByZero,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
    Null,
}

impl From<Value> for bool {
    fn from(val: Value) -> bool {
        match val {
            Value::String(ref s) => s.len() != 0,
            Value::Int(i) => i != 0,
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
            Value::Bool(b) => *b,
            Value::Null => false,
        }
    }
}

use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    fn type_string(&self) -> &str {
        match self {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Null => "null",
        }
    }

    pub fn add(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val + other_val)),
                _ => Err(ErrorKind::IncorrectRightType),
            },
            Value::String(self_val) => match right {
                Value::String(other_val) => Ok(Value::String(self_val + &other_val)),
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn sub(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val - other_val)),
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn mult(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val * other_val)),
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn modulo(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if self_val == 0 || other_val == 0 {
                        Err(ErrorKind::DivideByZero)
                    } else {
                        Ok(Value::Int(self_val % other_val))
                    }
                }
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn div(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if self_val == 0 || other_val == 0 {
                        Err(ErrorKind::DivideByZero)
                    } else {
                        Ok(Value::Int(self_val / other_val))
                    }
                }
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn pow(self, right: Value) -> Result<Value> {
        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if other_val > u32::MAX as i64 || other_val.is_negative() {
                        Err(ErrorKind::OutOfBounds(0, u32::MAX as i64))
                    } else {
                        Ok(Value::Int(self_val.pow(other_val as u32)))
                    }
                }
                _ => Err(ErrorKind::IncorrectRightType),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    pub fn or(self, right: Value) -> Value {
        if bool::from(&self) {
            self
        } else {
            right
        }
    }

    pub fn and(self, right: Value) -> Value {
        if !bool::from(&self) {
            self
        } else {
            right
        }
    }

    pub fn ne(self, right: Value) -> Value {
        Value::Bool(self != right)
    }

    pub fn eq(self, right: Value) -> Value {
        Value::Bool(self == right)
    }

    pub fn lt(self, right: Value) -> Value {
        Value::Bool(self < right)
    }
    pub fn gt(self, right: Value) -> Value {
        Value::Bool(self > right)
    }

    pub fn le(self, right: Value) -> Value {
        Value::Bool(self <= right)
    }

    pub fn ge(self, right: Value) -> Value {
        Value::Bool(self >= right)
    }
}
