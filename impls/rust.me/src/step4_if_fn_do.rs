use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::rc::Rc;

mod core;
mod env;
use env::Env;
mod errors;
use errors::MalErr;
mod printer;
mod reader;
mod types;
use types::MalType;

fn read(s: &str) -> Result<MalType, MalErr> {
    reader::read_str(s.to_string())
}

fn eval(ast: MalType, env: Rc<Env>) -> Result<MalType, MalErr> {
    match ast.clone() {
        MalType::List(l, _) => {
            if l.len() == 0 {
                return Ok(ast);
            }
            match l[0].to_string().as_str() {
                "def!" => {
                    let result = eval(l[2].clone(), env.clone())?;
                    env.set(l[1].to_string(), result.clone());
                    Ok(result)
                }
                "let*" => {
                    let let_env = Rc::new(Env::new(Some(env.clone())));
                    match &l[1] {
                        MalType::List(binding_list, _) | MalType::Vector(binding_list, _) => {
                            if binding_list.len() % 2 != 0 {
                                return Err(MalErr::InvalidLet(
                                    "Odd number of parameters in the binding list".to_string(),
                                ));
                            }
                            for w in binding_list.chunks(2) {
                                let_env.set(w[0].to_string(), eval(w[1].clone(), let_env.clone())?);
                            }
                        }
                        _ => {
                            return Err(MalErr::InvalidLet(
                                "let* expects a list or vector as the first parameter".to_string(),
                            ))
                        }
                    };
                    eval(l[2].clone(), let_env)
                }
                "do" => match eval_ast(&list!(l[1..].to_vec()), &env)? {
                    MalType::List(el, _) => Ok(el.last().unwrap_or(&MalType::Nil).clone()),
                    _ => Err(MalErr::InvalidDo("Invalid do construction".to_string())),
                },
                "if" => match eval(l[1].clone(), env.clone())? {
                    MalType::Nil | MalType::Bool(false) => {
                        l.get(3).map_or(Ok(MalType::Nil), |else_branch| {
                            eval(else_branch.clone(), env.clone())
                        })
                    }
                    _ => eval(l[2].clone(), env.clone()),
                },
                "fn*" => match &l[1..] {
                    [params @ (MalType::List(..) | MalType::Vector(..)), body] => {
                        Ok(MalType::MalFunction {
                            eval: eval,
                            params: Rc::new(params.clone()),
                            body: Rc::new(body.clone()),
                            env: env,
                        })
                    }
                    _ => Err(MalErr::MalFunctionErr(
                        "fn* expects two parameters".to_string(),
                    )),
                },
                _ => match eval_ast(&ast, &env)? {
                    MalType::List(ref el, _) => match el.split_first() {
                        Some((f, args)) => f.apply(args.to_vec()),
                        _ => Err(MalErr::Generic("Something bad happened".to_string())),
                    },
                    _ => Err(MalErr::Generic("Expected a list".to_string())),
                },
            }
        }
        _ => eval_ast(&ast, &env),
    }
}

fn print(ast: MalType) -> String {
    ast.pr_str()
}

fn rep(s: &str, env: &Rc<Env>) -> Result<String, MalErr> {
    let r = read(s)?;
    let e = eval(r, env.clone())?;
    let p = print(e);
    Ok(p)
}

fn eval_ast(ast: &MalType, env: &Rc<Env>) -> Result<MalType, MalErr> {
    match ast {
        MalType::Symbol(s) => env.get(s.as_str()),
        MalType::List(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), env.clone())?);
            }
            Ok(list!(results))
        }
        MalType::Vector(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), env.clone())?);
            }
            Ok(vector!(results))
        }
        MalType::HashMap(hm, _) => {
            let mut results = Vec::new();
            for (k, v) in hm.iter() {
                results.push(k.clone());
                results.push(eval(v.clone(), env.clone())?);
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

    let repl_env = Rc::new(Env::default());
    for (symbol, value) in core::ns() {
        repl_env.set(symbol.to_string(), value);
    }

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
                let output = rep(&line, &repl_env);
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
