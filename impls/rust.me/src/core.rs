use crate::errors::MalErr;
use crate::list;
use crate::printer::pr_list;
use crate::reader::read_str;
use crate::types::{atom, MalType};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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
        (MalType::List(l, _) | MalType::Vector(l, _), "empty") => l.is_empty(),
        _ => false,
    };
    Ok(MalType::Bool(is_type))
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

pub fn ns() -> HashMap<&'static str, MalType> {
    let mut ns = HashMap::new();
    ns.insert("+", MalType::Function(|a| accumulate(a, |x, y| x + y)));
    ns.insert("-", MalType::Function(|a| accumulate(a, |x, y| x - y)));
    ns.insert("*", MalType::Function(|a| accumulate(a, |x, y| x * y)));
    ns.insert("/", MalType::Function(|a| accumulate(a, |x, y| x / y)));
    ns.insert("=", MalType::Function(|a| compare(a, |x, y| x == y)));
    ns.insert("<", MalType::Function(|a| compare(a, |x, y| x < y)));
    ns.insert("<=", MalType::Function(|a| compare(a, |x, y| x <= y)));
    ns.insert(">", MalType::Function(|a| compare(a, |x, y| x > y)));
    ns.insert(">=", MalType::Function(|a| compare(a, |x, y| x >= y)));
    ns.insert("pr-str", MalType::Function(|a| make_string(a, true, " ")));
    ns.insert("str", MalType::Function(|a| make_string(a, false, "")));
    ns.insert("prn", MalType::Function(|a| print_string(a, true)));
    ns.insert("println", MalType::Function(|a| print_string(a, false)));
    ns.insert(
        "read-string",
        MalType::Function(|a| read_string(a, read_str)),
    );
    ns.insert("slurp", MalType::Function(|a| read_string(a, slurp)));
    ns.insert("list", MalType::Function(|a| Ok(list!(a))));
    ns.insert("list?", MalType::Function(|a| is_variant(&a[0], "list")));
    ns.insert("empty?", MalType::Function(|a| is_variant(&a[0], "empty")));
    ns.insert(
        "count",
        MalType::Function(|a| match &a[0] {
            MalType::List(l, _) | MalType::Vector(l, _) => Ok(MalType::Int(l.len() as i64)),
            _ => Ok(MalType::Int(0)),
        }),
    );
    ns.insert("atom", MalType::Function(|a| Ok(atom(&a[0]))));
    ns.insert("atom?", MalType::Function(|a| is_variant(&a[0], "atom")));
    ns.insert("deref", MalType::Function(|a| deref(&a[0])));
    ns.insert("reset!", MalType::Function(|a| reset(&a[0], &a[1])));
    ns.insert(
        "swap!",
        MalType::Function(|a| swap(&a[0], &a[1], a.get(2..).unwrap_or_default().to_vec())),
    );
    ns
}
