use crate::errors::MalErr;
use crate::printer::pr_list;
use crate::reader::read_str;
use crate::types::{atom, func, MalType};
use crate::{hashmap, list, vector};

use lazy_static::lazy_static;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub static KEYWORD_PREFIX: &'static str = "\u{29e}";

fn accumulate(args: Vec<MalType>, op: fn(MalType, MalType) -> MalType) -> Result<MalType, MalErr> {
    if args.len() < 2 {
        return Err(MalErr::FunctionErr(
            "Expected two or more arguments".to_string(),
        ));
    }
    Ok(args
        .iter()
        .skip(1)
        .fold(args[0].clone(), |acc, x| op(acc, x.clone())))
}

fn compare(args: Vec<MalType>, op: fn(&MalType, &MalType) -> bool) -> Result<MalType, MalErr> {
    if args.len() != 2 {
        return Err(MalErr::FunctionErr(
            "Expected exactly two arguments".to_string(),
        ));
    }
    Ok(MalType::Bool(op(&args[0], &args[1])))
}

fn make_string(args: Vec<MalType>, print_readably: bool, join: &str) -> Result<MalType, MalErr> {
    Ok(MalType::Str(pr_list(&args, "", "", print_readably, join)))
}

fn print_string(args: Vec<MalType>, print_readably: bool) -> Result<MalType, MalErr> {
    println!("{}", pr_list(&args, "", "", print_readably, " "));
    Ok(MalType::Nil)
}

fn slurp(f: String) -> Result<MalType, MalErr> {
    let mut s = String::new();
    match File::open(f).and_then(|mut f| f.read_to_string(&mut s)) {
        Ok(_) => Ok(MalType::Str(s)),
        Err(e) => Err(MalErr::FunctionErr(e.to_string())),
    }
}

fn read_string(
    args: Vec<MalType>,
    reader: fn(String) -> Result<MalType, MalErr>,
) -> Result<MalType, MalErr> {
    match &args[0] {
        MalType::Str(s) => reader(s.clone()),
        _ => Err(MalErr::FunctionErr("Expected a string".to_string())),
    }
}

fn is_variant(value: &MalType, variant: &str) -> Result<MalType, MalErr> {
    let is_type = match (value, variant) {
        (MalType::List(..), "list") => true,
        (MalType::Atom(..), "atom") => true,
        (MalType::Nil, "nil") => true,
        (MalType::Bool(true), "true") => true,
        (MalType::Bool(false), "false") => true,
        (MalType::Symbol(..), "symbol") => true,
        (MalType::Str(s), "string") => !s.starts_with(KEYWORD_PREFIX),
        (MalType::Str(s), "keyword") => s.starts_with(KEYWORD_PREFIX),
        (MalType::Int(..), "number") => true,
        (MalType::MalFunction { is_macro, .. }, "macro") => *is_macro,
        (MalType::MalFunction { is_macro, .. }, "function") => !*is_macro,
        (MalType::Function(..), "function") => true,
        (MalType::Vector(..), "vector") => true,
        (MalType::List(..) | MalType::Vector(..), "sequential") => true,
        (MalType::HashMap(..), "hashmap") => true,
        (MalType::List(l, _) | MalType::Vector(l, _), "empty") => l.is_empty(),
        _ => false,
    };
    Ok(MalType::Bool(is_type))
}

fn symbol(value: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::Str(s) => Ok(MalType::Symbol(s.to_string())),
        _ => Err(MalErr::FunctionErr("Expected a string".to_string())),
    }
}

fn keyword(value: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::Str(s) if s.starts_with(KEYWORD_PREFIX) => Ok(value.clone()),
        MalType::Str(s) => Ok(MalType::Str(KEYWORD_PREFIX.to_owned() + s)),
        _ => Err(MalErr::FunctionErr("Expected a string".to_string())),
    }
}

