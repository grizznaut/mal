use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
mod errors;
use crate::errors::MalErr;
mod printer;
mod reader;
mod types;
use types::MalType;

fn read(s: String) -> Result<MalType, MalErr> {
    reader::read_str(s)
}

fn eval(ast: MalType, repl_env: &ReplEnv) -> Result<MalType, MalErr> {
    match ast.clone() {
        MalType::List(l, _) => {
            if l.len() == 0 {
                return Ok(ast);
            }
            match eval_ast(&ast, &repl_env)? {
                MalType::List(ref el, _) => match el.split_first() {
                    Some((f, args)) => f.apply(args.to_vec()),
                    _ => Err(MalErr::Generic("Something bad happened".to_string())),
                },
                _ => Err(MalErr::Generic("Expected a list".to_string())),
            }
        }
        _ => eval_ast(&ast, repl_env),
    }
}

fn print(ast: MalType) -> String {
    ast.pr_str()
}

fn rep(s: String, repl_env: &ReplEnv) -> Result<String, MalErr> {
    let r = read(s)?;
    let e = eval(r, &repl_env)?;
    let p = print(e);
    Ok(p)
}

type ReplEnv = HashMap<&'static str, fn(Vec<MalType>) -> MalType>;

fn eval_ast(ast: &MalType, repl_env: &ReplEnv) -> Result<MalType, MalErr> {
    match ast {
        MalType::Symbol(s) => match repl_env.get(s.as_str()) {
            Some(f) => Ok(MalType::Function(*f)),
            None => Err(MalErr::SymbolNotFound("Invalid symbol {e:}".to_string())),
        },
        MalType::List(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), &repl_env)?);
            }
            Ok(list!(results))
        }
        MalType::Vector(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), &repl_env)?);
            }
            Ok(vector!(results))
        }
        MalType::HashMap(hm, _) => {
            let mut results = Vec::new();
            for (k, v) in hm.iter() {
                results.push(k.clone());
                results.push(eval(v.clone(), &repl_env)?);
            }
            hashmap!(results)
        }
        _ => Ok(ast.clone()),
    }
}

fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut repl_env: ReplEnv = ReplEnv::new();
    repl_env.insert("+", |a| {
        a.iter()
            .skip(1)
            .fold(a[0].clone(), |acc, x| acc + x.clone())
    });
    repl_env.insert("-", |a| {
        a.iter()
            .skip(1)
            .fold(a[0].clone(), |acc, x| acc - x.clone())
    });
    repl_env.insert("*", |a| {
        a.iter()
            .skip(1)
            .fold(a[0].clone(), |acc, x| acc * x.clone())
    });
    repl_env.insert("/", |a| {
        a.iter()
            .skip(1)
            .fold(a[0].clone(), |acc, x| acc / x.clone())
    });

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                let output = rep(line, &repl_env);
                match output {
                    Ok(val) => println!("{}", val),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    Ok(())
}
