mod bytecode;
mod diagnostics;
mod evaluator;
mod lexer;
mod lowerer;
mod parser;
mod scope;
mod serialize;
mod source_text;
mod syntax_node;
mod text_span;
mod tokens;
mod types;
mod value;

#[cfg(test)]
mod test_helpers;

pub use bytecode::*;
pub use diagnostics::Diagnostics;
pub use evaluator::Evaluator;
pub use lexer::Lexer;
pub use lowerer::Lowerer;
pub use parser::Parser;
pub use scope::Scope;
pub use serialize::{DeserializationContext, Deserialize, DeserializeCtx, Serialize};
pub use source_text::SourceText;
pub use tokens::TokenKind;
pub use types::Type;
pub use value::Value;

pub mod function {
    pub use super::value::{AnilangFn, Function, NativeFn};

    pub mod native {
        pub use crate::value::function::native_fn::*;
    }
}
