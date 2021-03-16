use enumflags2::BitFlags;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub enum Cast {
    Implicit(Type),
    Explicit, // NOTE: there is no way to explicitly convert as of now
}

/// The different types of values that can exist
///
/// Derives from BitFlags, to allow for multiple types in the same byte for errors
///
/// For example for some arithmetic between to numbers, if a different type is found, this expected
/// type can be set as `expected` given below.
/// ```
/// use vm::Type;
///
/// let expected = Type::Int | Type::Float;
/// assert!(expected.contains(Type::Int));
/// assert!(expected.contains(Type::Float));
/// ```
/// Also since Type is `#[repr(u8)]`, it is only one byte in size, therefore multiple types can be
/// stored in just one byte, rather than storing multiple types in a `Vec` or array
#[derive(Copy, Clone, Debug, PartialEq, Eq, BitFlags)]
#[rustfmt::skip]
#[repr(u16)]
pub enum Type {
    Int      = 0b000000001,
    Float    = 0b000000010,
    String   = 0b000000100,
    List     = 0b000001000,
    Object   = 0b000010000,
    Range    = 0b000100000,
    Bool     = 0b001000000,
    Function = 0b010000000,
    Null     = 0b100000000,
}

impl Type {
    pub fn cast_type(&self, other: &Type) -> Cast {
        if self == other {
            return Cast::Implicit(*self);
        }

        match self {
            Type::Int if other == &Type::Float => Cast::Implicit(Type::Float),
            Type::Float if other == &Type::Int => Cast::Implicit(Type::Float),
            _ => Cast::Explicit,
        }
    }
}

impl From<u16> for Type {
    fn from(value: u16) -> Type {
        match value {
            0b000000001 => Type::Int,
            0b000000010 => Type::Float,
            0b000000100 => Type::String,
            0b000001000 => Type::List,
            0b000010000 => Type::Object,
            0b000100000 => Type::Range,
            0b001000000 => Type::Bool,
            0b010000000 => Type::Function,
            0b100000000 => Type::Null,
            n => panic!(
                "Invalid u16 {}, this method is only meant to be called with valid tags.",
                n
            ),
        }
    }
}

use std::fmt;
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "int",
                Type::Float => "float",
                Type::String => "string",
                Type::List => "list",
                Type::Object => "object",
                Type::Range => "range",
                Type::Bool => "bool",
                Type::Function => "function",
                Type::Null => "null",
            }
        )
    }
}

/// Rust doesn't allow implementing non user defined traits on non user defined structs
/// This is a workaround to give to_string, so the diagnostics can print easily
///
/// `error[E0117]: only traits defined in the current crate can be implemented for arbitrary types`
pub trait ToString {
    fn to_string(&self) -> String;
}
impl ToString for BitFlags<Type> {
    fn to_string(&self) -> String {
        let mut iter = self.iter();
        let mut s = format!("{}", iter.next().unwrap());
        for t in iter {
            s += &format!(" | {}", t);
        }
        s
    }
}

use crate::value::Value;
impl Value {
    /// Gives the type of the value, unfortunately type is a keyword in rust, so type_() has been
    /// used
    pub fn type_(&self) -> Type {
        match self {
            Value::Bool(_) => Type::Bool,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::List(_) => Type::List,
            Value::Object(_) => Type::Object,
            Value::Range(..) => Type::Range,
            Value::Function(_) => Type::Function,
            Value::Null => Type::Null,
        }
    }

    /// Handles converting implicit casting otherwise panics, should only be called through try_cast()
    fn implicit_cast(&self, to_type: Type) -> Value {
        match to_type {
            Type::Float if self.type_() == Type::Int => {
                Value::Float(self.into())
            }
            _ => unreachable!(
                "Unexpected explicit cast from {:?} to {:?}, for possible explicit casts call try_cast() instead",
                self.type_(), to_type
            ),
        }
    }

    /// Performs implicit cast if available or else returns the cast type
    /// For now there is only implicit & explicit, so it will always error `Explicit`.
    /// In the future, if no more cast types are added, it will error `()`
    pub fn try_cast(&self, to_type: Type) -> Result<Value, Cast> {
        let self_type = self.type_();
        match self_type.cast_type(&to_type) {
            Cast::Implicit(to_type) => {
                if self_type == to_type {
                    Ok(self.clone())
                } else {
                    Ok(self.implicit_cast(to_type))
                }
            }
            Cast::Explicit => Err(Cast::Explicit),
        }
    }
}
