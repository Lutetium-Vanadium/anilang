#[macro_use]
mod common;
use common::*;

#[test]
fn numeric_algebra() {
    assert_eq!(execute("1+1").unwrap(), v::i(2));
    assert_eq!(execute("387-23").unwrap(), v::i(364));
    assert_eq!(execute("23*3").unwrap(), v::i(69));
    assert_eq!(execute("23 / 6").unwrap(), v::i(3));
    assert_eq!(execute("3^4").unwrap(), v::i(81));
    assert_eq!(execute("22.0 / 4").unwrap(), v::f(5.5));

    assert_eq!(execute("12 + 23 - ((23 * 56 / 12)%7)^3").unwrap(), v::i(27));
    assert_eq!(
        execute("32 * 23 + 223 - (2 - 24 * 2)/27").unwrap(),
        v::i(960)
    );
    assert_eq!(
        execute("23^2 - 212*23 +(213 - 1 * 231)").unwrap(),
        v::i(-4365)
    );
    assert_eq!(execute("23.4 * (23 + 12)/2.1").unwrap(), v::f(390.0));
    assert_almost_eq!(
        f64::from(execute("23^2 - 212.4/23 +(213 - 1.7/23)").unwrap()),
        732.6913
    );
    assert_almost_eq!(
        f64::from(execute("32 * 23 + 223 - (2 - 24.0 * 2)/27").unwrap()),
        960.7037
    );
}

#[test]
fn boolean_algebra() {
    assert_eq!(execute("true  || false").unwrap(), v::b(true));
    assert_eq!(execute("false || true").unwrap(), v::b(true));
    assert_eq!(execute("true  || true").unwrap(), v::b(true));
    assert_eq!(execute("false || false").unwrap(), v::b(false));

    assert_eq!(execute("true  && false").unwrap(), v::b(false));
    assert_eq!(execute("false && true").unwrap(), v::b(false));
    assert_eq!(execute("false && false").unwrap(), v::b(false));
    assert_eq!(execute("true  && true").unwrap(), v::b(true));

    assert_eq!(execute("!false").unwrap(), v::b(true));
    assert_eq!(execute("!true").unwrap(), v::b(false));

    assert_eq!(execute("true && !false").unwrap(), v::b(true));
    assert_eq!(execute("!true || !false").unwrap(), v::b(true));

    assert_eq!(execute("true && (true || false)").unwrap(), v::b(true));
    assert_eq!(execute("false || true && (!true)").unwrap(), v::b(false));
    assert_eq!(execute("true && (!true || false)").unwrap(), v::b(false));
    assert_eq!(
        execute("(false && true) || (!false || !true)").unwrap(),
        v::b(true)
    );

    assert_eq!(execute("23 > 1").unwrap(), v::b(true));
    assert_eq!(execute("2 > 12").unwrap(), v::b(false));

    assert_eq!(execute("23 < 1").unwrap(), v::b(false));
    assert_eq!(execute("2 < 12").unwrap(), v::b(true));
    assert_eq!(execute("23 >= 1").unwrap(), v::b(true));
    assert_eq!(execute("2 >= 12").unwrap(), v::b(false));
    assert_eq!(execute("12 >= 12").unwrap(), v::b(true));

    assert_eq!(execute("23 <= 1").unwrap(), v::b(false));
    assert_eq!(execute("2 <= 12").unwrap(), v::b(true));
    assert_eq!(execute("12 <= 12").unwrap(), v::b(true));

    assert_eq!(execute("12 != 12").unwrap(), v::b(false));
    assert_eq!(execute("32 != 12").unwrap(), v::b(true));

    assert_eq!(execute("12 == 12").unwrap(), v::b(true));
    assert_eq!(execute("12 == 42").unwrap(), v::b(false));
}
