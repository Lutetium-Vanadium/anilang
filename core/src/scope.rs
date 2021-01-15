use crate::value;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A wrapper around `HashMap` to provide scoping functions.
///
/// It is only needed to be manually created when some global variables are to be provided while
/// evaluating, otherwise, the lowerer automatically creates the required scopes for each block.
///
/// # Examples
/// ```
/// use anilang::{Scope, Value};
///
/// let scope = Scope::new(0, None);
///
/// assert_eq!(scope.try_get_value("variable"), None);
/// scope.declare("variable".to_owned(), Value::Int(123));
/// assert_eq!(scope.try_get_value("variable"), Some(Value::Int(123)));
/// ```
#[derive(Default, Debug)]
pub struct Scope {
    pub id: usize,
    vars: UnsafeCell<HashMap<String, value::Value>>,
    parent: Option<Rc<Scope>>,
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        // Scopes should have a unique identifier
        self.id == other.id
    }
}

impl Eq for Scope {}

impl Scope {
    /// Creates a new Scope
    pub fn new(id: usize, parent: Option<Rc<Scope>>) -> Self {
        Self {
            id,
            vars: Default::default(),
            parent,
        }
    }

    pub fn duplicate(&self) -> Self {
        Self {
            id: self.id,
            vars: Default::default(),
            parent: self.parent.clone(),
        }
    }

    /////////////////////       UNSAFETY        /////////////////////

    // SAFETY: The safety is very similar to the safety of `std::cell::Cell`. Since no references
    // are given to the underlying data inside the HashMap, it can be mutated safely through a
    // shared reference. Also since this is !Sync, data races are not an issue.
    //
    // Comparison to Cell<T>: The principle which a cell works on is that no references are possible
    // since the only way value is accessible is to copy the whole value. Similarly, here no
    // references are possible since the value for the key is copied.

    fn vars(&self) -> &HashMap<String, value::Value> {
        unsafe { &*self.vars.get() }
    }

    fn insert(&self, key: String, value: value::Value) {
        let vars = unsafe { &mut *self.vars.get() };
        vars.insert(key, value);
    }

    /////////////////////////////////////////////////////////////////

    /// Creates the variable `key` with value `value` in this scope.
    ///
    /// If the variable could be declared, it returns Ok(()), otherwise it errors with the key
    pub fn declare(&self, key: String, value: value::Value) -> Result<(), String> {
        if !self.vars().contains_key(&key) {
            self.insert(key, value);
            Ok(())
        } else {
            Err(key)
        }
    }

    /// Sets the value for the variable. If the variable is not found, it recurses to its parent.
    ///
    /// If the variable could be set, it returns Ok(()), otherwise it errors with the key
    pub fn set(&self, key: String, value: value::Value) -> Result<(), String> {
        if self.vars().contains_key(&key) {
            self.insert(key, value);
            Ok(())
        } else if let Some(ref parent) = self.parent {
            parent.set(key, value)
        } else {
            Err(key)
        }
    }

    /// Returns a copy of the value stored at key.
    pub fn try_get_value(&self, key: &str) -> Option<value::Value> {
        if let Some(value) = self.vars().get(key) {
            // The value must be cloned so that the safety argument holds
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.try_get_value(key)
        } else {
            None
        }
    }

    pub fn parent_id(&self) -> Option<usize> {
        self.parent.as_ref().map(|p| p.id)
    }
}
