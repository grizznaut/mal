use crate::errors::MalErr;
use crate::types::MalType;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Env {
    data: RefCell<BTreeMap<String, MalType>>,
    outer: Option<Rc<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Env {
    pub fn new(outer: Option<Rc<Env>>) -> Self {
        Self {
            data: RefCell::new(BTreeMap::new()),
            outer: outer,
        }
    }

    /// takes a symbol key and a mal value and adds to the data structure
    pub fn set(&self, symbol: String, value: MalType) -> Option<MalType> {
        self.data.borrow_mut().insert(symbol, value)
    }

    /// takes a symbol key and if the current environment contains that key then return the environment.
    /// If no key is found and outer is not nil then call find (recurse) on the outer environment.
    pub fn find(&self, symbol: &str) -> Option<Self> {
        if self.data.borrow().contains_key(symbol) {
            return Some(self.clone());
        } else {
            match &self.outer {
                Some(env) => env.find(symbol),
                None => None,
            }
        }
    }

    /// takes a symbol key and uses the find method to locate the environment with the key, then returns the matching value.
    /// If no key is found up the outer chain, then throws/raises a "not found" error.
    pub fn get(&self, symbol: &str) -> Result<MalType, MalErr> {
        match self.find(symbol) {
            Some(env) => Ok(env.data.borrow().get(symbol).unwrap().clone()), // unwrap() is safe because find() checks for existence of key
            None => Err(MalErr::SymbolNotFound(symbol.to_string())),
        }
    }

    /// Bind (set) each element (symbol) of the binds list to the respective element of the exprs list.
    pub fn bind(&self, binds: MalType, exprs: Vec<MalType>) -> Result<Self, MalErr> {
        match binds {
            MalType::List(b, _) | MalType::Vector(b, _) => {
                for (i, bind) in b.iter().enumerate() {
                    self.set(bind.to_string(), exprs[i].clone());
                }
                Ok(self.clone())
            }
            _ => Err(MalErr::Generic("binds is not a list or vector".to_string())),
        }
    }
}
