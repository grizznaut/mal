use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use crate::env::Env;
use crate::errors::MalErr;

#[derive(Clone, Debug, Ord, PartialOrd)]
pub enum MalType {
    Nil,
    Bool(bool),
    Int(i64),
    Str(String),
    Symbol(String),
    List(Rc<Vec<MalType>>, Rc<MalType>),
    Vector(Rc<Vec<MalType>>, Rc<MalType>),
    HashMap(Rc<BTreeMap<MalType, MalType>>, Rc<MalType>),
    Function(fn(Vec<MalType>) -> Result<MalType, MalErr>),
    MalFunction {
        eval: fn(ast: MalType, env: Rc<Env>) -> Result<MalType, MalErr>,
        params: Rc<MalType>,
        ast: Rc<MalType>,
        env: Rc<Env>,
    },
    Atom(Rc<RefCell<MalType>>),
}

impl fmt::Display for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pr_str(true))
    }
}

// Implemented manually to handle list <> vector comparison
impl Eq for MalType {}
impl PartialEq for MalType {
    fn eq(&self, other: &MalType) -> bool {
        match (self, other) {
            (MalType::Nil, MalType::Nil) => true,
            (MalType::Bool(ref a), MalType::Bool(ref b)) => a == b,
            (MalType::Int(ref a), MalType::Int(ref b)) => a == b,
            (MalType::Str(ref a), MalType::Str(ref b)) => a == b,
            (MalType::Symbol(ref a), MalType::Symbol(ref b)) => a == b,
            (MalType::List(ref a, _), MalType::List(ref b, _))
            | (MalType::Vector(ref a, _), MalType::Vector(ref b, _))
            | (MalType::List(ref a, _), MalType::Vector(ref b, _))
            | (MalType::Vector(ref a, _), MalType::List(ref b, _)) => a == b,
            (MalType::HashMap(ref a, _), MalType::HashMap(ref b, _)) => a == b,
            (MalType::MalFunction { .. }, MalType::MalFunction { .. }) => false,
            _ => false,
        }
    }
}

impl Add for MalType {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (MalType::Int(lhs), MalType::Int(rhs)) => MalType::Int(lhs + rhs),
            _ => todo!(),
        }
    }
}

impl Sub for MalType {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (MalType::Int(lhs), MalType::Int(rhs)) => MalType::Int(lhs - rhs),
            _ => todo!(),
        }
    }
}

impl Mul for MalType {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (MalType::Int(lhs), MalType::Int(rhs)) => MalType::Int(lhs * rhs),
            _ => todo!(),
        }
    }
}

impl Div for MalType {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (MalType::Int(lhs), MalType::Int(rhs)) => MalType::Int(lhs / rhs),
            _ => todo!(),
        }
    }
}

impl MalType {
    pub fn apply(&self, args: Vec<MalType>) -> Result<MalType, MalErr> {
        match self {
            MalType::Function(f) => f(args),
            MalType::MalFunction {
                eval,
                params,
                ast,
                env,
            } => {
                let fn_env = Rc::new(Env::new(Some(Rc::clone(&env))));
                fn_env.bind((**params).clone(), args)?;
                eval((**ast).clone(), fn_env)
            }
            _ => Err(MalErr::Generic("Cannot apply non-function".to_string())),
        }
    }
}

pub fn atom(a: &MalType) -> MalType {
    MalType::Atom(Rc::new(RefCell::new(a.clone())))
}

#[macro_export]
macro_rules! list {
    ( $l:expr ) => {{
        MalType::List(std::rc::Rc::new($l), std::rc::Rc::new(MalType::Nil))
    }};
    [ $($args:expr),* ] => {{
        let v: Vec<MalType> = vec![$($args),*];
        MalType::List(std::rc::Rc::new(v), std::rc::Rc::new(MalType::Nil))
    }};
}

#[macro_export]
macro_rules! vector {
    ( $l:expr ) => {{
        MalType::Vector(std::rc::Rc::new($l), std::rc::Rc::new(MalType::Nil))
    }};
    [ $($args:expr),* ] => {{
        let v: Vec<MalType> = vec![$($args),*];
        MalType::Vector(std::rc::Rc::new(v), std::rc::Rc::new(MalType::Nil))
    }};
}

#[macro_export]
macro_rules! hashmap {
    ( $l:expr ) => {{
        if $l.len() % 2 != 0 {
            return Err(crate::errors::MalErr::Generic(
                "Odd number of arguments".to_string(),
            ));
        }
        let mut hm = std::collections::BTreeMap::new();
        for w in $l.chunks(2) {
            hm.insert(w[0].clone(), w[1].clone());
        }
        Ok(MalType::HashMap(
            std::rc::Rc::new(hm),
            std::rc::Rc::new(MalType::Nil),
        ))
    }};
}
