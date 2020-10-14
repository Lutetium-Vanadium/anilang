use super::*;

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
    Value::String(s.to_owned())
}
fn n() -> Value {
    Value::Null
}

fn err_it(t: Type) -> Result<Value> {
    Err(ErrorKind::IncorrectType {
        got: t,
        expected: Type::Int | Type::Float,
    })
}

fn err_ir(g: Type, e: BitFlags<Type>) -> Result<Value> {
    Err(ErrorKind::IncorrectRightType {
        got: g,
        expected: e,
    })
}

fn err_eb(got: i64) -> Result<Value> {
    Err(ErrorKind::OutOfBounds {
        got,
        start: 0,
        end: u32::MAX as i64,
    })
}

impl Value {
    pub fn is_null(&self) -> bool {
        if let Type::Null = self.type_() {
            true
        } else {
            false
        }
    }
}

#[test]
fn unary_plus_valid() {
    assert_eq!(i(10).plus(), Ok(i(10)));
    assert_eq!(f(10.0).plus(), Ok(f(10.0)));
}

#[test]
fn unary_plus_invalid() {
    assert_eq!(b(true).plus(), err_it(Type::Bool));
    assert_eq!(s("a").plus(), err_it(Type::String));
    assert_eq!(n().plus(), err_it(Type::Null));
}

#[test]
fn unary_minus_valid() {
    assert_eq!(i(10).minus(), Ok(i(-10)));
    assert_eq!(f(10.0).minus(), Ok(f(-10.0)));
}

#[test]
fn unary_minus_invalid() {
    assert_eq!(b(true).minus(), err_it(Type::Bool));
    assert_eq!(s("a").minus(), err_it(Type::String));
    assert_eq!(n().minus(), err_it(Type::Null));
}

#[test]
fn unary_not() {
    assert_eq!(bool::from(i(10).not()), false);
    assert_eq!(bool::from(i(0).not()), true);

    assert_eq!(bool::from(f(10.0).not()), false);
    assert_eq!(bool::from(f(0.0).not()), true);

    assert_eq!(bool::from(b(true).not()), false);
    assert_eq!(bool::from(b(false).not()), true);

    assert_eq!(bool::from(s("s").not()), false);
    assert_eq!(bool::from(s("").not()), true);

    assert_eq!(bool::from(n().not()), true);
}

#[test]
fn binary_add_valid() {
    assert_eq!(i(1).add(i(2)), Ok(i(3)));
    assert_eq!(i(1).add(f(2.0)), Ok(f(3.0)));
    assert_eq!(f(1.0).add(i(2)), Ok(f(3.0)));
    assert_eq!(f(1.0).add(f(2.0)), Ok(f(3.0)));
    assert_eq!(s("hello").add(s("world")), Ok(s("helloworld")));
}

