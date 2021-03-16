use super::*;
use crate::test_helpers::*;

fn err_ior(index: i64, len: i64) -> Result<Value> {
    Err(ErrorKind::IndexOutOfRange { index, len })
}

fn test_invalid_prop(v: Value, prop: &str) {
    let p = s(prop);
    let prop_str = Gc::clone(&p.to_gc_str());
    assert_eq!(
        v.clone().get_at(p),
        Err(ErrorKind::InvalidProperty {
            val: v,
            property: prop_str
        })
    );
}

fn test_invalid_prop_set(v: Value, prop: &str, set: Value) {
    let p = s(prop);
    let prop_str = Gc::clone(&p.to_gc_str());
    assert_eq!(
        v.clone().set_at(p, set),
        Err(ErrorKind::InvalidProperty {
            val: v,
            property: prop_str
        })
    );
}

fn test_readonly_prop(v: Value, prop: &str, set: Value) {
    let p = s(prop);
    let prop_str = Gc::clone(&p.to_gc_str());
    assert_eq!(
        v.clone().set_at(p, set),
        Err(ErrorKind::ReadonlyProperty {
            val: v,
            property: prop_str
        })
    );
}

#[test]
fn indexable_valid() {
    assert!(s("string").indexable(Type::Int));
    assert!(l(vec![]).indexable(Type::Int));
    assert!(s("string").indexable(Type::String));
    assert!(l(vec![]).indexable(Type::String));
    assert!(o(vec![]).indexable(Type::String));
    assert!(r(0, 1).indexable(Type::String));
    assert!(func().indexable(Type::String));
}

#[test]
fn indexable_invalid() {
    let values = [
        i(0),
        f(0.0),
        b(false),
        n(),
        r(0, 1),
        func(),
        o(vec![]),
        s("string"),
        l(vec![]),
    ];

    for value in &values[..] {
        assert!(!value.indexable(Type::Float));
        assert!(!value.indexable(Type::Bool));
        assert!(!value.indexable(Type::Function));
        assert!(!value.indexable(Type::Object));
        assert!(!value.indexable(Type::List));
        assert!(!value.indexable(Type::Null));
    }

    for value in &values[..7] {
        assert!(!value.indexable(Type::Int));
        assert!(!value.indexable(Type::Range));
    }

    for value in &values[..4] {
        assert!(!value.indexable(Type::String));
    }
}

#[test]
fn get_at_valid() {
    assert_eq!(s("string").get_at(i(0)).unwrap().to_ref_str().as_str(), "s");
    assert_eq!(
        s("string").get_at(i(-2)).unwrap().to_ref_str().as_str(),
        "n"
    );
    assert_eq!(
        s("string").get_at(r(1, 4)).unwrap().to_ref_str().as_str(),
        "tri"
    );
    assert_eq!(s("string").get_at(s("len")).unwrap(), i(6));

    assert_eq!(l(vec![i(0), f(2.0), b(true)]).get_at(i(0)).unwrap(), i(0));
    assert_eq!(l(vec![i(0), f(2.0), b(true)]).get_at(i(-3)).unwrap(), i(0));
    assert_eq!(
        l(vec![i(0), f(2.0), b(true), b(false)])
            .get_at(r(1, -1))
            .unwrap(),
        l(vec![f(2.0), b(true)])
    );
    assert_eq!(
        l(vec![i(0), f(2.0), b(true), b(false)])
            .get_at(s("len"))
            .unwrap(),
        i(4)
    );
    assert!(l(vec![]).get_at(s("push")).is_ok());
    assert!(l(vec![]).get_at(s("pop")).is_ok());

    assert_eq!(o(vec![("key", i(0))]).get_at(s("key")).unwrap(), i(0));
    assert_eq!(
        o(vec![("unicode└", b(false))])
            .get_at(s("unicode└"))
            .unwrap(),
        b(false)
    );

    assert_eq!(r(0, 1).get_at(s("start")).unwrap(), i(0));
    assert_eq!(r(0, 1).get_at(s("end")).unwrap(), i(1));

    let f = func();
    assert_eq!(f.clone().get_at(s("call")).unwrap(), f);
}

