use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Deref;

use crate::{core::KEYWORD_PREFIX, types::MalType};
use itertools::Itertools;

lazy_static! {
    static ref ESCAPE_RE: Regex = Regex::new(r#"(["\n\\])"#).unwrap();
}
// The reverse of reader::read_str_transform
fn pr_str_transform(s: &str) -> String {
    let t = ESCAPE_RE
        .replace_all(&s, |caps: &regex::Captures| {
            format!("\\{}", if &caps[1] == "\n" { "n" } else { &caps[1] })
        })
        .to_string();
    format!("\"{}\"", t)
}

impl MalType {
    pub fn pr_str(self: &Self, print_readably: bool) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(b) => b.to_string(),
            MalType::Int(i) => i.to_string(),
            MalType::Str(s) => {
                if s.starts_with(KEYWORD_PREFIX) {
                    format!(":{}", &s[2..])
                } else if print_readably {
                    pr_str_transform(s)
                } else {
                    s.to_string()
                }
            }
            MalType::Symbol(s) => s.to_string(),
            MalType::List(l, _) => pr_list(l.deref(), "(", ")", print_readably, " "),
            MalType::Vector(l, _) => pr_list(l.deref(), "[", "]", print_readably, " "),
            MalType::HashMap(hm, _) => pr_list(
                &hm.iter()
                    .flat_map(|(k, v)| vec![k.clone(), v.clone()])
                    .collect(),
                "{",
                "}",
                print_readably,
                " ",
            ),
            MalType::Function(f) => format!("#<fn {:?}>", f),
            MalType::MalFunction { .. } => "#<function>".to_string(),
            MalType::Atom(a) => format!("(atom {})", a.borrow().to_string()),
        }
    }
}

pub fn pr_list(
    seq: &Vec<MalType>,
    open: &str,
    close: &str,
    print_readably: bool,
    join: &str,
) -> String {
    let inner = seq.iter().map(|el| el.pr_str(print_readably)).join(join);
    format!("{}{}{}", open, inner, close)
}
