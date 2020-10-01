use crate::value;
use std::collections::HashMap;

pub struct Scope {
    vars: HashMap<String, value::Value>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new(parent: Box<Scope>) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn root() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn insert(&mut self, key: String, value: value::Value) -> Option<value::Value> {
        self.vars.insert(key, value)
    }

    pub fn try_get_value(&self, key: &str) -> Option<&value::Value> {
        // While cleaner, the below code has some issues with moving out values under a shared
        // reference
        // self.vars
        //     .get(key)
        //     .or_else(|| (self.parent).and_then(|p| p.try_get_value(key)))

        match self.vars.get(key) {
            Some(v) => Some(v),
            None => match self.parent {
                Some(ref p) => p.try_get_value(key),
                None => None,
            },
        }
    }
}
