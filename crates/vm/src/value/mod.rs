use crate::types::{Cast, Type};
use enumflags2::BitFlags;
use std::cell::RefCell;
use std::rc::Rc;

mod cmp_impl;
mod fmt_impl;
mod from_impl;
mod indexing;
mod serialize;

#[cfg(test)]
mod tests;

use crate::function::Function;

pub type List = Vec<Value>;
pub type Object = std::collections::HashMap<String, Value>;
pub type Ref<T> = Rc<RefCell<T>>;
pub(crate) type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    IncorrectType { got: Type, expected: BitFlags<Type> },
    IncorrectLeftType { got: Type, expected: BitFlags<Type> },
    IncorrectRightType { got: Type, expected: BitFlags<Type> },
    OutOfBounds { got: i64, start: i64, end: i64 },
    IndexOutOfRange { index: i64, len: i64 },
    Unindexable { val_t: Type, index_t: Type },
    CannotCompare { left: Type, right: Type },
    IncorrectArgCount { got: usize, expected: usize },
    InvalidProperty { val: Value, property: Ref<String> },
    ReadonlyProperty { val: Value, property: Ref<String> },
    DivideByZero,
    Other { message: String },
}

/// Enum to store value of any type, values which are tuple structs, contain the actual value in
/// the element
#[derive(Clone)]
pub enum Value {
    /// `String`s are expensive to copy, so a `Rc` is used, copying the reference to the String,
    /// and not the string itself. `Rc<T>` however gives only immutable access to the inner `T`,
    /// so instead of directly using `Rc<String>`, we use `Rc<RefCell<String>>` to provide mutable
    /// strings.
    String(Ref<String>),
    /// `Vec`s are expensive to copy, so a `Rc` is used, copying the reference to the List, and not
    /// the list itself. `Rc<T>` however gives only immutable access to the inner `T`, so instead
    /// of directly using `Rc<Vec>`, we use `Rc<RefCell<Vec>>` to provide mutable lists.
    List(Ref<List>),
    /// `HashMap`s are expensive to copy so a `Rc` is used, copying the reference to the Object, and
    /// not the object itself. `Rc<T>` however gives only immutable access to the inner `T`, so
    /// instead of directly using `Rc<HashMap>`, we use `Rc<RefCell<HashMap>>` to provide mutable
    /// objects.
    Object(Ref<Object>),
    /// A pointer to a function, see `core/src/value/function/mod.rs` for more information, easy to
    /// copy so not placed in a `Rc`.
    Function(Rc<Function>),
    /// A range value, easy to copy, so it is not placed in a `Rc`
    Range(i64, i64),
    /// A primitive integer type, easy to copy, so is not placed in a `Rc`
    Int(i64),
    /// A primitive float type, easy to copy, so is not placed in a `Rc`
    Float(f64),
    /// A primitive bool type, easy to copy, so is not placed in a `Rc`
    Bool(bool),
    /// `null` value is generated as a default value when no other value is known. So it is mainly
    /// used when an error has occured, but the function still has to return a value even if it
    /// will not be used.
    Null,
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
}

use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};

impl Neg for Value {
    type Output = Result<Self>;

    fn neg(self) -> Result<Self> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(ErrorKind::IncorrectType {
                got: self.type_(),
                expected: Type::Int | Type::Float,
            }),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Value {
        Value::Bool(!bool::from(self))
    }
}

impl Value {
    /// Range (s..e)
    ///
    /// NOTE currently only int to int Ranges are allowed
    pub fn range_to(self, right: Value) -> Result<Value> {
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
            Value::Int(start) => Ok(Value::Range(start, i64::from(right))),
            _ => Err(ErrorKind::IncorrectType {
                got: right.type_(),
                expected: Type::Int.into(),
            }),
        }
    }
}

use std::cmp::Ordering;
/// impl for the various binary operations
impl Add for Value {
    type Output = Result<Self>;

    /// Binary Addition
    ///   * Arithmetic: <num> + <num>
    ///   * Concatenation: <str1> + <str2> = "<str1><str2>" |
    ///                    <lst1> + <lst2> = [...<lst1>, ...<lst2>]
    fn add(self, right: Value) -> Result<Value> {
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
                let right = right.into_rc_str();

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
            Value::List(left) => {
                let right = right.into_rc_list();

                Ok(Value::List(if Rc::strong_count(&left) == 1 {
                    left.borrow_mut().extend_from_slice(&right.borrow()[..]);
                    left
                } else if Rc::strong_count(&right) == 1 {
                    right.borrow_mut().extend_from_slice(&left.borrow()[..]);
                    right
                } else {
                    let l = left.borrow();
                    let r = right.borrow();
                    let mut v = Vec::with_capacity(l.len() + r.len());
                    v.extend_from_slice(&l[..]);
                    v.extend_from_slice(&r[..]);
                    Rc::new(RefCell::new(v))
                }))
            }
            _ => Err(ErrorKind::IncorrectLeftType {
                got: self.type_(),
                expected: Type::Int | Type::Float | Type::String,
            }),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    /// Binary subtraction <num> - <num>
    fn sub(self, right: Value) -> Result<Value> {
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
}

impl Mul for Value {
    type Output = Result<Self>;

    /// Binary multiplication <num> * <num>
    fn mul(self, right: Value) -> Result<Value> {
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
}

impl Div for Value {
    type Output = Result<Self>;

    /// Binary division <num> / <num>
    fn div(self, right: Value) -> Result<Value> {
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
}

impl Rem for Value {
    type Output = Result<Self>;

    /// Binary mod <num> % <num>
    fn rem(self, right: Value) -> Result<Value> {
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
}

impl Value {
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
    pub fn ne(self, right: Value) -> Result<Value> {
        match self.type_().cast_type(&right.type_()) {
            Cast::Implicit(_) => Ok(Value::Bool(self != right)),
            _ => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }

    /// <val> == <val>
    pub fn eq(self, right: Value) -> Result<Value> {
        match self.type_().cast_type(&right.type_()) {
            Cast::Implicit(_) => Ok(Value::Bool(self == right)),
            _ => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }

    /// <val> < <val>
    pub fn lt(self, right: Value) -> Result<Value> {
        match self.partial_cmp(&right) {
            Some(ordering) => Ok(Value::Bool(ordering == Ordering::Less)),
            None => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }

    /// <val> > <val>
    pub fn gt(self, right: Value) -> Result<Value> {
        match self.partial_cmp(&right) {
            Some(ordering) => Ok(Value::Bool(ordering == Ordering::Greater)),
            None => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }

    /// <val> <= <val>
    pub fn le(self, right: Value) -> Result<Value> {
        match self.partial_cmp(&right) {
            Some(ordering) => Ok(Value::Bool(ordering.then(Ordering::Less) == Ordering::Less)),
            None => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }

    /// <val> >= <val>
    pub fn ge(self, right: Value) -> Result<Value> {
        match self.partial_cmp(&right) {
            Some(ordering) => Ok(Value::Bool(
                ordering.then(Ordering::Greater) == Ordering::Greater,
            )),
            None => Err(ErrorKind::CannotCompare {
                left: self.type_(),
                right: right.type_(),
            }),
        }
    }
}
