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

// Currently broken since functions do not have access to the parent scope where they were declared
//
// #[test]
// fn functions_recurse() {
//     assert_eq!(
//         execute(
//             "fn fact(a) {
//                 if a == 2 {
//                     2
//                 } else {
//                     a * fact(a-1)
//                 }
//             }
//             fact(5)"
//         )
//         .unwrap(),
//         v::i(120),
//     );
// }
