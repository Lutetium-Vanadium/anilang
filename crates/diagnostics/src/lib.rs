use crossterm::{queue, style};
use intermediaries::{Token, TokenKind};
use source::{SourceText, TextBase, TextSpan};
use std::cell::Cell;
use std::io::{self, prelude::*};
use vm::types::ToString;
use vm::value;

/// A general Error struct for printing errors raised during the
#[derive(Debug)]
struct Diagnostic {
    message: String,
    span: TextSpan,
    level: DiagnosticLevel,
}

#[derive(Debug)]
enum DiagnosticLevel {
    Error,
    Warning,
}

impl Diagnostic {
    fn warning(message: String, span: TextSpan) -> Self {
        Self {
            message,
            span,
            level: DiagnosticLevel::Warning,
        }
    }

    fn error(message: String, span: TextSpan) -> Self {
        Self {
            message,
            span,
            level: DiagnosticLevel::Error,
        }
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
    fn prt<T: TextBase>(&self, src: &SourceText<'_, T>) -> crossterm::Result<()> {
        let s = match src.lineno(self.span.start()) {
            Some(s) => s,
            None => {
                return Err(crossterm::ErrorKind::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid span",
                )))
            }
        };

        // End is non inclusive
        let e = match src.lineno(self.span.end() - 1) {
            Some(e) => e,
            None => {
                return Err(crossterm::ErrorKind::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid span",
                )))
            }
        };

        let mut stdout = std::io::stdout();
        let (color, msg) = match self.level {
            DiagnosticLevel::Error => (style::Color::DarkRed, "error: "),
            DiagnosticLevel::Warning => (style::Color::Yellow, "warning: "),
        };

        queue!(
            stdout,
            style::SetForegroundColor(color),
            style::SetAttribute(style::Attribute::Bold),
            style::Print(msg),
            style::SetForegroundColor(style::Color::White),
            style::Print(&self.message),
            style::SetAttribute(style::Attribute::NoBold),
            style::Print('\n'),
        )?;

        if !src.has_text() {
            queue!(
                stdout,
                style::SetForegroundColor(color),
                style::Print("The error occurred in "),
                style::Print(if s == e {
                    format!(
                        "line: {}  character: {}",
                        s,
                        self.span.start() - src.line(s).0
                    )
                } else {
                    format!("lines: {} - {}", s, e)
                }),
                style::Print('\n'),
                style::ResetColor,
            )?;
        } else {
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

            queue!(
                stdout,
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
                    style::Print(&src[src.line(s).0..self.span.start()]),
                    style::SetForegroundColor(color),
                    style::SetAttribute(style::Attribute::Underlined),
                    style::Print(&src[&self.span]),
                    style::ResetColor,
                    style::SetAttribute(style::Attribute::NoUnderline),
                    style::Print(&src[self.span.end()..src.line(s).1]),
                    style::Print('\n'),
                )?;
            } else {
                queue!(
                    stdout,
                    style::SetForegroundColor(style::Color::DarkBlue),
                    style::Print(format!("{: >w$} | ", s, w = w)),
                    style::ResetColor,
                    style::Print(&src[src.line(s).0..self.span.start()]),
                    style::SetForegroundColor(color),
                    style::SetAttribute(style::Attribute::Underlined),
                    style::Print(&src[self.span.start()..src.line(s).1]),
                    style::Print('\n'),
                )?;

                for i in (s + 1)..e {
                    queue!(
                        stdout,
                        style::SetForegroundColor(style::Color::DarkBlue),
                        style::SetAttribute(style::Attribute::NoUnderline),
                        style::Print(format!("{: >w$} | ", i, w = w)),
                        style::SetForegroundColor(color),
                        style::SetAttribute(style::Attribute::Underlined),
                        style::Print(&src[src.line(i).0..src.line(i).1]),
                        style::Print('\n'),
                    )?;
                }

                queue!(
                    stdout,
                    style::SetForegroundColor(style::Color::DarkBlue),
                    style::SetAttribute(style::Attribute::NoUnderline),
                    style::Print(e),
                    style::Print(" | "),
                    style::SetForegroundColor(color),
                    style::SetAttribute(style::Attribute::Underlined),
                    style::Print(&src[src.line(e).0..self.span.end()]),
                    style::ResetColor,
                    style::SetAttribute(style::Attribute::NoUnderline),
                    style::Print(&src[self.span.end()..src.line(e).1]),
                    style::Print('\n'),
                )?;
            }

            queue!(
                stdout,
                style::SetForegroundColor(style::Color::DarkBlue),
                style::Print(" ".repeat(w)),
                style::Print(" |\n"),
                style::ResetColor,
            )?;
        }

        stdout.flush().map_err(crossterm::ErrorKind::IoError)
    }
}

