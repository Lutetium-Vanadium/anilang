#[derive(Debug, PartialEq, Eq)]
pub enum Cast {
    Implicit,
    Explicit, // NOTE: there is no way to explicitly convert as of now
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Null,
}

impl Type {
    fn cast_type(&self, other: &Type) -> Cast {
        if self == other || other == &Type::Bool {
            return Cast::Implicit;
        }

        match self {
            Type::Int if other == &Type::Float => Cast::Implicit,
            Type::Float if other == &Type::Int => Cast::Implicit,
            Type::Bool => Cast::Implicit,
            _ => Cast::Explicit,
        }
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
            Value::String(_) => Type::String,
            Value::Null => Type::Null,
        }
    }

    /// Handles converting implicit casting, should only be called through try_cast()
    fn implicit_cast(&self, to_type: Type) -> Value {
        match self.type_() {
            Type::Int if to_type == Type::Float => {
                unimplemented!("Float is currently unimplemented in Value")
            }
            Type::Bool => Value::Bool(bool::from(self)),
            _ => panic!(
                "Unexpected explicit cast, for possible explicit casts call try_cast() instead"
            ),
        }
    }

    /// Performs implicit cast if available or else returns the cast type
    /// For now there is only implicit & explicit, so it will always error `Explicit`.
    /// If in the future, no more cast types are added, it will error `()`
    pub fn try_cast(self, to_type: Type) -> Result<Value, Cast> {
        let self_type = self.type_();
        match self_type.cast_type(&to_type) {
            Cast::Implicit if self_type == to_type => Ok(self),
            Cast::Implicit => Ok(self.implicit_cast(to_type)),
            Cast::Explicit => Err(Cast::Explicit),
        }
    }
}
