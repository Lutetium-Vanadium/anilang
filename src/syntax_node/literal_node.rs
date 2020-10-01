use super::Node;
use crate::source_text::SourceText;
use crate::text_span::TextSpan;
use crate::tokens::{Token, TokenKind};

type Result<T> = std::result::Result<T, ()>;

#[derive(Debug, Default)]
pub struct Variable {
    ident: String,
}

pub trait Parse<T> {
    fn parse(src: &str) -> Result<T>;
}

impl Parse<i64> for i64 {
    fn parse(src: &str) -> Result<i64> {
        match src.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        }
    }
}

impl Parse<String> for String {
    fn parse(src: &str) -> Result<String> {
        Ok(src.to_owned())
    }
}

impl Parse<bool> for bool {
    fn parse(src: &str) -> Result<bool> {
        match src {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(()),
        }
    }
}

impl Parse<Variable> for Variable {
    fn parse(src: &str) -> Result<Variable> {
        Ok(Variable {
            ident: src.to_owned(),
        })
    }
}

pub struct LiteralNode<T: Parse<T>> {
    token_kind: TokenKind,
    value: T,
    span: TextSpan,
}

impl<T: Parse<T>> LiteralNode<T> {
    pub fn new(token: &Token, src: &SourceText) -> Result<Self> {
        Ok(Self {
            value: T::parse(&src[&token.text_span])?,
            token_kind: token.kind.clone(),
            span: token.text_span.clone(),
        })
    }
}

impl<T> LiteralNode<T>
where
    T: Parse<T> + Default,
{
    pub fn bad() -> LiteralNode<T> {
        LiteralNode {
            token_kind: TokenKind::Bad,
            value: Default::default(),
            span: Default::default(),
        }
    }
}

use std::fmt;
impl<T> fmt::Display for LiteralNode<T>
where
    T: Parse<T> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} <{:?}>", self.value, self.token_kind)
    }
}

impl<T> Node for LiteralNode<T>
where
    T: Parse<T> + fmt::Debug,
{
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
            crate::colour::BRIGHT_YELLOW,
            self,
            crate::colour::RESET,
        );
    }
}
