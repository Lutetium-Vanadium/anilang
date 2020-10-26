use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crate::types::ToString;
use crate::value;
use crossterm::{queue, style};
use node::SyntaxNode;
use std::cell::Cell;
use std::io::prelude::*;

/// A general Error struct for printing errors raised during the
#[derive(Debug)]
struct Error {
    message: String,
    span: TextSpan,
}

impl Error {
    fn new(message: String, span: TextSpan) -> Error {
        Error { message, span }
    }

    /// For the following code sample;
    /// 0 | let a = 234 + "sada"
    ///
    /// The following error is expected to be generated:
    /// Error {
    ///     message: "IncorrectRightType: Expected <int>, got <string>",
    ///     span: TextSpan { start: 12, len: 1 }
    /// }
    ///
    /// Which prints the following:
    /// IncorrectRightType: Expected <int>, got <string>
    ///   |
    /// 0 | let a = 234 + "sada"
    ///   |
    /// note the + is in red and underlined
    fn prt(&self, src: &SourceText) -> crossterm::Result<()> {
        let s = match src.lineno(self.span.start()) {
            Some(s) => s,
            None => return Err(crossterm::ErrorKind::__Nonexhaustive),
        };

        // End is non inclusive
        let e = match src.lineno(self.span.end() - 1) {
            Some(e) => e,
            None => return Err(crossterm::ErrorKind::__Nonexhaustive),
        };

        // get the width of the largest line number so all the '|' line up
        let w = if e > 0 {
            let mut e2 = e;
            let mut w = 0;
            while e2 > 0 {
                w += 1;
                e2 /= 10;
            }
            w
        } else {
            1
        };

        let mut stdout = std::io::stdout();

        queue!(
            stdout,
            style::SetForegroundColor(style::Color::White),
            style::Print(&self.message),
            style::Print("\n"),
            style::SetForegroundColor(style::Color::DarkBlue),
            style::Print(" ".repeat(w)),
            style::Print(" |\n"),
        )?;

        if s == e {
            queue!(
                stdout,
                style::Print(s),
                style::Print(" | "),
                style::ResetColor,
                style::Print(&src.text[src.line(s).0..self.span.start()]),
                style::SetForegroundColor(style::Color::DarkRed),
                style::SetAttribute(style::Attribute::Underlined),
                style::Print(&src[&self.span]),
                style::ResetColor,
                style::SetAttribute(style::Attribute::NoUnderline),
                style::Print(&src.text[self.span.end()..src.line(s).1]),
                style::Print("\n"),
            )?;
        } else {
            queue!(
                stdout,
                style::SetForegroundColor(style::Color::DarkBlue),
                style::Print(format!("{: >w$} | ", s, w = w)),
                style::ResetColor,
                style::Print(&src.text[src.line(s).0..self.span.start()]),
                style::SetForegroundColor(style::Color::DarkRed),
                style::SetAttribute(style::Attribute::Underlined),
                style::Print(&src.text[self.span.start()..src.line(s).1]),
                style::Print("\n"),
            )?;

            for i in (s + 1)..e {
                queue!(
                    stdout,
                    style::SetForegroundColor(style::Color::DarkBlue),
                    style::SetAttribute(style::Attribute::NoUnderline),
                    style::Print(format!("{: >w$} | ", i, w = w)),
                    style::SetForegroundColor(style::Color::DarkRed),
                    style::SetAttribute(style::Attribute::Underlined),
                    style::Print(&src.text[src.line(i).0..src.line(i).1]),
                    style::Print("\n"),
                )?;
            }

            queue!(
                stdout,
                style::SetForegroundColor(style::Color::DarkBlue),
                style::SetAttribute(style::Attribute::NoUnderline),
                style::Print(e),
                style::Print(" | "),
                style::SetForegroundColor(style::Color::DarkRed),
                style::SetAttribute(style::Attribute::Underlined),
                style::Print(&src.text[src.line(e).0..self.span.end()]),
                style::ResetColor,
                style::SetAttribute(style::Attribute::NoUnderline),
                style::Print(&src.text[self.span.end()..src.line(e).1]),
                style::Print("\n"),
            )?;
        }

        queue!(
            stdout,
            style::SetForegroundColor(style::Color::DarkBlue),
            style::Print(" ".repeat(w)),
            style::Print(" |\n"),
            style::ResetColor,
        )?;

        stdout.flush().map_err(|e| crossterm::ErrorKind::IoError(e))
    }
}

