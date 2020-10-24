mod common;
use common::*;

#[test]
fn concat_lists() {
    assert_eq!(
        execute("[12, 12.3, 'string'] + [false, 'string']").unwrap(),
        v::l(vec![
            v::i(12),
            v::f(12.3),
            v::s("string"),
            v::b(false),
            v::s("string")
        ]),
    );

    assert_eq!(
        execute("[12, 12.3, 'string'] + [[false, 'string']]").unwrap(),
        v::l(vec![
            v::i(12),
            v::f(12.3),
            v::s("string"),
            v::l(vec![v::b(false), v::s("string")])
        ]),
    );
}

#[test]
fn index_lists() {
    assert_eq!(execute("[123, 1231, 9812][2]").unwrap(), v::i(9812));
    assert_eq!(execute("[123, 1231, 9812][-2]").unwrap(), v::i(1231));
}
