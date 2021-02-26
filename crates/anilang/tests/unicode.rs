mod common;
use common::*;

// Places where nothing is asserted, are tests to make sure it can process them without panicking
#[test]
fn has_unicode() {
    let _ = execute("Some └ Unicode symbols");
}

#[test]
fn unicode_in_string() {
    assert_eq!(execute("'Hello └ World'").unwrap(), v::s("Hello └ World"));
}

#[test]
fn unterminated_string_unicode() {
    let _ = execute("let a = 'Hello └");
}

#[test]
fn indexing_unicode_string() {
    assert_eq!(execute("'Hello └ World'[3]").unwrap(), v::s("l"));
    assert_eq!(execute("'Hello └ World'[6]").unwrap(), v::s("└"));
    assert_eq!(execute("'Hello └ World'[9]").unwrap(), v::s("o"));
}

#[test]
fn index_assign_unicode_string() {
    assert_eq!(
        execute("let s = 'Hello └ World' s[3] = 'r'").unwrap(),
        v::s("Helro └ World")
    );
    assert_eq!(
        execute("let s = 'Hello └ World' s[6] = '»'").unwrap(),
        v::s("Hello » World")
    );
    assert_eq!(
        execute("let s = 'Hello └ World' s[9] = 'a'").unwrap(),
        v::s("Hello └ Warld")
    );
    assert_eq!(
        execute("let s = 'Hello └ World' s[3] = '»'").unwrap(),
        v::s("Hel»o └ World")
    );
}
