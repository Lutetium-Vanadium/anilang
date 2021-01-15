mod common;
use common::*;

#[test]
fn functions_no_args() {
    assert_eq!(
        execute(
            "fn f() {
                1
            }
            f()"
        )
        .unwrap(),
        v::i(1)
    );

    assert!(execute(
        "fn f() {}
         f()"
    )
    .unwrap()
    .is_null());
}

#[test]
fn functions_with_args() {
    assert_eq!(
        execute(
            "fn add(a, b) {
                a + b
            }
            add(0, 2)"
        )
        .unwrap(),
        v::i(2)
    );

    assert_eq!(
        execute(
            "fn add2(a, b) {
                a + b*2
            }
            add2(2, 5)"
        )
        .unwrap(),
        v::i(12)
    );
}

#[test]
fn functions_declaration_returns_function() {
    assert_eq!(
        execute(
            "let f1 = fn f2() {}
            f1 == f2"
        )
        .unwrap(),
        v::b(true)
    );
}

#[test]
fn functions_recurse() {
    assert_eq!(
        execute(
            "fn fact(a) {
                if a == 2 {
                    2
                } else {
                    a * fact(a-1)
                }
            }
            fact(5)"
        )
        .unwrap(),
        v::i(120),
    );
}

#[test]
fn functions_use_proper_scope() {
    assert_eq!(
        execute(
            "fn create_fact() {
            let mem = [1, 1, 2, 0, 0, 0, 0, 0, 0, 0]

            fn fact(a) {
                if mem[a] > 0{
                    mem[a]
                } else {
                    mem[a] = a * fact(a-1)
                    mem[a]
                }
            }
        }

        let fact = create_fact()
        fact(5)

        fact(9)"
        )
        .unwrap(),
        v::i(362880)
    );

    assert_eq!(
        execute(
            "fn outer(a) {
                fn inner() {
                    a + 1
                }
            }

            let five = outer(4)
            let four = outer(3)
            five() + four()"
        )
        .unwrap(),
        v::i(9)
    );

    assert_eq!(
        execute(
            "let a = 100
            fn f(a) {
                a + 123
            }
            f(10)"
        )
        .unwrap(),
        v::i(133)
    );
}

#[test]
fn anonymous_functions() {
    assert_eq!(execute("(fn(a, b) { a + b })(1, 2)").unwrap(), v::i(3));
    assert_eq!(
        execute(
            "let add = fn(a, b) { a + b }
            add(1, 2)"
        )
        .unwrap(),
        v::i(3)
    );
}