/// An error reporter which pretty prints errors for a given `SourceText`, while keeping track of
/// number of errors are reported
///
/// # Examples
///
/// Regular diagnostics which will count and print
/// ```
/// use anilang::{SourceText, Diagnostics};
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
/// ```
///
/// If you need to mock diagnostics, without printing the errors
/// ```
/// use anilang::{SourceText, Diagnostics};
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src).no_print();
/// ```
pub struct Diagnostics<'a> {
    /// The source from which text spans are taken
    src: &'a SourceText<'a>,
    /// To keep track of the number of errors generated
    /// A Cell is used so that `diagnostics` can be passed immutably, which reduces the number of
    /// clones required in the parser, since if a mutable reference to the `diagnostics` is needed,
    /// the an immutable reference to a token cannot simultaneously be held
    num_errors: Cell<usize>,
    /// If enabled, errors are just counted and not printed, this can be used for dummy Diagnostics,
    /// like if you need to look at the tokens generated from some code, but are not actually going
    /// to run it, and so don't need to print errors
    no_print: bool,
}

impl<'a> Diagnostics<'a> {
    pub fn new(src: &'a SourceText) -> Self {
        Diagnostics {
            src,
            num_errors: Cell::new(0),
            no_print: false,
        }
    }

    pub fn no_print(mut self) -> Self {
        self.no_print = true;
        self
    }

    pub fn any(&self) -> bool {
        self.num_errors() > 0
    }

    pub fn num_errors(&self) -> usize {
        self.num_errors.get()
    }

    /// base function which constructs errors and prints them as long as `no_print` is disabled
    fn report(&self, message: String, span: TextSpan) {
        self.num_errors.set(self.num_errors.get() + 1);

        if !self.no_print {
            let _ = Error::new(message, span).prt(self.src);
        }
    }

    /// Generated in the lexer
    ///
    /// Is reported when an unknown character is present in the source code, see `core/src/lexer.rs`
    /// Examples:
    /// let a = ~1213
    ///         ^
    /// This character is currently unsupported
    ///
    /// However, these are not raised during string parsing
    /// let a = "~1213"
    /// Completely legal code
    pub fn bad_char(&self, index: usize) {
        let span = TextSpan::new(index, 1);
        self.report(
            format!("BadChar: Unknown character '{}'", &self.src[&span]),
            span,
        );
    }

    /// Generated in the lexer and parser
    ///
    /// Is reported when a token is expected, but an EOF is found, see `core/src/parser.rs`
    /// Examples:
    /// if true {
    ///         ^
    /// A block of code, or the close brace is expected
    pub fn unexpected_eof(&self) {
        self.report(
            format!("UnexpectedEOF"),
            TextSpan::new(self.src.text.len() - 1, 1),
        );
    }

    /// Generated in the parser
    ///
    /// Is reported when a part of the source text being evaluated as a literal fails to parse into
    /// the rust format, see `core/src/syntax_node/literal_node.rs`
    /// Examples:
    /// let a = 16398612361278713193
    ///         ^^^^^^^^^^^^^^^^^^^^
    /// Currently an integer is represented as a i64, so a number passed these bounds cannot be
    /// parsed into rust representation, so this will error
    pub fn failed_parse(&self, token: &Token) {
        if token.kind != TokenKind::Bad {
            self.report(
                format!(
                    "FailedParse: Couldn't parse the value into a '{}'",
                    match token.kind {
                        TokenKind::String => "string",
                        TokenKind::Number => "number",
                        TokenKind::Boolean => "boolean",
                        _ => unreachable!(),
                    }
                ),
                token.text_span.clone(),
            );
        }
    }

