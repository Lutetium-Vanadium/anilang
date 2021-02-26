use anilang::function::{native, Function};
use std::rc::Rc;

type Result = std::result::Result<anilang::Value, ()>;

fn base_scope() -> Rc<anilang::Scope> {
    let scope = Rc::new(anilang::Scope::new(0, None));
    scope
        .declare(
            "assert".into(),
            anilang::Value::Function(Rc::new(Function::native_fn(native::assert))),
        )
        .expect("Could not declare assert");
    scope
}

#[allow(dead_code)]
fn _execute(code: &str, scope: Rc<anilang::Scope>) -> Result {
    let src = anilang::SourceText::new(code);
    let diagnostics = anilang::Diagnostics::new(&src).no_print();

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);
    let bytecode = anilang::Lowerer::lower_with_global(root, &diagnostics, scope, false);
    let value = anilang::Evaluator::evaluate(&bytecode, &diagnostics);

    if diagnostics.any() {
        Err(())
    } else {
        Ok(value)
    }
}
#[allow(dead_code)]
/// Executes one statement
pub fn execute(code: &str) -> Result {
    _execute(code, base_scope())
}

#[allow(dead_code)]
/// Executes many statements, with the same global scope
pub fn execute_many(code: Vec<&str>) -> Vec<Result> {
    let scope = base_scope();
    code.iter()
        .map(|code| _execute(code, Rc::clone(&scope)))
        .collect()
}

#[macro_export]
macro_rules! assert_almost_eq {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !((*left_val * 1000f64).round() == (*right_val * 1000f64).round()) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(
                        r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#,
                        &*left_val, &*right_val
                    )
                }
            }
        }
    }};
}

pub mod v {
    pub use vm::test_helpers::*;
}