#[test]
fn binary_add_invalid() {
    assert_eq!(b(true).add(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").add(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().add(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).add(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).add(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).add(n()), err_ir(Type::Null, Type::Int.into()));
}

#[test]
fn binary_sub_valid() {
    assert_eq!(i(1).sub(i(2)), Ok(i(-1)));
    assert_eq!(i(1).sub(f(2.0)), Ok(f(-1.0)));
    assert_eq!(f(1.0).sub(i(2)), Ok(f(-1.0)));
    assert_eq!(f(1.0).sub(f(2.0)), Ok(f(-1.0)));
}

#[test]
fn binary_sub_invalid() {
    assert_eq!(b(true).sub(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").sub(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().sub(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).sub(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).sub(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).sub(n()), err_ir(Type::Null, Type::Int.into()));
}

#[test]
fn binary_mult_valid() {
    assert_eq!(i(1).mult(i(2)), Ok(i(2)));
    assert_eq!(i(1).mult(f(2.0)), Ok(f(2.0)));
    assert_eq!(f(1.0).mult(i(2)), Ok(f(2.0)));
    assert_eq!(f(1.0).mult(f(2.0)), Ok(f(2.0)));
}

#[test]
fn binary_mult_invalid() {
    assert_eq!(b(true).mult(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").mult(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().mult(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).mult(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).mult(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).mult(n()), err_ir(Type::Null, Type::Int.into()));
}

#[test]
fn binary_div_valid() {
    assert_eq!(i(1).div(i(2)), Ok(i(0)));
    assert_eq!(i(1).div(f(2.0)), Ok(f(0.5)));
    assert_eq!(f(1.0).div(i(2)), Ok(f(0.5)));
    assert_eq!(f(1.0).div(f(2.0)), Ok(f(0.5)));
}

#[test]
fn binary_div_invalid() {
    assert_eq!(b(true).div(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").div(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().div(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).div(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).div(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).div(n()), err_ir(Type::Null, Type::Int.into()));

    assert_eq!(i(10).div(i(0)), Err(ErrorKind::DivideByZero));
}

#[test]
fn binary_mod_valid() {
    assert_eq!(i(19).modulo(i(12)), Ok(i(7)));
    assert_eq!(i(19).modulo(f(12.0)), Ok(f(7.0)));
    assert_eq!(f(19.0).modulo(i(12)), Ok(f(7.0)));
    assert_eq!(f(19.0).modulo(f(12.0)), Ok(f(7.0)));
}

#[test]
fn binary_mod_invalid() {
    assert_eq!(b(true).modulo(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").modulo(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().modulo(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).modulo(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).modulo(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).modulo(n()), err_ir(Type::Null, Type::Int.into()));

    assert_eq!(i(10).modulo(i(0)), Err(ErrorKind::DivideByZero));
}

#[test]
fn binary_pow_valid() {
    assert_eq!(i(2).pow(i(12)), Ok(i(4096)));
    assert_eq!(i(2).pow(f(12.0)), Ok(f(4096.0)));
    assert_eq!(f(2.0).pow(i(12)), Ok(f(4096.0)));
    assert_eq!(f(2.0).pow(f(12.0)), Ok(f(4096.0)));
}

#[test]
fn binary_pow_invalid() {
    assert_eq!(b(true).pow(i(10)), err_ir(Type::Int, Type::Bool.into()));
    assert_eq!(s("a").pow(i(10)), err_ir(Type::Int, Type::String.into()));
    assert_eq!(n().pow(i(10)), err_ir(Type::Int, Type::Null.into()));

    assert_eq!(i(10).pow(b(true)), err_ir(Type::Bool, Type::Int.into()));
    assert_eq!(i(10).pow(s("a")), err_ir(Type::String, Type::Int.into()));
    assert_eq!(i(10).pow(n()), err_ir(Type::Null, Type::Int.into()));

    assert_eq!(i(10).pow(i(-10)), err_eb(-10));
    assert_eq!(
        i(10).pow(i(u32::MAX as i64 + 5)),
        err_eb(u32::MAX as i64 + 5)
    );
}

#[test]
fn binary_or() {
    assert_eq!(i(21).or(i(2)), i(21));
    assert_eq!(i(0).or(i(2)), i(2));

    assert_eq!(f(12.0).or(f(2.123)), f(12.0));
    assert_eq!(f(0.0).or(f(2.123)), f(2.123));

    assert_eq!(s("hello").or(s("world")), s("hello"));
    assert_eq!(s("").or(s("world")), s("world"));

    assert_eq!(n().or(i(2)), i(2));
    assert!(n().or(n()).is_null());

    assert_eq!(b(true).or(b(true)), b(true));
    assert_eq!(b(false).or(b(true)), b(true));
    assert_eq!(b(true).or(b(false)), b(true));
    assert_eq!(b(false).or(b(false)), b(false));
}

#[test]
fn binary_and() {
    assert_eq!(i(21).and(i(2)), i(2));
    assert_eq!(i(0).and(i(2)), i(0));

    assert_eq!(f(12.0).and(f(2.123)), f(2.123));
    assert_eq!(f(0.0).and(f(2.123)), f(0.0));

    assert_eq!(s("hello").and(s("world")), s("world"));
    assert_eq!(s("").and(s("world")), s(""));

    assert!(n().and(i(2)).is_null());
    assert!(n().and(n()).is_null());

    assert_eq!(b(true).and(b(true)), b(true));
    assert_eq!(b(false).and(b(true)), b(false));
    assert_eq!(b(true).and(b(false)), b(false));
    assert_eq!(b(false).and(b(false)), b(false));
}

#[test]
fn binary_eq() {
    assert_eq!(i(1), i(1));
    assert_eq!(f(1.0), i(1));
    assert_eq!(i(1), f(1.0));
    assert_eq!(f(1.0), f(1.0));
    assert_eq!(s("hello"), s("hello"));
    assert_eq!(b(true), b(true));
    assert_eq!(b(false), b(false));
}

#[test]
fn binary_ne() {
    assert_ne!(i(1), i(2));
    assert_ne!(f(1.0), i(2));
    assert_ne!(i(1), f(1.1));
    assert_ne!(f(1.0), f(2.0));
    assert_ne!(s("hello"), s("world"));
    assert_ne!(b(true), b(false));
    assert_ne!(b(false), b(true));
    assert_ne!(n(), n());
}

#[test]
fn binary_lt() {
    assert!(i(1) < i(2));
    assert!(f(1.0) < i(2));
    assert!(i(1) < f(1.1));
    assert!(f(1.0) < f(2.0));
    assert!(s("hello") < s("world"));
    assert!(b(false) < b(true));
}

#[test]
fn binary_gt() {
    assert!(i(3) > i(2));
    assert!(f(3.0) > i(2));
    assert!(i(3) > f(1.1));
    assert!(f(3.0) > f(2.0));
    assert!(s("xyz") > s("world"));
    assert!(b(true) > b(false));
}

#[test]
fn binary_le() {
    assert!(i(3) <= i(4));
    assert!(i(2) <= i(2));
    assert!(f(3.0) <= i(4));
    assert!(f(2.0) <= i(2));
    assert!(i(3) <= f(3.3));
    assert!(i(1) <= f(1.0));
    assert!(f(3.0) <= f(3.2));
    assert!(f(2.0) <= f(2.0));
    assert!(s("abc") <= s("world"));
    assert!(s("world") <= s("world"));
    assert!(b(true) <= b(true));
    assert!(b(false) <= b(true));
}

#[test]
fn binary_ge() {
    assert!(i(3) >= i(2));
    assert!(i(2) >= i(2));
    assert!(f(3.0) >= i(2));
    assert!(f(2.0) >= i(2));
    assert!(i(3) >= f(1.1));
    assert!(i(1) >= f(1.0));
    assert!(f(3.0) >= f(2.0));
    assert!(f(2.0) >= f(2.0));
    assert!(s("xyz") >= s("world"));
    assert!(s("world") >= s("world"));
    assert!(b(true) >= b(false));
    assert!(b(false) >= b(false));
}
