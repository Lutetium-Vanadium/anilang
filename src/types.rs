use enumflags2::BitFlags;

#[derive(Debug, PartialEq, Eq)]
pub enum Cast {
    Implicit(Type),
    Explicit, // NOTE: there is no way to explicitly convert as of now
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BitFlags)]
#[repr(u8)]
pub enum Type {
    Int = 0b00001,
    Float = 0b00010,
    String = 0b00100,
    Bool = 0b01000,
    Null = 0b10000,
}

impl Type {
    fn cast_type(&self, other: &Type) -> Cast {
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

use std::fmt;
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "int",
                Type::Float => "float",
                Type::Bool => "bool",
                Type::String => "string",
                Type::Null => "null",
            }
        )
    }
}

/// For whatever reason, it won't allow implementing `std::fmt::Display` for library structs
/// This is a workaround to give to_string, so the error bag can print easily
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
            Value::Null => Type::Null,
        }
    }

    /// Handles converting implicit casting, should only be called through try_cast()
    fn implicit_cast(&self, to_type: Type) -> Value {
        match to_type {
            Type::Float if self.type_() == Type::Int => {
                Value::Float(self.into())
            }
            _ => panic!(
                "Unexpected explicit cast from {:?} to {:?}, for possible explicit casts call try_cast() instead",
                self, to_type
            ),
        }
    }

    /// Performs implicit cast if available or else returns the cast type
    /// For now there is only implicit & explicit, so it will always error `Explicit`.
    /// If in the future, no more cast types are added, it will error `()`
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