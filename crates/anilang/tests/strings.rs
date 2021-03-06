mod common;
use common::*;

#[test]
fn concat_strings() {
    assert_eq!(execute(r#"'Hello' + 'World'"#).unwrap(), v::s("HelloWorld"));
    assert_eq!(execute(r#""Hello" + 'World'"#).unwrap(), v::s("HelloWorld"));
}

#[test]
fn escapes_string() {
    assert_eq!(execute(r#"'String\''"#).unwrap(), v::s("String'"));
    assert_eq!(execute(r#"'String\\'"#).unwrap(), v::s("String\\"));
}

#[test]
fn index_strings() {
    assert_eq!(execute(r#"'hello'[2]"#).unwrap(), v::s("l"));
    assert_eq!(execute(r#"'hello'[-2]"#).unwrap(), v::s("l"));

    assert_eq!(execute(r#"'hello'[2..5]"#).unwrap(), v::s("llo"));
    assert_eq!(execute(r#"'hello'[-4..-2]"#).unwrap(), v::s("el"));
}

#[test]
fn inbuilt_property_strings() {
    assert_eq!(execute(r#"'hello'.len"#).unwrap(), v::i(5));
}
