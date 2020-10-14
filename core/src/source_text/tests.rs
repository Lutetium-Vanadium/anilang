use super::*;

const PROG: &'static str = "let a = 1231\nlet b = 123123\nlet c = a + b";
const PROG_CR: &'static str = "let a = 1231\n\rlet b = 123123\n\rlet c = a + b";

#[test]
fn detect_correct_lines() {
    let src = SourceText::new(PROG);
    assert_eq!(src.lines, vec![(0, 12), (13, 27), (28, 41)]);
}

#[test]
fn detect_correct_lines_with_cr() {
    let src = SourceText::new(PROG_CR);
    assert_eq!(src.lines, vec![(0, 12), (14, 28), (30, 43)]);
}

#[test]
fn get_correct_lineno() {
    let src = SourceText::new(PROG);

    assert_eq!(src.lineno(1).unwrap(), 0);
    assert_eq!(src.lineno(11).unwrap(), 0);
    assert_eq!(src.lineno(15).unwrap(), 1);
    assert_eq!(src.lineno(24).unwrap(), 1);
    assert_eq!(src.lineno(34).unwrap(), 2);
    assert_eq!(src.lineno(40).unwrap(), 2);
}

#[test]
fn get_correct_lineno_with_offset() {
    let src = SourceText::with_offset(PROG, 22);

    assert_eq!(src.lineno(1).unwrap(), 22);
    assert_eq!(src.lineno(11).unwrap(), 22);
    assert_eq!(src.lineno(15).unwrap(), 23);
    assert_eq!(src.lineno(24).unwrap(), 23);
    assert_eq!(src.lineno(34).unwrap(), 24);
    assert_eq!(src.lineno(40).unwrap(), 24);
}

#[test]
fn none_lineno() {
    let src = SourceText::new(PROG);

    assert_eq!(src.lineno(12), None);
    assert_eq!(src.lineno(27), None);
    assert_eq!(src.lineno(41), None);
    assert_eq!(src.lineno(240), None);
}

#[test]
fn correct_str_for_span() {
    let src = SourceText::new(PROG);

    assert_eq!(&src[&TextSpan::new(0, 4)], "let ");
    assert_eq!(&src[&TextSpan::new(21, 6)], "123123");
    assert_eq!(&src[&TextSpan::new(38, 3)], "+ b");
}