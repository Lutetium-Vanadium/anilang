use crate::colour;
use crate::source_text::SourceText;
use crate::syntax_node as node;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};
use crate::types::ToString;
use crate::value;
use node::{Node, SyntaxNode};
use std::cell::Cell;

#[derive(Debug)]
struct Error {
    message: String,
    span: TextSpan,
}

impl Error {
    pub fn new(message: String, span: TextSpan) -> Error {
        Error { message, span }
    }

    pub fn prt(&self, src: &SourceText) {
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
    src: &'a SourceText<'a>,
    num_errors: Cell<usize>,
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

    #[allow(dead_code)]
    pub fn num_errors(&self) -> usize {
        self.num_errors.get()
    }
}

impl<'a> Diagnostics<'a> {
    fn report(&self, message: String, span: TextSpan) {
        self.num_errors.set(self.num_errors.get() + 1);

        if !self.no_print {
            Error::new(message, span).prt(self.src);
        }
    }

    pub fn bad_char(&self, index: usize) {
        let span = TextSpan::new(index, 1);
        self.report(
            format!("BadCharError: Unknown character '{}'", &self.src[&span]),
            span,
        );
    }

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

    pub fn unexpected_eof(&self) {
        self.report(
            format!("UnexpectedEOF"),
            TextSpan::new(self.src.text.len() - 1, 1),
        );
    }

    pub fn unknown_reference(&self, variable: &node::VariableNode) {
        self.report(
            format!("UnknownReference: Variable `{}` not found", variable.ident),
            variable.span().clone(),
        )
    }

    pub fn already_declared(&self, variable: &node::DeclarationNode) {
        self.report(
            format!(
                "SyntaxError: Variable `{}` was already declared",
                variable.ident
            ),
            variable.span().clone(),
        )
    }

    pub fn expected_variable(&self, got: &SyntaxNode) {
        self.report(
            format!(
                "ExpectedVariable: `++` and `--` can only be performed on variables, got `{}`",
                got
            ),
            got.span().clone(),
        )
    }

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
