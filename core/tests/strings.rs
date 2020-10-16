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