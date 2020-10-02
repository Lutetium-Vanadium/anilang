type Result<T> = std::result::Result<T, ErrorKind>;
pub enum ErrorKind {
    IncorrectType,
    IncorrectLeftType,
    IncorrectRightType,
    OutOfBounds(i64, i64),
    DivideByZero,
}

// Enum to store value of any type
// TODO: Add Float (floats are currently unparsable)
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

// When printing we want to only show the inner value, which is what the user expects
// for example for an integer 1, when printing, the user expects for it to be printed as
// `1` and not Value::Int(1)
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

// Also see `src/types.rs` for type impls

// impl for the various unary operations
impl Value {
    // Unary Plus +<num>
    pub fn plus(self) -> Result<Value> {
        match self {
            Value::Int(_) => Ok(self),
            _ => Err(ErrorKind::IncorrectType),
        }
    }

    // Unary Minus -<num>
    pub fn minus(self) -> Result<Value> {
        match self {
            Value::Int(i) => Ok(Value::Int(-i)),
            _ => Err(ErrorKind::IncorrectType),
        }
    }

    // Unary Not !<val>
    pub fn not(self) -> Value {
        Value::Bool(!bool::from(self))
    }
}

// impl for the various binary operations
impl Value {
    // Binary Addition
    //   * Arithemtic: <num> + <num>
    //   * Concatenation: <str1> + <str2> = "<str1><str2>"
    pub fn add(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val + other_val)),
                _ => unreachable!(),
            },
            Value::String(self_val) => match right {
                Value::String(other_val) => Ok(Value::String(self_val + &other_val)),
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary subtraction <num> - <num>
    pub fn sub(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val - other_val)),
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary multiplication <num> * <num>
    pub fn mult(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => Ok(Value::Int(self_val * other_val)),
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary division <num> / <num>
    pub fn div(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if self_val == 0 || other_val == 0 {
                        Err(ErrorKind::DivideByZero)
                    } else {
                        Ok(Value::Int(self_val / other_val))
                    }
                }
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary mod <num> % <num>
    pub fn modulo(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if self_val == 0 || other_val == 0 {
                        Err(ErrorKind::DivideByZero)
                    } else {
                        Ok(Value::Int(self_val % other_val))
                    }
                }
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary exponentiation <num>^<num>
    pub fn pow(self, right: Value) -> Result<Value> {
        let right = right
            .try_cast(self.type_())
            .map_err(|_| ErrorKind::IncorrectRightType)?;

        match self {
            Value::Int(self_val) => match right {
                Value::Int(other_val) => {
                    if other_val > u32::MAX as i64 || other_val.is_negative() {
                        Err(ErrorKind::OutOfBounds(0, u32::MAX as i64))
                    } else {
                        Ok(Value::Int(self_val.pow(other_val as u32)))
                    }
                }
                _ => unreachable!(),
            },
            _ => Err(ErrorKind::IncorrectLeftType),
        }
    }

    // Binary or <val> || <val>
    pub fn or(self, right: Value) -> Value {
        if bool::from(&self) {
            self
        } else {
            right
        }
    }

    // Binary and <val> && <val>
    pub fn and(self, right: Value) -> Value {
        if !bool::from(&self) {
            self
        } else {
            right
        }
    }

    // <val> != <val>
    pub fn ne(self, right: Value) -> Value {
        Value::Bool(self != right)
    }

    // <val> == <val>
    pub fn eq(self, right: Value) -> Value {
        Value::Bool(self == right)
    }

    // <val> < <val>
    pub fn lt(self, right: Value) -> Value {
        Value::Bool(self < right)
    }

    // <val> > <val>
    pub fn gt(self, right: Value) -> Value {
        Value::Bool(self > right)
    }

    // <val> <= <val>
    pub fn le(self, right: Value) -> Value {
        Value::Bool(self <= right)
    }

    // <val> >= <val>
    pub fn ge(self, right: Value) -> Value {
        Value::Bool(self >= right)
    }
}
