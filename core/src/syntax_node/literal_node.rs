use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::value::Value;
use crossterm::style;
use std::cell::RefCell;
use std::rc::Rc;

type Result<T> = std::result::Result<T, ()>;

pub trait Parse {
    fn parse(src: &str) -> Result<Value>;
}

impl Parse for i64 {
    fn parse(src: &str) -> Result<Value> {
        match src.parse() {
            Ok(v) => Ok(Value::Int(v)),
            Err(_) => Err(()),
        }
    }
}

impl Parse for f64 {
    fn parse(src: &str) -> Result<Value> {
        match src.parse() {
            Ok(v) => Ok(Value::Float(v)),
            Err(_) => Err(()),
        }
    }
}

impl Parse for String {
    fn parse(src: &str) -> Result<Value> {
        let mut string = String::new();
        let mut is_escaped = false;
        let mut chars = src.chars();

        // ignore the delimiters
        // note: although the lexer already operates on the assumption
        // that the delimiters are 1 byte long, there is no guarantee,
        // that the first and last characters are necessarily delimiters.
        // For example:
        // ```
        // "Pa└
        // ```
        // Parsing of the above code would panic.
        // The lexer would report an UnexpectedEOF, but since error count
        // is not checked before every step, the incomplete string will
        // still be parsed
        let _ = chars.next();
        let _ = chars.next_back();

        // Ignore the delimiter
        for chr in chars {
            if is_escaped {
                is_escaped = !is_escaped;
            } else if chr == '\\' {
                is_escaped = true;
                // The escaping `\` should no be added to the string
                continue;
            }

            string.push(chr);
        }

        Ok(Value::String(Rc::new(RefCell::new(string))))
    }
}

impl Parse for bool {
    fn parse(src: &str) -> Result<Value> {
        match src {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiteralNode {
    pub span: TextSpan,
    pub value: Value,
}

impl LiteralNode {
    pub fn new<T: Parse>(span: TextSpan, src: &SourceText) -> Result<Self> {
        Ok(Self {
            value: T::parse(&src[&span])?,
            span,
        })
    }

    pub(super) fn _prt(&self, indent: String, is_last: bool, stdout: &mut std::io::Stdout) {
        let _ = super::print_node(style::Color::Green, &indent, self, is_last, stdout);
    }
}

use std::fmt;
impl fmt::Display for LiteralNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.value, self.value.type_())
    }
}