#[test]
fn get_at_invalid() {
    assert_eq!(s("string").get_at(i(7)), err_ior(7, 6));
    assert_eq!(s("string").get_at(i(-12)), err_ior(-12, 6));
    test_invalid_prop(s("string"), "unknown_property");

    assert_eq!(l(vec![i(0), f(2.0), b(true)]).get_at(i(7)), err_ior(7, 3));
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).get_at(i(-12)),
        err_ior(-12, 3)
    );
    test_invalid_prop(l(vec![i(0), f(2.0), b(true)]), "unknown_property");

    test_invalid_prop(o(vec![]), "unknown_property");

    test_invalid_prop(r(0, 1), "unknown_property");
    test_invalid_prop(func(), "unknown_property");
}

#[test]
fn set_at_valid() {
    assert_eq!(
        s("string")
            .set_at(i(0), s("f"))
            .unwrap()
            .to_ref_str()
            .as_str(),
        "ftring"
    );
    assert_eq!(
        s("string")
            .set_at(i(-2), s("f"))
            .unwrap()
            .to_ref_str()
            .as_str(),
        "strifg"
    );
    assert_eq!(
        s("string")
            .set_at(r(2, -2), s("zz"))
            .unwrap()
            .to_ref_str()
            .as_str(),
        "stzzng"
    );

    assert_eq!(
        l(vec![i(0), f(2.0), b(true)])
            .set_at(i(0), s("string"))
            .unwrap()
            .to_ref_list()[..],
        [s("string"), f(2.0), b(true)]
    );
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)])
            .set_at(i(-2), s("string"))
            .unwrap()
            .to_ref_list()[..],
        [i(0), s("string"), b(true)]
    );

    assert_eq!(
        l(vec![i(0), f(2.0), b(true)])
            .set_at(r(0, 2), l(vec![s("string")]))
            .unwrap()
            .to_ref_list()[..],
        [s("string"), b(true)]
    );
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)])
            .set_at(r(1, 3), l(vec![b(true), s("string")]))
            .unwrap()
            .to_ref_list()[..],
        [i(0), b(true), s("string")]
    );
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)])
            .set_at(r(1, 2), l(vec![b(false), s("string")]))
            .unwrap()
            .to_ref_list()[..],
        [i(0), b(false), s("string"), b(true)]
    );

    let obj = o(vec![("already_exists", n())]);
    assert_eq!(obj.clone().set_at(s("already_exists"), i(1)).unwrap(), i(1));
    assert_eq!(obj.clone().set_at(s("doesnt_exist"), i(2)).unwrap(), i(2));

    let obj = obj.to_ref_obj();

    assert_eq!(*obj.get("already_exists").unwrap(), i(1));
    assert_eq!(*obj.get("doesnt_exist").unwrap(), i(2));
}

#[test]
fn set_at_invalid() {
    assert_eq!(s("string").set_at(i(7), s("")), err_ior(7, 6));
    assert_eq!(s("string").set_at(i(-12), s("")), err_ior(-12, 6));
    test_invalid_prop_set(s("string"), "unknown_property", n());

    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).set_at(i(7), s("")),
        err_ior(7, 3)
    );
    assert_eq!(
        l(vec![i(0), f(2.0), b(true)]).set_at(i(-12), s("")),
        err_ior(-12, 3)
    );
    test_invalid_prop_set(l(vec![i(0), f(2.0), b(true)]), "unknown_property", n());

    test_invalid_prop_set(r(0, 1), "unknown_property", n());
    test_invalid_prop_set(func(), "unknown_property", n());

    test_readonly_prop(s("string"), "len", n());
    test_readonly_prop(l(vec![]), "len", n());
    test_readonly_prop(l(vec![]), "push", n());
    test_readonly_prop(l(vec![]), "pop", n());
    test_readonly_prop(r(0, 1), "start", n());
    test_readonly_prop(r(0, 1), "end", n());
    test_readonly_prop(func(), "call", n());
}
