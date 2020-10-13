use crate::value;
use std::collections::HashMap;

/// A wrapper around HashMap to store variables
#[derive(Clone)]
pub struct Scope {
    vars: HashMap<String, value::Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: value::Value) -> Option<value::Value> {
        self.vars.insert(key, value)
    }

    pub fn try_get_value(&self, key: &str) -> Option<&value::Value> {
        self.vars.get(key)
    }

    /// Replaces the variables in this scope, with that of another scope
    pub fn replace(&mut self, scope: Scope) {
        self.vars = scope.vars;
    }
}
