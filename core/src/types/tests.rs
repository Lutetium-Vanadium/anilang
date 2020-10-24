use super::*;
use crate::value::List;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn detect_implicit_cast() {
    assert_eq!((Type::Int).cast_type(&Type::Int), Cast::Implicit(Type::Int));
    assert_eq!(
        (Type::Float).cast_type(&Type::Float),
        Cast::Implicit(Type::Float)
    );
    assert_eq!(
        (Type::List).cast_type(&Type::List),
        Cast::Implicit(Type::List)
    );
    assert_eq!(
        (Type::String).cast_type(&Type::String),
        Cast::Implicit(Type::String)
    );
    assert_eq!(
        (Type::Bool).cast_type(&Type::Bool),
        Cast::Implicit(Type::Bool)
    );
    assert_eq!(
        (Type::Null).cast_type(&Type::Null),
        Cast::Implicit(Type::Null)
    );
    assert_eq!(
        (Type::Int).cast_type(&Type::Float),
        Cast::Implicit(Type::Float)
    );
    assert_eq!(
        (Type::Float).cast_type(&Type::Int),
        Cast::Implicit(Type::Float)
    );
}

#[test]
fn detect_explicit_cast() {
    assert_eq!((Type::Int).cast_type(&Type::Bool), Cast::Explicit);
    assert_eq!((Type::Int).cast_type(&Type::String), Cast::Explicit);
    assert_eq!((Type::Int).cast_type(&Type::List), Cast::Explicit);
    assert_eq!((Type::Int).cast_type(&Type::Function), Cast::Explicit);
    assert_eq!((Type::Int).cast_type(&Type::Null), Cast::Explicit);

    assert_eq!((Type::Bool).cast_type(&Type::Int), Cast::Explicit);
    assert_eq!((Type::String).cast_type(&Type::Int), Cast::Explicit);
    assert_eq!((Type::List).cast_type(&Type::Int), Cast::Explicit);
    assert_eq!((Type::Function).cast_type(&Type::Int), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::Int), Cast::Explicit);

    assert_eq!((Type::Float).cast_type(&Type::Bool), Cast::Explicit);
    assert_eq!((Type::Float).cast_type(&Type::String), Cast::Explicit);
    assert_eq!((Type::Float).cast_type(&Type::List), Cast::Explicit);
    assert_eq!((Type::Float).cast_type(&Type::Function), Cast::Explicit);
    assert_eq!((Type::Float).cast_type(&Type::Null), Cast::Explicit);

    assert_eq!((Type::Bool).cast_type(&Type::Float), Cast::Explicit);
    assert_eq!((Type::String).cast_type(&Type::Float), Cast::Explicit);
    assert_eq!((Type::List).cast_type(&Type::Float), Cast::Explicit);
    assert_eq!((Type::Function).cast_type(&Type::Float), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::Float), Cast::Explicit);

    assert_eq!((Type::Bool).cast_type(&Type::String), Cast::Explicit);
    assert_eq!((Type::Function).cast_type(&Type::String), Cast::Explicit);
    assert_eq!((Type::List).cast_type(&Type::String), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::String), Cast::Explicit);

    assert_eq!((Type::String).cast_type(&Type::Bool), Cast::Explicit);
    assert_eq!((Type::String).cast_type(&Type::Function), Cast::Explicit);
    assert_eq!((Type::String).cast_type(&Type::List), Cast::Explicit);
    assert_eq!((Type::String).cast_type(&Type::Null), Cast::Explicit);

    assert_eq!((Type::Bool).cast_type(&Type::Function), Cast::Explicit);
    assert_eq!((Type::Bool).cast_type(&Type::List), Cast::Explicit);
    assert_eq!((Type::Bool).cast_type(&Type::Null), Cast::Explicit);

    assert_eq!((Type::Function).cast_type(&Type::Bool), Cast::Explicit);
    assert_eq!((Type::List).cast_type(&Type::Bool), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::Bool), Cast::Explicit);

    assert_eq!((Type::Function).cast_type(&Type::List), Cast::Explicit);
    assert_eq!((Type::Function).cast_type(&Type::Null), Cast::Explicit);

    assert_eq!((Type::List).cast_type(&Type::Function), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::Function), Cast::Explicit);

    assert_eq!((Type::List).cast_type(&Type::Null), Cast::Explicit);
    assert_eq!((Type::Null).cast_type(&Type::List), Cast::Explicit);
}

#[test]
fn bitflag_type_to_string() {
    assert_eq!(&(Type::Int | Type::Float).to_string(), "int | float");
    assert_eq!(&(Type::String | Type::List).to_string(), "string | list");
    assert_eq!(&(Type::Bool | Type::Null).to_string(), "bool | null");
    // It gives the types in order of enum declaration, and not the order in which they are
    // instantiated
    assert_eq!(
        &(Type::Function | Type::Float | Type::String).to_string(),
        "float | string | function"
    );
    assert_eq!(&BitFlags::from(Type::Bool).to_string(), "bool");
}

fn i(i: i64) -> Value {
    Value::Int(i)
}
fn f(f: f64) -> Value {
    Value::Float(f)
}
fn b(b: bool) -> Value {
    Value::Bool(b)
}
fn s(s: &str) -> Value {
    Value::String(Rc::new(RefCell::new(s.to_owned())))
}
fn l(l: List) -> Value {
    Value::List(Rc::new(RefCell::new(l)))
}
fn func() -> Value {
    Value::Function(Rc::new(Default::default()))
}
fn n() -> Value {
    Value::Null
}

#[test]
fn get_correct_type() {
    assert_eq!(i(0).type_(), Type::Int);
    assert_eq!(f(0.0).type_(), Type::Float);
    assert_eq!(s("hello").type_(), Type::String);
    assert_eq!(l(vec![i(0), f(2.0), b(true)]).type_(), Type::List);
    assert_eq!(b(true).type_(), Type::Bool);
    assert_eq!(func().type_(), Type::Function);
    assert_eq!(n().type_(), Type::Null);
}

#[test]
fn try_cast_success() {
    assert_eq!(i(0).try_cast(Type::Float).unwrap(), f(0.0));
    assert_eq!(f(0.0).try_cast(Type::Int).unwrap(), f(0.0));

    assert_eq!(i(0).try_cast(Type::Int).unwrap(), i(0));
    assert_eq!(f(0.0).try_cast(Type::Float).unwrap(), f(0.0));
    assert_eq!(s("hello").try_cast(Type::String).unwrap(), s("hello"));
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).try_cast(Type::List).unwrap(),
        l(vec![i(0), f(2.0), b(true)])
    );
    assert_eq!(b(true).try_cast(Type::Bool).unwrap(), b(true));
    assert!(n().try_cast(Type::Null).unwrap().is_null());
}

#[test]
fn try_cast_fail() {
    assert_eq!(i(0).try_cast(Type::Bool).err().unwrap(), Cast::Explicit);
    assert_eq!(f(0.0).try_cast(Type::String).err().unwrap(), Cast::Explicit);
    assert_eq!(b(true).try_cast(Type::List).err().unwrap(), Cast::Explicit);
    assert_eq!(
        f(0.0).try_cast(Type::Function).err().unwrap(),
        Cast::Explicit
    );
    assert_eq!(
        b(true).try_cast(Type::String).err().unwrap(),
        Cast::Explicit
    );
    assert_eq!(
        s("hello").try_cast(Type::Float).err().unwrap(),
        Cast::Explicit
    );
    assert_eq!(func().try_cast(Type::Float).err().unwrap(), Cast::Explicit);
    assert_eq!(n().try_cast(Type::Int).err().unwrap(), Cast::Explicit);
}
