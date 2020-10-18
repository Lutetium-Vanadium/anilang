use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::value::Value;
use crossterm::{queue, style};
use std::cell::RefCell;
use std::io::prelude::*;
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

        // Ignore the delimiter
        for chr in src[1..(src.len() - 1)].chars() {
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
        let marker = if is_last { "└── " } else { "├── " };

        let _ = queue!(
            stdout,
            style::SetForegroundColor(style::Color::Grey),
            style::Print(&indent),
            style::Print(marker),
            style::SetForegroundColor(style::Color::Green),
            style::Print(format!("{}\n", self)),
            style::ResetColor,
        );
    }
}

use std::fmt;
impl fmt::Display for LiteralNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.value, self.value.type_())
    }
}
