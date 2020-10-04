use crate::value;
use std::collections::HashMap;

pub struct Scope {
    vars: HashMap<String, value::Value>,
}

impl Scope {
    #[allow(dead_code)]
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
}
