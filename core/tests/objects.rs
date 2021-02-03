mod common;
use common::*;

#[test]
fn index_objects() {
    assert_eq!(execute("{ a: 1231 }['a']").unwrap(), v::i(1231));
    assert_eq!(execute("({ a: 1231 }).a").unwrap(), v::i(1231));

    // NOTE assertions cannot be made at each statement, because they
    // are all executed first and then returned, since lists are passed
    // by reference, all assertions except the last one will fail since
    // it will already be how it is at the end
    assert_eq!(
        execute(
            "let a = 'hello '
            let o = { a, }
            o.b = 'world'
            o.a + o.b"
        )
        .unwrap(),
        v::s("hello world"),
    );
}

#[test]
fn nested_index_on_objects() {
    assert_eq!(execute("({ a: { b: 1 }}).a.b").unwrap(), v::i(1));
    assert_eq!(execute("({ a: { b: 1 }})['a']['b']").unwrap(), v::i(1));

    assert_eq!(
        execute(
            "let a = { b: 1 }
            let o = { a, }
            o.a.b"
        )
        .unwrap(),
        v::i(1),
    );
}

#[test]
fn weird_object_declarations() {
    assert_eq!(
        execute("{ ('a'): 123 }").unwrap(),
        v::o(vec![("a", v::i(123))])
    );

    assert_eq!(execute("({ a() { 123 } }).a()").unwrap(), v::i(123));

    assert_eq!(
        execute(
            "let o = {
                c,                      // Use the c to give c: 'b' (values are evaluated right to left)
                let c = 'b': 'world ',  // Parse `let c = 'b'` as an expression leading to b: 'world '
                 { 'a' }: 'hello ',     // Parse `{ 'a' }` as an expression leading to a: 'hello '
            }
            o.a + o.b + o.c"
        )
        .unwrap(),
        v::s("hello world b"),
    );
}
