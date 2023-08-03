use itertools::Itertools;

use crate::list;
use crate::types::MalType;
use std::collections::HashMap;

pub static KEYWORD_PREFIX: &'static str = "\u{29e}";

pub fn ns() -> HashMap<&'static str, MalType> {
    let mut ns = HashMap::new();
    ns.insert(
        "+",
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc + x.clone())
        }),
    );
    ns.insert(
        "-",
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc - x.clone())
        }),
    );
    ns.insert(
        "*",
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc * x.clone())
        }),
    );
    ns.insert(
        "/",
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc / x.clone())
        }),
    );
    ns.insert("=", MalType::Function(|a| MalType::Bool(&a[0] == &a[1])));
    ns.insert("<", MalType::Function(|a| MalType::Bool(&a[0] < &a[1])));
    ns.insert("<=", MalType::Function(|a| MalType::Bool(&a[0] <= &a[1])));
    ns.insert(">", MalType::Function(|a| MalType::Bool(&a[0] > &a[1])));
    ns.insert(">=", MalType::Function(|a| MalType::Bool(&a[0] >= &a[1])));
    ns.insert(
        "pr-str",
        MalType::Function(|a| MalType::Str(a.iter().map(|el| el.pr_str(true)).join(" "))),
    );
    ns.insert(
        "str",
        MalType::Function(|a| MalType::Str(a.iter().map(|el| el.pr_str(false)).join(""))),
    );
    ns.insert(
        "prn",
        MalType::Function(|a| {
            println!("{}", a.iter().map(|el| el.pr_str(true)).join(" "));
            MalType::Nil
        }),
    );
    ns.insert(
        "println",
        MalType::Function(|a| {
            println!("{}", a.iter().map(|el| el.pr_str(false)).join(" "));
            MalType::Nil
        }),
    );
    ns.insert("list", MalType::Function(|a| list!(a)));
    ns.insert(
        "list?",
        MalType::Function(|a| match &a[0] {
            MalType::List(..) => MalType::Bool(true),
            _ => MalType::Bool(false),
        }),
    );
    ns.insert(
        "empty?",
        MalType::Function(|a| match &a[0] {
            MalType::List(l, _) | MalType::Vector(l, _) => MalType::Bool(l.is_empty()),
            _ => MalType::Bool(false),
        }),
    );
    ns.insert(
        "count",
        MalType::Function(|a| match &a[0] {
            MalType::List(l, _) | MalType::Vector(l, _) => MalType::Int(l.len() as i64),
            _ => MalType::Int(0),
        }),
    );
    ns
}
