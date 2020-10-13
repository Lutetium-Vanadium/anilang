use crate::colour;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crate::types::ToString;
use crate::value;
use node::SyntaxNode;
use std::cell::Cell;

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
    fn prt(&self, src: &SourceText) {
        let s = match src.lineno(self.span.start()) {
            Some(s) => s,
            None => return,
        };

        // End is non inclusive
        let e = match src.lineno(self.span.end() - 1) {
            Some(e) => e,
            None => return,
        };

        // get the width of the largest line number so all the '|' line up
        let w = if e > 0 {
            let mut e2 = std::cmp::min(e, 1);
            let mut w = 0;
            while e2 > 0 {
                w += 1;
                e2 /= 10;
            }
            w
        } else {
            1
        };

        println!("{}{}{}", colour::WHITE, self.message, colour::RESET);
        println!("{}{} |{}", colour::BLUE, " ".repeat(w), colour::RESET);

        if s == e {
            println!(
                "{}{} |{} {}{}{}{}{}{}",
                colour::BLUE,
                s,
                colour::RESET,
                &src.text[src.line(s).0..self.span.start()],
                colour::RED,
                colour::UNDERLINE,
                &src[&self.span],
                colour::RESET,
                &src.text[self.span.end()..src.line(s).1],
            );
        } else {
            println!(
                "{}{:0w$} |{} {}{}{}{}",
                colour::BLUE,
                s,
                colour::RESET,
                &src.text[src.line(s).0..self.span.start()],
                colour::RED,
                colour::UNDERLINE,
                &src.text[self.span.start()..src.line(s).1],
                w = w
            );

            for i in (s + 1)..e {
                println!(
                    "{}{} |{} {}",
                    colour::BLUE,
                    i,
                    colour::RESET,
                    &src.text[src.line(i).0..src.line(i).1],
                );
            }

            println!(
                "{}{} |{} {}{}{}",
                colour::BLUE,
                e,
                colour::RESET,
                &src.text[src.line(e).1..self.span.end()],
                colour::RESET,
                &src.text[self.span.end()..src.line(e).1]
            );
        }

        println!("{}{} |{}", colour::BLUE, " ".repeat(w), colour::RESET);
    }
}

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
            Error::new(message, span).prt(self.src);
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
                        TokenKind::String(_) => "string",
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

    /// Generated in the parser
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

    /// Generated in the evaluator
    ///
    /// Is reported when a variable is used without being previously declared,
    /// see `core/src/evaluator/mod.rs`
    /// Examples:
    /// let a = a + 123
    ///         ^
    /// `a` wasn't not declared before
    pub fn unknown_reference(&self, variable: &node::VariableNode) {
        self.report(
            format!("UnknownReference: Variable `{}` not found", variable.ident),
            variable.span.clone(),
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
    pub fn already_declared(&self, variable: &node::DeclarationNode) {
        self.report(
            format!(
                "SyntaxError: Variable `{}` was already declared",
                variable.ident
            ),
            variable.span.clone(),
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
        };

        self.report(msg, span)
    }
}
