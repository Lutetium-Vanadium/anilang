use super::*;
use crate::test_helpers::*;

fn err_it(t: Type) -> Result<Value> {
    err_ite(t, Type::Int | Type::Float)
}

fn err_ite(t: Type, e: BitFlags<Type>) -> Result<Value> {
    Err(ErrorKind::IncorrectType {
        got: t,
        expected: e,
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

#[test]
fn unary_plus_valid() {
    assert_eq!(i(10).plus(), Ok(i(10)));
    assert_eq!(f(10.0).plus(), Ok(f(10.0)));
}

#[test]
fn unary_plus_invalid() {
    assert_eq!(b(true).plus(), err_it(Type::Bool));
    assert_eq!(s("a").plus(), err_it(Type::String));
    assert_eq!(l(vec![]).plus(), err_it(Type::List));
    assert_eq!(o(vec![]).plus(), err_it(Type::Object));
    assert_eq!(r(0, 1).plus(), err_it(Type::Range));
    assert_eq!(func().plus(), err_it(Type::Function));
    assert_eq!(n().plus(), err_it(Type::Null));
}

#[test]
fn unary_minus_valid() {
    assert_eq!(-i(10), Ok(i(-10)));
    assert_eq!(-f(10.0), Ok(f(-10.0)));
}

#[test]
fn unary_minus_invalid() {
    assert_eq!(-b(true), err_it(Type::Bool));
    assert_eq!(-s("a"), err_it(Type::String));
    assert_eq!(-l(vec![]), err_it(Type::List));
    assert_eq!(-o(vec![]), err_it(Type::Object));
    assert_eq!(-r(0, 1), err_it(Type::Range));
    assert_eq!(-func(), err_it(Type::Function));
    assert_eq!(-n(), err_it(Type::Null));
}

#[test]
fn unary_not() {
    assert_eq!(bool::from(!i(10)), false);
    assert_eq!(bool::from(!i(0)), true);

    assert_eq!(bool::from(!f(10.0)), false);
    assert_eq!(bool::from(!f(0.0)), true);

    assert_eq!(bool::from(!b(true)), false);
    assert_eq!(bool::from(!b(false)), true);

    assert_eq!(bool::from(!s("s")), false);
    assert_eq!(bool::from(!s("")), true);

    assert_eq!(bool::from(!l(vec![i(0)])), false);
    assert_eq!(bool::from(!l(vec![])), true);

    assert_eq!(bool::from(!o(vec![("key", i(0))])), false);
    assert_eq!(bool::from(!o(vec![])), true);

    assert_eq!(bool::from(!r(0, 1)), false);
    assert_eq!(bool::from(!r(0, 0)), true);

    assert_eq!(bool::from(!func()), false);
    assert_eq!(bool::from(!n()), true);
}

#[test]
fn binary_range_valid() {
    assert_eq!(i(1).range_to(i(2)), Ok(r(1, 2)));
    assert_eq!(i(-1).range_to(i(2)), Ok(r(-1, 2)));
}

#[test]
fn binary_range_invalid() {
    assert_eq!(
        f(0.0).range_to(i(10)),
        err_ite(Type::Float, Type::Int.into())
    );

    assert_eq!(
        i(10).range_to(f(0.0)),
        err_ite(Type::Float, Type::Int.into())
    );

    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(
            val.clone().range_to(i(10)),
            err_ir(Type::Int, val_t.clone().into())
        );

        assert_eq!(i(10).range_to(val), err_ir(val_t, Type::Int.into()));
    }
}

#[test]
fn binary_add_valid() {
    assert_eq!(i(1) + i(2), Ok(i(3)));
    assert_eq!(i(1) + f(2.0), Ok(f(3.0)));
    assert_eq!(f(1.0) + i(2), Ok(f(3.0)));
    assert_eq!(f(1.0) + f(2.0), Ok(f(3.0)));
    assert_eq!(
        l(vec![i(0), f(2.0)]) + l(vec![b(true)]),
        Ok(l(vec![i(0), f(2.0), b(true)]))
    );
}

#[test]
fn binary_add_invalid() {
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(val.clone() + i(10), err_ir(Type::Int, val_t.clone().into()));

        assert_eq!(i(10) + val, err_ir(val_t, Type::Int.into()));
    }
}

#[test]
fn binary_sub_valid() {
    assert_eq!(i(1) - i(2), Ok(i(-1)));
    assert_eq!(i(1) - f(2.0), Ok(f(-1.0)));
    assert_eq!(f(1.0) - i(2), Ok(f(-1.0)));
    assert_eq!(f(1.0) - f(2.0), Ok(f(-1.0)));
}

#[test]
fn binary_sub_invalid() {
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(val.clone() - i(10), err_ir(Type::Int, val_t.clone().into()));

        assert_eq!(i(10) - val, err_ir(val_t, Type::Int.into()));
    }
}

#[test]
fn binary_mult_valid() {
    assert_eq!(i(1) * i(2), Ok(i(2)));
    assert_eq!(i(1) * f(2.0), Ok(f(2.0)));
    assert_eq!(f(1.0) * i(2), Ok(f(2.0)));
    assert_eq!(f(1.0) * f(2.0), Ok(f(2.0)));
}

#[test]
fn binary_mult_invalid() {
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(val.clone() * i(10), err_ir(Type::Int, val_t.clone().into()));

        assert_eq!(i(10) * val, err_ir(val_t, Type::Int.into()));
    }
}

#[test]
fn binary_div_valid() {
    assert_eq!(i(1) / i(2), Ok(i(0)));
    assert_eq!(i(1) / f(2.0), Ok(f(0.5)));
    assert_eq!(f(1.0) / i(2), Ok(f(0.5)));
    assert_eq!(f(1.0) / f(2.0), Ok(f(0.5)));
}

