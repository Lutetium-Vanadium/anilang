use anilang::function::{native, Function};
use std::rc::Rc;

type Result = std::result::Result<anilang::Value, ()>;

fn base_scope() -> Rc<anilang::Scope> {
    let scope = Rc::new(anilang::Scope::new(0, None));
    scope
        .declare(
            "assert".to_owned(),
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

    /// Creates an `anilang::Value::List()`
    #[allow(dead_code)]
    pub fn l(l: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(l)))
    }

    /// Creates an `anilang::Value::Object()`
    #[allow(dead_code)]
    pub fn o(o: Vec<(&str, Value)>) -> Value {
        let mut obj = std::collections::HashMap::new();
        for (k, v) in o {
            obj.insert(k.to_owned(), v);
        }
        Value::Object(Rc::new(RefCell::new(obj)))
    }

    /// Creates an `anilang::Value::Null`
    #[allow(dead_code)]
    pub fn n() -> Value {
        Value::Null
    }
}

// FIXME: No longer true, replace all is_null() with == Value::Null
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