    /// Generated in the parser
    ///
    /// Is reported when a token is expected, but a different one is found `core/src/parser.rs`
    /// Examples:
    /// if true 2131
    ///         ^^^^
    /// An OpenBrace is expected, but a number was found
    pub fn unexpected_token(&self, unexpected: &Token, expected: Option<&TokenKind>) {
        if unexpected.kind != TokenKind::Bad {
            if let Some(correct) = expected {
                self.report(
                    format!(
                        "IncorrectToken: {:?}, expected {:?}",
                        unexpected.kind, correct,
                    ),
                    unexpected.text_span.clone(),
                );
            } else {
                self.report(
                    format!("UnexpectedToken: {:?}", unexpected.kind),
                    unexpected.text_span.clone(),
                );
            }
        }
    }

    /// Generated in the evaluator
    ///
    /// Is reported when a variable is used without being previously declared,
    /// see `core/src/evaluator/mod.rs`
    /// Examples:
    /// let a = a + 123
    ///         ^
    /// `a` wasn't not declared before
    pub fn unknown_reference(&self, ident: &str, span: TextSpan) {
        self.report(
            format!("UnknownReference: Variable `{}` not found", ident),
            span,
        )
    }

    /// Generated in the evaluator
    ///
    /// Is reported when a variable is being redeclared while already being declared in the
    /// current, see `core/src/evaluator/mod.rs`
    /// Examples:
    /// let a = 123
    /// let a = a + 123
    /// ^^^^^^^^^^^^^^^
    /// `a` was already declared in the previous line
    pub fn already_declared(&self, ident: &str, span: TextSpan) {
        self.report(
            format!("SyntaxError: Variable `{}` was already declared", ident),
            span,
        )
    }

    /// Generated in the evaluator
    ///
    /// Is reported when a variable is expected, but a value is gotten, it is mainly raised with
    /// the `++` and `--` operators, see `core/src/evaluator/mod.rs`
    /// Examples:
    /// ++123
    /// ^^
    /// `++` stores the value back in the variable, so cannot be performed on a literal
    pub fn expected_variable(&self, got: &SyntaxNode) {
        self.report(
            format!(
                "ExpectedVariable: `++` and `--` can only be performed on variables, got `{}`",
                got
            ),
            got.span().clone(),
        )
    }

    /// Generated in the evaluator
    ///
    /// This is a general method to convert a `value::ErrorKind` to an printed error, these are
    /// errors from executing binary or unary operations, see `core/src/value/mod.rs`
    /// Examples:
    /// 123 / 0
    ///     ^
    /// 123 % 0
    ///     ^
    /// Numbers cannot be divided by zero
    pub fn from_value_error(&self, err: value::ErrorKind, span: TextSpan) {
        let msg = match err {
            value::ErrorKind::DivideByZero => String::from("DivideByZero: Cannot divide by zero"),
            value::ErrorKind::OutOfBounds { got, start, end } => format!(
                "OutOfBounds: Value {} is out of the bounds {} <= x < {}",
                got, start, end
            ),
            value::ErrorKind::IncorrectType { got, expected } => format!(
                "IncorrectType: Expected <{}>, got <{}>",
                expected.to_string(),
                got
            ),
            value::ErrorKind::IncorrectLeftType { got, expected } => format!(
                "IncorrectLeftType: Expected <{}>, got <{}>",
                expected.to_string(),
                got
            ),
            value::ErrorKind::IncorrectRightType { got, expected } => format!(
                "IncorrectRightType: Expected <{}>, got <{}>",
                expected.to_string(),
                got
            ),
            value::ErrorKind::Unindexable { val_t, index_t } => format!(
                "Unindexable: Value of type <{}> is not indexable by <{}>",
                val_t, index_t,
            ),
            value::ErrorKind::IndexOutOfRange { index, len } => format!(
                "IndexOutOfRange: index {} out of range, len: {}",
                index, len,
            ),
        };

        self.report(msg, span)
    }

    pub fn incorrect_arg_count(&self, expected: usize, got: usize, span: TextSpan) {
        self.report(
            format!("TypeError: expected {} args, got {} args", expected, got),
            span,
        )
    }
}
