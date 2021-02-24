mod common;
use common::*;

#[test]
fn if_block() {
    assert_eq!(
        execute(
            "if true {
                1
            } else if true {
                2
            } else {
                3
            }"
        )
        .unwrap(),
        v::i(1)
    );

    assert_eq!(
        execute(
            "if false {
                1
            } else {
                2
            }"
        )
        .unwrap(),
        v::i(2)
    );

    assert_eq!(
        execute(
            "if false {
                1
            } else if true {
                2
            } else {
                3
            }"
        )
        .unwrap(),
        v::i(2)
    );

    assert_eq!(
        execute(
            "if false {
                1
            } else if false {
                2
            } else {
                3
            }"
        )
        .unwrap(),
        v::i(3)
    );
}

#[test]
fn while_loops() {
    assert_eq!(
        execute(
            "let r = 0
            let i = 1
            while i <= 100 {
                r += i
                i += 1
            }
            r"
        )
        .unwrap(),
        v::i(5050)
    );

    assert_eq!(
        execute(
            "let a = 1
            let c = 0
            while a < 1024 {
                a *= 2
                c += 1
            }
            c"
        )
        .unwrap(),
        v::i(10)
    );
}
