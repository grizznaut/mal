use std::ops::Deref;

use crate::types::MalType;
use itertools::Itertools;

impl MalType {
    pub fn pr_str(self: &Self) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(b) => b.to_string(),
            MalType::Int(i) => i.to_string(),
            MalType::Symbol(s) => s.to_string(),
            MalType::List(l, _) => pr_list(l.deref()),
        }
    }
}

fn pr_list(seq: &Vec<MalType>) -> String {
    let inner = seq.iter().map(|el| el.pr_str()).join(" ");
    format!("({})", inner)
}
