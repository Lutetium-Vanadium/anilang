type Result = std::result::Result<anilang::Value, ()>;

#[allow(dead_code)]
fn _execute(code: &str, scope: &mut anilang::Scope) -> Result {
    let src = anilang::SourceText::new(code);
    let diagnostics = anilang::Diagnostics::new(&src);

    let tokens = anilang::Lexer::lex(&src, &diagnostics);
    let root = anilang::Parser::parse(tokens, &src, &diagnostics);
    let value = anilang::Evaluator::evaluate_with_global(root, &diagnostics, scope);

    if diagnostics.any() {
        Err(())
    } else {
        Ok(value)
    }
}
#[allow(dead_code)]
/// Executes one statement
pub fn execute(code: &str) -> Result {
    _execute(code, &mut anilang::Scope::new())
}

#[allow(dead_code)]
/// Executes many statements, with the same global scope
pub fn execute_many(code: Vec<&str>) -> Vec<Result> {
    let mut scope = anilang::Scope::new();
    code.iter().map(|code| _execute(code, &mut scope)).collect()
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
    use anilang::Value;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Creates an `anilang::Value::Int()`
    #[allow(dead_code)]
    pub fn i(i: i64) -> Value {
        Value::Int(i)
    }

    /// Creates an `anilang::Value::Float()`
    #[allow(dead_code)]
    pub fn f(f: f64) -> Value {
        Value::Float(f)
    }

    /// Creates an `anilang::Value::Bool()`
    #[allow(dead_code)]
    pub fn b(b: bool) -> Value {
        Value::Bool(b)
    }

    /// Creates an `anilang::Value::String()`
    #[allow(dead_code)]
    pub fn s(s: &str) -> Value {
        Value::String(Rc::new(RefCell::new(s.to_owned())))
    }

    /// Creates an `anilang::Value::Null`
    #[allow(dead_code)]
    pub fn n() -> Value {
        Value::Null
    }
}

/// null != null, therefore checking if a value is null be using `assert_eq` or `==`, gives false
/// negative, this should be used instead
pub trait IsNull {
    fn is_null(&self) -> bool;
}
impl IsNull for anilang::Value {
    #[allow(dead_code)]
    fn is_null(&self) -> bool {
        match self {
            anilang::Value::Null => true,
            _ => false,
        }
    }
}
