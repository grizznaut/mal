use crate::errors::MalErr;
use crate::types::MalType;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct Env {
    data: HashMap<String, MalType>,
    outer: Rc<Option<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        Self {
            data: HashMap::new(),
            outer: Rc::new(outer),
        }
    }

    /// takes a symbol key and a mal value and adds to the data structure
    pub fn set(&mut self, symbol: String, value: MalType) -> Option<MalType> {
        self.data.insert(symbol, value)
    }

    /// takes a symbol key and if the current environment contains that key then return the environment.
    /// If no key is found and outer is not nil then call find (recurse) on the outer environment.
    pub fn find(&self, symbol: &str) -> Option<Env> {
        if self.data.contains_key(symbol) {
            return Some(self.clone());
        } else {
            match self.outer.deref() {
                Some(env) => env.find(symbol),
                None => None,
            }
        }
    }

    /// takes a symbol key and uses the find method to locate the environment with the key, then returns the matching value.
    /// If no key is found up the outer chain, then throws/raises a "not found" error.
    pub fn get(&self, symbol: &str) -> Result<MalType, MalErr> {
        match self.find(symbol) {
            Some(env) => Ok(env.data.get(symbol).unwrap().clone()), // unwrap() is safe because find() checks for existence of key
            None => Err(MalErr::SymbolNotFound(symbol.to_string())),
        }
    }
}
