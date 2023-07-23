use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum MalType {
    Nil,
    Bool(bool),
    Int(i64),
    Symbol(String),
    List(Rc<Vec<MalType>>, Rc<MalType>),
}

impl fmt::Display for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pr_str())
    }
}

#[macro_export]
macro_rules! list {
    ( $l:expr ) => {{
        MalType::List(Rc::new($l), Rc::new(MalType::Nil))
    }};
    [ $($args:expr),* ] => {{
        let v: Vec<MalType> = vec![$($args),*];
        MalType::List(Rc::new(v),Rc::new(MalType::Nil))
    }};
}