fn contains(value: &MalType, key: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::HashMap(hm, _) => Ok(MalType::Bool(hm.contains_key(key))),
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn get(value: &MalType, key: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::HashMap(hm, _) => Ok(hm.get(key).unwrap_or(&MalType::Nil).clone()),
        MalType::Nil => Ok(MalType::Nil),
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn keys(value: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::HashMap(hm, _) => Ok(list!(hm.keys().map(|k| k.clone()).collect())),
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn vals(value: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::HashMap(hm, _) => Ok(list!(hm.values().map(|v| v.clone()).collect())),
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn assoc(args: Vec<MalType>) -> Result<MalType, MalErr> {
    match &args[0] {
        MalType::HashMap(hm, _) => {
            let mut kvs: Vec<MalType> = (**hm)
                .clone()
                .into_iter()
                .flat_map(|(k, v)| vec![k, v])
                .collect();
            kvs.extend_from_slice(&args[1..]);
            hashmap!(kvs)
        }
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn dissoc(args: Vec<MalType>) -> Result<MalType, MalErr> {
    match &args[0] {
        MalType::HashMap(hm, _) => {
            let mut new_hm = (**hm).clone();
            for key in &args[1..] {
                new_hm.remove(key);
            }
            Ok(MalType::HashMap(
                std::rc::Rc::new(new_hm),
                std::rc::Rc::new(MalType::Nil),
            ))
        }
        _ => Err(MalErr::FunctionErr("Expected a hash-map".to_string())),
    }
}

fn deref(atom: &MalType) -> Result<MalType, MalErr> {
    match atom {
        MalType::Atom(a) => Ok(a.borrow().clone()),
        _ => Err(MalErr::FunctionErr("Cannot deref a non-atom".to_string())),
    }
}

fn reset(atom: &MalType, new_val: &MalType) -> Result<MalType, MalErr> {
    match atom {
        MalType::Atom(a) => {
            *a.borrow_mut() = new_val.to_owned();
            Ok(a.borrow().clone())
        }
        _ => Err(MalErr::FunctionErr("Cannot reset a non-atom".to_string())),
    }
}

fn swap(atom: &MalType, f: &MalType, optargs: Vec<MalType>) -> Result<MalType, MalErr> {
    match atom {
        MalType::Atom(a) => {
            let mut args = optargs;
            args.insert(0, a.borrow().clone());
            *a.borrow_mut() = f.apply(args)?;
            Ok(a.borrow().clone())
        }
        _ => Err(MalErr::FunctionErr("Cannot swap a non-atom".to_string())),
    }
}

fn cons(args: Vec<MalType>) -> Result<MalType, MalErr> {
    if args.len() != 2 {
        return Err(MalErr::FunctionErr(
            "Expected exactly two arguments".to_string(),
        ));
    }
    match &args[1] {
        MalType::List(l, _) | MalType::Vector(l, _) => {
            let mut v = vec![args[0].clone()];
            v.extend_from_slice(l);
            Ok(list!(v))
        }
        _ => Err(MalErr::FunctionErr(
            "Expected a list/vector as the second parameter to cons".to_string(),
        )),
    }
}

fn concat(args: Vec<MalType>) -> Result<MalType, MalErr> {
    let mut v = Vec::new();
    for a in args.iter() {
        match a {
            MalType::List(l, _) | MalType::Vector(l, _) => v.extend_from_slice(l),
            _ => {
                return Err(MalErr::FunctionErr(
                    "concat does not support non-list items".to_string(),
                ))
            }
        }
    }
    Ok(list!(v))
}

fn vec(args: Vec<MalType>) -> Result<MalType, MalErr> {
    match &args[0] {
        MalType::List(l, _) | MalType::Vector(l, _) => Ok(vector!(l.to_vec())),
        _ => Err(MalErr::FunctionErr(
            "Expected a list/vector to convert into vector".to_string(),
        )),
    }
}

fn nth(list: &MalType, index: &MalType) -> Result<MalType, MalErr> {
    match (list, index) {
        (MalType::List(l, _) | MalType::Vector(l, _), MalType::Int(i)) => {
            match l.get(*i as usize) {
                Some(el) => Ok(el.clone()),
                None => Err(MalErr::FunctionErr("list index out of range".to_string())),
            }
        }
        _ => Err(MalErr::FunctionErr("Expected a list and index".to_string())),
    }
}

fn first(list: &MalType) -> Result<MalType, MalErr> {
    match nth(list, &MalType::Int(0)) {
        Ok(el) => Ok(el),
        Err(_) => Ok(MalType::Nil),
    }
}

fn rest(list: &MalType) -> Result<MalType, MalErr> {
    match list {
        MalType::List(l, _) | MalType::Vector(l, _) => {
            Ok(list!(l.get(1..).unwrap_or_default().to_vec()))
        }
        _ => Ok(list!(vec![])),
    }
}

fn apply(args: Vec<MalType>) -> Result<MalType, MalErr> {
    let mut fargs = args.iter();
    let (f, list) = (fargs.nth(0), fargs.nth_back(0)); // consumes the first and last iter items
    match list {
        Some(MalType::List(l, _)) | Some(MalType::Vector(l, _)) => {
            let mut v: Vec<MalType> = fargs.cloned().collect();
            v.extend_from_slice(l);
            f.unwrap().apply(v)
        }
        _ => Err(MalErr::FunctionErr("Expected a list of args".to_string())),
    }
}

fn map(args: Vec<MalType>) -> Result<MalType, MalErr> {
    let f = &args[0];
    match &args[1] {
        MalType::List(l, _) | MalType::Vector(l, _) => {
            let map_results = l.iter().map(|el| f.apply(vec![el.clone()])).collect();
            match map_results {
                Ok(m) => Ok(list!(m)),
                Err(e) => Err(e),
            }
        }
        _ => Err(MalErr::FunctionErr("Expected a list of args".to_string())),
    }
}

fn time() -> Result<MalType, MalErr> {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    Ok(MalType::Int(since_epoch.as_millis() as i64))
}

lazy_static! {
    static ref RL: Mutex<DefaultEditor> = Mutex::new(DefaultEditor::new().unwrap());
}
fn readline(prompt: &MalType) -> Result<MalType, MalErr> {
    match prompt {
        MalType::Str(p) => {
            let readline = RL.lock().unwrap().readline(p);
            match readline {
                Ok(line) => Ok(MalType::Str(line)),
                Err(ReadlineError::Eof) => Ok(MalType::Nil),
                Err(e) => Err(MalErr::FunctionErr(format!(
                    "readline error: {}",
                    e.to_string()
                ))),
            }
        }
        _ => Err(MalErr::FunctionErr(
            "readline prompt should be a string".to_string(),
        )),
    }
}

fn conj(args: Vec<MalType>) -> Result<MalType, MalErr> {
    match &args[0] {
        MalType::List(l, _) => {
            let mut new_list = (**l).clone();
            for a in &args[1..] {
                new_list.insert(0, a.clone());
            }
            Ok(list!(new_list))
        }
        MalType::Vector(v, _) => {
            let mut new_vec = (**v).clone();
            new_vec.extend_from_slice(&args[1..]);
            Ok(vector!(new_vec))
        }
        _ => Err(MalErr::FunctionErr("expected a list or vector".to_string())),
    }
}

fn seq(value: &MalType) -> Result<MalType, MalErr> {
    match value {
        MalType::List(l, _) | MalType::Vector(l, _) if l.is_empty() => Ok(MalType::Nil),
        MalType::List(l, _) | MalType::Vector(l, _) => Ok(list!(l.to_vec())),
        MalType::Str(s) if s.is_empty() => Ok(MalType::Nil),
        MalType::Str(s) => Ok(list!(s
            .chars()
            .map(|c| { MalType::Str(c.to_string()) })
            .collect())),
        MalType::Nil => Ok(MalType::Nil),
        _ => Err(MalErr::FunctionErr("invalid value for seq".to_string())),
    }
}

pub fn ns() -> HashMap<&'static str, MalType> {
    let mut ns = HashMap::new();
    ns.insert("+", func(|a| accumulate(a, |x, y| x + y)));
    ns.insert("-", func(|a| accumulate(a, |x, y| x - y)));
    ns.insert("*", func(|a| accumulate(a, |x, y| x * y)));
    ns.insert("/", func(|a| accumulate(a, |x, y| x / y)));
    ns.insert("=", func(|a| compare(a, |x, y| x == y)));
    ns.insert("<", func(|a| compare(a, |x, y| x < y)));
    ns.insert("<=", func(|a| compare(a, |x, y| x <= y)));
    ns.insert(">", func(|a| compare(a, |x, y| x > y)));
    ns.insert(">=", func(|a| compare(a, |x, y| x >= y)));
    ns.insert("pr-str", func(|a| make_string(a, true, " ")));
    ns.insert("str", func(|a| make_string(a, false, "")));
    ns.insert("prn", func(|a| print_string(a, true)));
    ns.insert("println", func(|a| print_string(a, false)));
    ns.insert("read-string", func(|a| read_string(a, read_str)));
    ns.insert("slurp", func(|a| read_string(a, slurp)));
    ns.insert("list", func(|a| Ok(list!(a))));
    ns.insert("list?", func(|a| is_variant(&a[0], "list")));
    ns.insert("empty?", func(|a| is_variant(&a[0], "empty")));
    ns.insert("nil?", func(|a| is_variant(&a[0], "nil")));
    ns.insert("true?", func(|a| is_variant(&a[0], "true")));
    ns.insert("false?", func(|a| is_variant(&a[0], "false")));
    ns.insert("symbol", func(|a| symbol(&a[0])));
    ns.insert("symbol?", func(|a| is_variant(&a[0], "symbol")));
    ns.insert("keyword", func(|a| keyword(&a[0])));
    ns.insert("keyword?", func(|a| is_variant(&a[0], "keyword")));
    ns.insert("vector", func(|a| Ok(vector!(a))));
    ns.insert("vector?", func(|a| is_variant(&a[0], "vector")));
    ns.insert("sequential?", func(|a| is_variant(&a[0], "sequential")));
    ns.insert("hash-map", func(|a| hashmap!(a)));
    ns.insert("map?", func(|a| is_variant(&a[0], "hashmap")));
    ns.insert("contains?", func(|a| contains(&a[0], &a[1])));
    ns.insert("get", func(|a| get(&a[0], &a[1])));
    ns.insert("keys", func(|a| keys(&a[0])));
    ns.insert("vals", func(|a| vals(&a[0])));
    ns.insert("assoc", func(|a| assoc(a)));
    ns.insert("dissoc", func(|a| dissoc(a)));
    ns.insert(
        "count",
        func(|a| match &a[0] {
            MalType::List(l, _) | MalType::Vector(l, _) => Ok(MalType::Int(l.len() as i64)),
            _ => Ok(MalType::Int(0)),
        }),
    );
    ns.insert("atom", func(|a| Ok(atom(&a[0]))));
    ns.insert("atom?", func(|a| is_variant(&a[0], "atom")));
    ns.insert("deref", func(|a| deref(&a[0])));
    ns.insert("reset!", func(|a| reset(&a[0], &a[1])));
    ns.insert(
        "swap!",
        func(|a| swap(&a[0], &a[1], a.get(2..).unwrap_or_default().to_vec())),
    );
    ns.insert("cons", func(|a| cons(a)));
    ns.insert("concat", func(|a| concat(a)));
    ns.insert("vec", func(|a| vec(a)));
    ns.insert("nth", func(|a| nth(&a[0], &a[1])));
    ns.insert("first", func(|a| first(&a[0])));
    ns.insert("rest", func(|a| rest(&a[0])));
    ns.insert("throw", func(|a| Err(MalErr::Throw(a[0].clone()))));
    ns.insert("apply", func(|a| apply(a)));
    ns.insert("map", func(|a| map(a)));
    ns.insert("readline", func(|a| readline(&a[0])));
    ns.insert("time-ms", func(|_| time()));
    ns.insert("meta", func(|a| (&a[0]).get_meta()));
    ns.insert("with-meta", func(|a| (a[0].clone()).set_meta(&a[1])));
    ns.insert("number?", func(|a| is_variant(&a[0], "number")));
    ns.insert("string?", func(|a| is_variant(&a[0], "string")));
    ns.insert("fn?", func(|a| is_variant(&a[0], "function")));
    ns.insert("macro?", func(|a| is_variant(&a[0], "macro")));
    ns.insert("conj", func(|a| conj(a)));
    ns.insert("seq", func(|a| seq(&a[0])));
    ns
}