/// An error reporter which pretty prints errors for a given `SourceText`, while keeping track of
/// number of errors are reported
///
/// # Examples
///
/// Regular diagnostics which will count and print
/// ```
/// # use source::SourceText;
/// use diagnostics::Diagnostics;
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src);
/// ```
///
/// If you need to mock diagnostics, without printing the errors
/// ```
/// # use source::SourceText;
/// use diagnostics::Diagnostics;
///
/// let src = SourceText::new("1 + 2 + 3");
/// let diagnostics = Diagnostics::new(&src).no_print();
/// ```
pub struct Diagnostics<'a, T: TextBase = &'a str> {
    /// The source from which text spans are taken
    src: &'a SourceText<'a, T>,
    /// To keep track of the number of errors generated
    /// A Cell is used so that `diagnostics` can be passed immutably, which reduces the number of
    /// clones required in the parser, since if a mutable reference to the `diagnostics` is needed,
    /// the an immutable reference to a token cannot simultaneously be held
    num_errors: Cell<usize>,
    /// To keep track of the number of warnings generated
    /// A Cell is used so that `diagnostics` can be passed immutably, which reduces the number of
    /// clones required in the parser, since if a mutable reference to the `diagnostics` is needed,
    /// the an immutable reference to a token cannot simultaneously be held
    num_warnings: Cell<usize>,
    /// If enabled, errors are just counted and not printed, this can be used for dummy Diagnostics,
    /// like if you need to look at the tokens generated from some code, but are not actually going
    /// to run it, and so don't need to print errors
    no_print: bool,
}

impl<'a, T: TextBase> Diagnostics<'a, T> {
    pub fn new(src: &'a SourceText<'a, T>) -> Self {
        Diagnostics {
            src,
            num_errors: Cell::new(0),
            num_warnings: Cell::new(0),
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

    pub fn num_warnings(&self) -> usize {
        self.num_warnings.get()
    }

    /// base function which constructs errors and prints them as long as `no_print` is disabled
    fn report_err(&self, message: String, span: TextSpan) {
        self.num_errors.set(self.num_errors() + 1);

        if !self.no_print {
            let _ = Diagnostic::error(message, span).prt(self.src);
        }
    }

    /// base function which constructs warnings and prints them as long as `no_print` is disabled
    fn report_warning(&self, message: String, span: TextSpan) {
        self.num_warnings.set(self.num_warnings() + 1);

        if !self.no_print {
            let _ = Diagnostic::warning(message, span).prt(self.src);
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
    pub fn bad_char(&self, span: TextSpan) {
        self.report_err(
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
    pub fn unexpected_eof(&self, span: TextSpan) {
        self.report_err("UnexpectedEOF".to_owned(), span);
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
            self.report_err(
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
        if unexpected.kind == TokenKind::EOF {
            return self.unexpected_eof(TextSpan::new(unexpected.text_span.start() - 2, 1));
        }

        if unexpected.kind != TokenKind::Bad {
            if let Some(correct) = expected {
                self.report_err(
                    format!(
                        "IncorrectToken: {:?}, expected {:?}",
                        unexpected.kind, correct,
                    ),
                    unexpected.text_span.clone(),
                );
            } else {
                self.report_err(
                    format!("UnexpectedToken: {:?}", unexpected.kind),
                    unexpected.text_span.clone(),
                );
            }
        }
    }

    /// Generated in the lowerer
    ///
    /// Is reported when there is a break statement outside a loop.
    /// see `core/src/lowerer/mod.rs`
    pub fn break_outside_loop(&self, span: TextSpan) {
        self.report_err(
            "BreakOutsideLoop: breaks can only be used in for loops, while loops and regular loops"
                .to_owned(),
            span,
        )
    }

    /// Generated in the lowerer
    ///
    /// Is reported when there is a break statement outside a loop.
    /// see `core/src/lowerer/mod.rs`
    pub fn return_outside_fn(&self, span: TextSpan) {
        self.report_err(
            "ReturnOutsideFn: return can only be used in function declarations".to_owned(),
            span,
        )
    }

    /// Generated in the lowerer
    ///
    /// Is reported when a statement is const evaluable, but does not occur at the end of the block
    /// see `core/src/lowerer/mod.rs`
    /// Examples:
    /// {
    ///     let a = 1231
    ///     1231 + 12313
    ///     ^^^^^^^^^^^^
    ///     a + 2131
    /// }
    /// Computing 1231 + 12313 is futile since it doesn't change a variable or get used anywhere.
    /// Note this is only checked for when the should_optimize flag is enabled in the lowerer.
    pub fn unused_statement(&self, span: TextSpan) {
        self.report_warning(
            "UnusedStatement: this statement has no side effects and the value produced by it is not used"
                .to_owned(),
            span
        )
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
        self.report_err(
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
        self.report_err(
            format!("SyntaxError: Variable `{}` was already declared", ident),
            span,
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
            value::ErrorKind::IncorrectArgCount { got, expected } => {
                format!("TypeError: expected {} args, got {} args", expected, got)
            }
            value::ErrorKind::InvalidProperty { val, property } => {
                if let value::Value::Object(_) = val {
                    format!(
                        "InvalidProperty: property '{}' does not exist on object {}",
                        property.borrow().as_str(),
                        val,
                    )
                } else {
                    format!(
                        "InvalidProperty: property '{}' does not exist on type <{}>",
                        property.borrow().as_str(),
                        val.type_()
                    )
                }
            }
            value::ErrorKind::ReadonlyProperty { val, property } => {
                format!(
                    "ReadonlyProperty: property '{}' is immutable for type <{}>",
                    property.borrow().as_str(),
                    val.type_()
                )
            }
            value::ErrorKind::CannotCompare { left, right } => {
                format!("Cannot compare values of type <{}> and <{}>", left, right)
            }
            value::ErrorKind::Other { message } => message,
        };

        self.report_err(msg, span)
    }
}
