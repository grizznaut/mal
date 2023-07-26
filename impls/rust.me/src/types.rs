use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum MalType {
    Nil,
    Bool(bool),
    Int(i64),
    Symbol(String),
    List(Rc<Vec<MalType>>, Rc<MalType>),
    Vector(Rc<Vec<MalType>>, Rc<MalType>),
    HashMap(Rc<BTreeMap<MalType, MalType>>, Rc<MalType>),
    Function(fn(Vec<MalType>) -> MalType),
}

impl fmt::Display for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pr_str())
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
    pub fn apply(&self, args: Vec<MalType>) -> Result<MalType, &'static str> {
        match self {
            MalType::Function(f) => Ok(f(args)),
            _ => Err("Cannot apply non-function"),
        }
    }
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
            return Err("Odd number of arguments");
        }
        let mut hm = std::collections::BTreeMap::new();
        for w in $l.windows(2) {
            hm.insert(w[0].clone(), w[1].clone());
        }
        Ok(MalType::HashMap(
            std::rc::Rc::new(hm),
            std::rc::Rc::new(MalType::Nil),
        ))
    }};
}
