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
            MalType::List(l, _) => pr_list(l.deref(), "(", ")"),
            MalType::Vector(l, _) => pr_list(l.deref(), "[", "]"),
            MalType::HashMap(hm, _) => pr_list(
                &hm.iter()
                    .flat_map(|(k, v)| vec![k.clone(), v.clone()])
                    .collect(),
                "{",
                "}",
            ),
            MalType::MalFunction { .. } => "#<function>".to_string(),
            // MalType::MalFunction { params, body, env } => {
            //     format!("params {:?}, body: {:?}, env: {:?}", params, body, env)
            // }
            _ => todo!(),
        }
    }
}

fn pr_list(seq: &Vec<MalType>, open: &str, close: &str) -> String {
    let inner = seq.iter().map(|el| el.pr_str()).join(" ");
    format!("{}{}{}", open, inner, close)
}
