use crate::types::Type;
use enumflags2::BitFlags;
mod from;

type Result<T> = std::result::Result<T, ErrorKind>;
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    IncorrectType { got: Type, expected: BitFlags<Type> },
    IncorrectLeftType { got: Type, expected: BitFlags<Type> },
    IncorrectRightType { got: Type, expected: BitFlags<Type> },
    OutOfBounds { got: i64, start: i64, end: i64 },
    DivideByZero,
}

// Enum to store value of any type
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        let l = match self.try_cast(other.type_()) {
            Ok(l) => l,
            Err(_) => return false,
        };

        let r = match other.try_cast(l.type_()) {
            Ok(r) => r,
            Err(_) => return false,
        };

        match l {
            Value::Int(l) => l == r.into(),
            Value::Float(l) => l == r.into(),
            Value::Bool(l) => l == r.into(),
            Value::String(ref l) => l == r.as_str(),
            Value::Null => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        let l = match self.try_cast(other.type_()) {
            Ok(l) => l,
            Err(_) => return None,
        };

        let r = match other.try_cast(l.type_()) {
            Ok(r) => r,
            Err(_) => return None,
        };

        match l {
            Value::Int(l) => l.partial_cmp(&r.into()),
            Value::Float(l) => l.partial_cmp(&r.into()),
            Value::Bool(l) => l.partial_cmp(&r.into()),
            Value::String(ref l) => (l as &str).partial_cmp(r.as_str()),
            Value::Null => None,
        }
    }
}

// When printing we want to only show the inner value, which is what the user expects
// for example for an integer 1, when printing, the user expects for it to be printed as
// `1` and not Value::Int(1)
use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

// Also see `src/types.rs` for type impls &
//          `src/value/from.rs` for to primitive impls

/// impl for the various unary operations
impl Value {
    /// Unary Plus +<num>
    pub fn plus(self) -> Result<Value> {
        match self {
            Value::Int(_) => Ok(self),
            Value::Float(_) => Ok(self),
            _ => Err(ErrorKind::IncorrectType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Unary Minus -<num>
    pub fn minus(self) -> Result<Value> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(ErrorKind::IncorrectType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Unary Not !<val>
    pub fn not(self) -> Value {
        Value::Bool(!bool::from(self))
    }
}

/// impl for the various binary operations
impl Value {
    /// Binary Addition
    ///   * Arithemtic: <num> + <num>
    ///   * Concatenation: <str1> + <str2> = "<str1><str2>"
    pub fn add(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: right.type_().into(),
            })?;

        match left {
            Value::Int(left) => Ok(Value::Int(left + i64::from(right))),
            Value::Float(left) => Ok(Value::Float(left + f64::from(right))),
            Value::String(left) => Ok(Value::String(left + right.as_str())),
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float | Type::String,
            }),
        }
    }

    /// Binary subtraction <num> - <num>
    pub fn sub(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: right.type_().into(),
            })?;

        match left {
            Value::Int(left) => Ok(Value::Int(left - i64::from(right))),
            Value::Float(left) => Ok(Value::Float(left - f64::from(right))),
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Binary multiplication <num> * <num>
    pub fn mult(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        match left {
            Value::Int(left) => Ok(Value::Int(left * i64::from(right))),
            Value::Float(left) => Ok(Value::Float(left * f64::from(right))),
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Binary division <num> / <num>
    pub fn div(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        match left {
            Value::Int(left) => {
                let right: i64 = right.into();
                if right == 0 {
                    Err(ErrorKind::DivideByZero)
                } else {
                    Ok(Value::Int(left / right))
                }
            }
            Value::Float(left) => {
                let right: f64 = right.into();
                if right == 0.0 {
                    Err(ErrorKind::DivideByZero)
                } else {
                    Ok(Value::Float(left / right))
                }
            }
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Binary mod <num> % <num>
    pub fn modulo(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        match left {
            Value::Int(left) => {
                let right: i64 = right.into();
                if right == 0 {
                    Err(ErrorKind::DivideByZero)
                } else {
                    Ok(Value::Int(left % right))
                }
            }
            Value::Float(left) => {
                let right: f64 = right.into();
                if right == 0.0 {
                    Err(ErrorKind::DivideByZero)
                } else {
                    Ok(Value::Float(left % right))
                }
            }
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Binary exponentiation <num>^<num>
    pub fn pow(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        let left = self
            .try_cast(right.type_())
            .map_err(|_| ErrorKind::IncorrectRightType {
                got: right.type_(),
                expected: self.type_().into(),
            })?;

        match left {
            Value::Int(left) => {
                let right: i64 = right.into();
                if right > u32::MAX as i64 || right.is_negative() {
                    Err(ErrorKind::OutOfBounds {
                        got: right,
                        start: 0,
                        end: u32::MAX as i64,
                    })
                } else {
                    Ok(Value::Int(left.pow(right as u32)))
                }
            }
            Value::Float(left) => Ok(Value::Float(left.powf(right.into()))),
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }

    /// Binary or <val> || <val>
    pub fn or(self, right: Value) -> Value {
        if bool::from(&self) {
            self
        } else {
            right
        }
    }

    /// Binary and <val> && <val>
    pub fn and(self, right: Value) -> Value {
        if !bool::from(&self) {
            self
        } else {
            right
        }
    }

    /// <val> != <val>
    pub fn ne(self, right: Value) -> Value {
        Value::Bool(self != right)
    }

    /// <val> == <val>
    pub fn eq(self, right: Value) -> Value {
        Value::Bool(self == right)
    }

    /// <val> < <val>
    pub fn lt(self, right: Value) -> Value {
        Value::Bool(self < right)
    }

    /// <val> > <val>
    pub fn gt(self, right: Value) -> Value {
        Value::Bool(self > right)
    }

    /// <val> <= <val>
    pub fn le(self, right: Value) -> Value {
        Value::Bool(self <= right)
    }

    /// <val> >= <val>
    pub fn ge(self, right: Value) -> Value {
        Value::Bool(self >= right)
    }
}