#[test]
fn binary_div_invalid() {
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(val.clone() / i(10), err_ir(Type::Int, val_t.clone().into()));

        assert_eq!(i(10) / val, err_ir(val_t, Type::Int.into()));
    }

    assert_eq!(i(10) / i(0), Err(ErrorKind::DivideByZero));
}

#[test]
fn binary_mod_valid() {
    assert_eq!(i(19) % i(12), Ok(i(7)));
    assert_eq!(i(19) % f(12.0), Ok(f(7.0)));
    assert_eq!(f(19.0) % i(12), Ok(f(7.0)));
    assert_eq!(f(19.0) % f(12.0), Ok(f(7.0)));
}

#[test]
fn binary_mod_invalid() {
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(val.clone() % i(10), err_ir(Type::Int, val_t.clone().into()));

        assert_eq!(i(10) % val, err_ir(val_t, Type::Int.into()));
    }

    assert_eq!(i(10) % i(0), Err(ErrorKind::DivideByZero));
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
    let values = vec![
        b(true),
        s("a"),
        l(vec![i(0), f(2.0), b(true)]),
        o(vec![("key", s("value"))]),
        r(0, 1),
        func(),
        n(),
    ];

    for val in values {
        let val_t = val.type_();

        assert_eq!(
            val.clone().pow(i(10)),
            err_ir(Type::Int, val_t.clone().into())
        );

        assert_eq!(i(10).pow(val), err_ir(val_t, Type::Int.into()));
    }

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

    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).or(l(vec![i(2), f(6.0), b(false)])),
        l(vec![i(0), f(2.0), b(true)])
    );
    assert_eq!(
        l(vec![]).or(l(vec![i(0), f(2.0), b(true)])),
        l(vec![i(0), f(2.0), b(true)])
    );

    assert_eq!(
        o(vec![("key", s("value"))]).or(o(vec![])),
        o(vec![("key", s("value"))])
    );
    assert_eq!(
        o(vec![]).or(o(vec![("key", s("value"))])),
        o(vec![("key", s("value"))])
    );

    assert_eq!(r(0, 1).or(r(2, 3)), r(0, 1));
    assert_eq!(r(2, 2).or(r(0, 1)), r(0, 1));

    let f = func();
    assert_eq!(f.clone().or(i(2)), f.clone());
    assert_eq!(b(false).or(f.clone()), f);

    assert_eq!(n().or(i(2)), i(2));
    assert_eq!(n().or(n()), n());

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

    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).and(l(vec![i(0), f(-2.0), b(false)])),
        l(vec![i(0), f(-2.0), b(false)])
    );
    assert_eq!(l(vec![]).and(l(vec![i(0), f(2.0), b(true)])), l(vec![]));

    assert_eq!(r(0, 1).and(r(2, 3)), r(2, 3));
    assert_eq!(r(2, 2).and(r(0, 1)), r(2, 2));

    assert_eq!(func().and(i(2)), i(2));
    assert_eq!(b(false).and(func()), b(false));

    assert_eq!(n().and(i(2)), n());
    assert_eq!(n().and(n()), n());

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
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]),
        l(vec![i(0), f(2.0), b(true)])
    );
    assert_eq!(o(vec![("key", s("value"))]), o(vec![("key", s("value"))]),);
    assert_eq!(r(0, 1), r(0, 1));
    assert_eq!(b(true), b(true));
    assert_eq!(b(false), b(false));
    let f = func();
    assert_eq!(f.clone(), f);
    assert_eq!(n(), n());
}

#[test]
fn binary_ne() {
    assert_ne!(i(1), i(2));
    assert_ne!(f(1.0), i(2));
    assert_ne!(i(1), f(1.1));
    assert_ne!(f(1.0), f(2.0));
    assert_ne!(s("hello"), s("world"));
    assert_ne!(
        l(vec![i(0), f(2.0), b(true)]),
        l(vec![s("world"), f(2.0), b(true)]),
    );
    assert_ne!(o(vec![("key", s("value"))]), o(vec![]),);
    assert_ne!(r(0, 1), r(2, 3));
    assert_ne!(b(true), b(false));
    assert_ne!(b(false), b(true));
    assert_ne!(func(), func());
}

#[test]
fn binary_lt() {
    assert!(i(1) < i(2));
    assert!(f(1.0) < i(2));
    assert!(i(1) < f(1.1));
    assert!(f(1.0) < f(2.0));
    assert!(s("hello") < s("world"));
    assert!(l(vec![i(0), f(8.0), b(true)]) < l(vec![i(2), f(2.0), b(true)]));
    assert!(b(false) < b(true));
}

#[test]
fn binary_gt() {
    assert!(i(3) > i(2));
    assert!(f(3.0) > i(2));
    assert!(i(3) > f(1.1));
    assert!(f(3.0) > f(2.0));
    assert!(s("xyz") > s("world"));
    assert!(l(vec![i(0), f(4.0), b(true)]) > l(vec![i(0), f(2.0), b(true)]));
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
    assert!(l(vec![i(0), f(8.0), b(true)]) <= l(vec![i(2), f(2.0), b(true)]));
    assert!(l(vec![i(0), f(2.0), b(true)]) <= l(vec![i(0), f(2.0), b(true)]));
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
    assert!(l(vec![i(0), f(4.0), b(true)]) >= l(vec![i(0), f(2.0), b(true)]));
    assert!(l(vec![i(0), f(2.0), b(true)]) >= l(vec![i(0), f(2.0), b(true)]));
    assert!(b(true) >= b(false));
    assert!(b(false) >= b(false));
}
