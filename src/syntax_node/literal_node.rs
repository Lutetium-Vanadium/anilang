use super::Node;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::value::Value;

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
        Ok(Value::String(src.to_owned()))
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
    pub value: Value,
    span: TextSpan,
}

impl LiteralNode {
    pub fn new<T: Parse>(span: TextSpan, src: &SourceText) -> Result<Self> {
        Ok(Self {
            value: T::parse(&src[&span])?,
            span,
        })
    }
}

use std::fmt;
impl fmt::Display for LiteralNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.value, self.value.type_())
    }
}

impl Node for LiteralNode {
    fn span(&self) -> &TextSpan {
        &self.span
    }

    fn prt(&self, indent: String, is_last: bool) {
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
