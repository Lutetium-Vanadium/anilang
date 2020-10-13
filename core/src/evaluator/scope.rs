use crate::value;
use std::collections::HashMap;

/// A wrapper around `HashMap` to provide scoping functions.
///
/// It's is only needed to be manually created when some global variables are to be provided while
/// evaluating, otherwise, the evaluator automatically creates the required scopes for each block.
///
/// # Examples
/// ```
/// use anilang::{Scope, Value};
///
/// let mut scope = Scope::new();
///
/// assert_eq!(scope.try_get_value("variable"), None);
/// scope.insert("variable".to_owned(), Value::Int(123));
/// assert_eq!(scope.try_get_value("variable"), Some(&Value::Int(123)));
/// ```
#[derive(Clone)]
pub struct Scope {
    vars: HashMap<String, value::Value>,
}

impl Scope {
    /// Creates a new Scope
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Sets the value for a variable in the scope if it exists, otherwise creates it
    pub fn insert(&mut self, key: String, value: value::Value) -> Option<value::Value> {
        self.vars.insert(key, value)
    }

    /// Returns a reference to the value stored at key
    pub fn try_get_value(&self, key: &str) -> Option<&value::Value> {
        self.vars.get(key)
    }

    /// Replaces the variables in this scope, with that of another scope
    pub fn replace(&mut self, scope: Scope) {
        self.vars = scope.vars;
    }
}
