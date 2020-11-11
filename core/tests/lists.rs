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

    // NOTE assertions cannot be made at each statement, because they
    // are all executed first and then returned, since lists are passed
    // by reference, all assertions except the last one will fail since
    // it will already be how it is at the end
    assert_eq!(
        execute(
            "let l = [123, 123.1, 9812]
            l[0] = 50
            l[-1] = 'hello world'"
        )
        .unwrap(),
        v::l(vec![v::i(50), v::f(123.1), v::s("hello world"),]),
    );

    assert_eq!(
        execute("[123, 1231, 9812][0..23-24]").unwrap(),
        v::l(vec![v::i(123), v::i(1231)])
    );
}

#[test]
fn nested_index() {
    assert_eq!(execute("[123, [1231], 9812][1][0]").unwrap(), v::i(1231));
    assert_eq!(
        execute("[123, [[1231], 9812]][-1][-2][0]").unwrap(),
        v::i(1231)
    );

    assert_eq!(
        execute(
            "let l = [123, [123.1, [9812]]]
            l[0] = 50
            l[1][0] += 50
            l[-1][1][-1] = 'hello world'"
        )
        .unwrap(),
        v::l(vec![
            v::i(50),
            v::l(vec![v::f(173.1), v::l(vec![v::s("hello world")])]),
        ]),
    );
}
