use crate::types::Type;
use enumflags2::BitFlags;
use std::cell::RefCell;
use std::rc::Rc;
mod from;
mod function;

#[cfg(test)]
mod tests;

pub use function::Function;

pub type Ref<T> = Rc<RefCell<T>>;
type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    IncorrectType { got: Type, expected: BitFlags<Type> },
    IncorrectLeftType { got: Type, expected: BitFlags<Type> },
    IncorrectRightType { got: Type, expected: BitFlags<Type> },
    OutOfBounds { got: i64, start: i64, end: i64 },
    DivideByZero,
}

/// Enum to store value of any type, values which are tuple structs, contain the actual value in
/// the element
#[derive(Debug, Clone)]
pub enum Value {
    String(Rc<RefCell<String>>),
    Function(Rc<Function>), // Functions are not mutable
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// Only `PartialEq` can be implemented, since `f32` does not support `Eq`, and Null is not equal
/// to anything
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
            Value::String(ref l_rc) => {
                // Easy to check if both are references to the same string, otherwise check if the
                // actual strings are equal
                Rc::ptr_eq(&l_rc, &r.clone().as_rc_str())
                    || l_rc.borrow().as_str() == r.as_ref_str().as_str()
            }
            // Functions are only equal if they are references to the same defination, the actual
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
            Value::String(ref l) => l.borrow().as_str().partial_cmp(r.as_ref_str().as_str()),
            Value::Function(_) => None,
            Value::Null => None,
        }
    }
}

/// When printing we want to only show the inner value, which is what the user expects
/// for example for an integer 1, when printing, the user expects for it to be printed as
/// `1` and not Value::Int(1)
use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => {
                let s = &s.borrow();
                // while printing quotes must be escaped to avoid confusion
                if s.contains("'") && !s.contains("\"") {
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
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

// Also see `core/src/types.rs` for type impls &
//          `core/src/value/from.rs` for to base type impls

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
            Value::String(left) => {
                let right: Ref<String> = right.as_rc_str();
                Ok(Value::String(if Rc::strong_count(&left) == 1 {
                    left.borrow_mut().push_str(&right.borrow());
                    left
                } else if Rc::strong_count(&right) == 1 {
                    right.borrow_mut().push_str(&left.borrow());
                    right
                } else {
                    let l = left.borrow();
                    let r = right.borrow();
                    let mut s = String::with_capacity(l.len() + r.len());
                    s += &l;
                    s += &r;
                    Rc::new(RefCell::new(s))
                }))
            }
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
