mod common;
use common::*;

#[test]
fn declare_variable() {
    assert_eq!(execute("let a = 123").unwrap(), v::i(123));
    assert_eq!(execute("let a = 12.3").unwrap(), v::f(12.3));
    assert_eq!(execute("let a = '123'").unwrap(), v::s("123"));
    assert_eq!(execute("let a = true").unwrap(), v::b(true));
}

#[test]
fn use_variable() {
    assert!(execute_many(vec![
        "let a = true",
        "let b = false",
        "a || a",
        "a || b",
        "b || a",
        "b || b",
        "a && a",
        "a && b",
        "b && a",
        "b && b",
        "a &&= false",
        "b ||= true",
        "a == b",
    ])
    .iter()
    .map(|r| r.as_ref().unwrap())
    .eq(vec![
        v::b(true),
        v::b(false),
        v::b(true),
        v::b(true),
        v::b(true),
        v::b(false),
        v::b(true),
        v::b(false),
        v::b(false),
        v::b(false),
        v::b(false),
        v::b(true),
        v::b(false)
    ]
    .iter()));

    assert!(execute_many(vec![
        "let a = 23",
        "a += 46",
        "let b = 41",
        "b -= 12",
        "let c = 13",
        "c *= 11",
        "let d = 23",
        "d /= 7",
        "let e = 23",
        "e %= 7",
    ])
    .iter()
    .map(|r| r.as_ref().unwrap())
    .eq(vec![
        v::i(23),
        v::i(69),
        v::i(41),
        v::i(29),
        v::i(13),
        v::i(143),
        v::i(23),
        v::i(3),
        v::i(23),
        v::i(2),
    ]
    .iter()));
}

#[test]
fn variable_scoping() {
    assert_eq!(
        execute(
            "let a = 23
            {
                a += 27
            }"
        )
        .unwrap(),
        v::i(50)
    );

    assert_eq!(
        execute(
            "let a = 23
            {
                let a = 4
                a + 4
            }"
        )
        .unwrap(),
        v::i(8)
    );

    assert_eq!(
        execute(
            "let a = 23
            {
                let a = true
                a && true
            }"
        )
        .unwrap(),
        v::b(true)
    );

    assert_eq!(
        execute(
            "let a = 23
            {
                let a = true
            }
            a"
        )
        .unwrap(),
        v::i(23)
    );

    assert_eq!(
        execute(
            "let a = 23
            {
                let a = 75
                {
                    a += 23
                }
                a
            }"
        )
        .unwrap(),
        v::i(98)
    );
}
