use crate::types::Type;
use enumflags2::BitFlags;
use std::cell::RefCell;
use std::rc::Rc;
mod cmp_impl;
mod fmt_impl;
mod from_impl;
mod function;

#[cfg(test)]
mod tests;

pub use function::Function;

pub type List = Vec<Value>;
pub type Ref<T> = Rc<RefCell<T>>;
type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    IncorrectType { got: Type, expected: BitFlags<Type> },
    IncorrectLeftType { got: Type, expected: BitFlags<Type> },
    IncorrectRightType { got: Type, expected: BitFlags<Type> },
    OutOfBounds { got: i64, start: i64, end: i64 },
    IndexOutOfRange { index: i64, len: i64 },
    Unindexable { val_t: Type, index_t: Type },
    DivideByZero,
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
    /// `Vec`s are expensive to copy, so a `Rc` is used, copying the reference to the String,
    /// and not the string itself. `Rc<T>` however gives only immutable access to the inner `T`,
    /// so instead of directly using `Rc<Vec>`, we use `Rc<RefCell<Vec>>` to provide mutable
    /// strings.
    List(Ref<List>),
    /// Functions are expensive to copy because they contain the whole function body as well as
    /// a `Vec<String>`, therefore a `Rc` is used. Since functions are immutable, `Rc<Function>`
    /// can directly be used.
    Function(Rc<Function>), // Functions are not mutable
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

/// Convert an `i64` index on some length, to a `usize`
///
/// len has to be a positive i64.
/// if `-len <= index < len` returns a `usize`
/// otherwise returns an error
///
/// It works similar to python indexing
///   0   1   2   3   4   5   6   7   8
/// .---.---.---.---.---.---.---.---.---.
/// |   |   |   |   |   |   |   |   |   | ---> len = 9
/// '---'---'---'---'---'---'---'---'---'
///  -9  -8  -7  -6  -5  -4  -3  -2  -1
fn normalise_index(index: i64, len: i64) -> Result<usize> {
    if index < 0 {
        if len < -index {
            Err(ErrorKind::IndexOutOfRange { index, len })
        } else {
            Ok((len + index) as usize)
        }
    } else {
        if len <= index {
            Err(ErrorKind::IndexOutOfRange { index, len })
        } else {
            Ok(index as usize)
        }
    }
}

/// impl for index operations
impl Value {
    pub fn indexable(&self, index_type: Type) -> bool {
        match self.type_() {
            Type::String if index_type == Type::Int => true,
            Type::List if index_type == Type::Int => true,
            _ => false,
        }
    }

    pub fn get_at(self, index: Value) -> Result<Value> {
        if !self.indexable(index.type_()) {
            return Err(ErrorKind::Unindexable {
                val_t: self.type_(),
                index_t: index.type_(),
            });
        }

        match self {
            Value::String(s) => {
                let s = s.borrow();
                let i = normalise_index(i64::from(index), s.chars().count() as i64)?;

                Ok(Value::String(Rc::new(RefCell::new(String::from(
                    s.chars().nth(i).unwrap(),
                )))))
            }
            Value::List(l) => {
                let l = l.borrow();
                let i = normalise_index(i64::from(index), l.len() as i64)?;

                Ok(l[i].clone())
            }
            _ => unreachable!(),
        }
    }

    pub fn set_at(self, index: Value, value: Value) -> Result<Value> {
        if !self.indexable(index.type_()) {
            return Err(ErrorKind::Unindexable {
                val_t: self.type_(),
                index_t: index.type_(),
            });
        }

        match &self {
            Value::String(s) => {
                let value = value
                    .try_cast(Type::String)
                    .map_err(|_| ErrorKind::IncorrectType {
                        got: value.type_(),
                        expected: Type::String.into(),
                    })?;

                let i = normalise_index(i64::from(index), s.borrow().chars().count() as i64)?;

                let (index, _) = s.borrow().char_indices().nth(i).unwrap();
                s.borrow_mut()
                    .replace_range(index..(index + 1), value.as_ref_str().as_str());

                Ok(self)
            }
            Value::List(l) => {
                let i = normalise_index(i64::from(index), l.borrow().len() as i64)?;

                l.borrow_mut()[i] = value;

                Ok(self)
            }
            _ => unreachable!(),
        }
    }
}

/// impl for the various binary operations
impl Value {
    /// Binary Addition
    ///   * Arithmetic: <num> + <num>
    ///   * Concatenation: <str1> + <str2> = "<str1><str2>" |
    ///                    <lst1> + <lst2> = [...<lst1>, ...<lst2>]
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
                let right = right.as_rc_str();

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
                let right = right.as_rc_list();

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
