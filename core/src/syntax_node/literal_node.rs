use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::value::Value;

#[repr(u8)]
pub enum ErrorKind {
    FailedParse,
    NoDelim,
}

type Result<T> = std::result::Result<T, ErrorKind>;

pub trait Parse {
    fn parse(src: &str) -> Result<Value>;
}

impl Parse for i64 {
    fn parse(src: &str) -> Result<Value> {
        match src.parse() {
            Ok(v) => Ok(Value::Int(v)),
            Err(_) => Err(ErrorKind::FailedParse),
        }
    }
}

impl Parse for f64 {
    fn parse(src: &str) -> Result<Value> {
        match src.parse() {
            Ok(v) => Ok(Value::Float(v)),
            Err(_) => Err(ErrorKind::FailedParse),
        }
    }
}

impl Parse for String {
    fn parse(src: &str) -> Result<Value> {
        let mut string = String::new();
        let mut is_escaped = false;
        let mut chars = src.chars();
        let start_delim = chars.next().ok_or(ErrorKind::FailedParse)?;
        let end_delim = chars.next_back().ok_or(ErrorKind::FailedParse)?;

        if start_delim != end_delim {
            return Err(ErrorKind::NoDelim);
        }

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

        Ok(Value::String(string))
    }
}

impl Parse for bool {
    fn parse(src: &str) -> Result<Value> {
        match src {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(ErrorKind::FailedParse),
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

    pub(super) fn prt(&self, indent: String, is_last: bool) {
        let marker = if is_last { "└──" } else { "├──" };

        println!(
            "{}{}{} {}{}{}",
            crate::colour::LIGHT_GRAY,
            indent,
            marker,
            crate::colour::BRIGHT_GREEN,
            self,
            crate::colour::RESET,
        );
    }
}

use std::fmt;
impl fmt::Display for LiteralNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.value, self.value.type_())
    }
}
