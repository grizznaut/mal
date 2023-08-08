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

fn eval(mut ast: MalType, mut env: Rc<Env>) -> Result<MalType, MalErr> {
    let res: Result<MalType, MalErr>;

    loop {
        res = match ast.clone() {
            MalType::List(l, _) => {
                if l.len() == 0 {
                    return Ok(ast);
                }
                match l[0].to_string().as_str() {
                    "def!" => {
                        let result = eval(l[2].clone(), Rc::clone(&env))?;
                        env.set(l[1].to_string(), result.clone());
                        return Ok(result);
                    }
                    "let*" => {
                        let let_env = Rc::new(Env::new(Some(Rc::clone(&env))));
                        match &l[1] {
                            MalType::List(binding_list, _) | MalType::Vector(binding_list, _) => {
                                if binding_list.len() % 2 != 0 {
                                    return Err(MalErr::InvalidLet(
                                        "Odd number of parameters in the binding list".to_string(),
                                    ));
                                }
                                for w in binding_list.chunks(2) {
                                    let_env.set(
                                        w[0].to_string(),
                                        eval(w[1].clone(), Rc::clone(&let_env))?,
                                    );
                                }
                            }
                            _ => {
                                return Err(MalErr::InvalidLet(
                                    "let* expects a list or vector as the first parameter"
                                        .to_string(),
                                ))
                            }
                        };
                        ast = l[2].clone();
                        env = let_env;
                        continue;
                    }
                    "do" => match eval_ast(&list!(l[1..l.len() - 1].to_vec()), &env)? {
                        MalType::List(_, _) => {
                            ast = l.last().unwrap_or(&MalType::Nil).clone();
                            continue;
                        }
                        _ => Err(MalErr::InvalidDo("Invalid do construction".to_string())),
                    },
                    "if" => match eval(l[1].clone(), Rc::clone(&env))? {
                        MalType::Nil | MalType::Bool(false) => {
                            ast = l
                                .get(3)
                                .map_or(MalType::Nil, |else_branch| else_branch.clone());
                            continue;
                        }
                        _ => {
                            ast = l[2].clone();
                            continue;
                        }
                    },
                    "fn*" => match &l[1..] {
                        [params @ (MalType::List(..) | MalType::Vector(..)), body] => {
                            return Ok(MalType::MalFunction {
                                eval: eval,
                                params: Rc::new(params.clone()),
                                ast: Rc::new(body.clone()),
                                env: env,
                            });
                        }
                        _ => Err(MalErr::MalFunctionErr(
                            "fn* expects two parameters".to_string(),
                        )),
                    },
                    "eval" => {
                        ast = eval(l[1].clone(), Rc::clone(&env))?;
                        while let Some(ref e) = Rc::clone(&env).outer {
                            env = Rc::clone(&e);
                        }
                        continue;
                    }
                    _ => match eval_ast(&ast, &env)? {
                        MalType::List(ref el, _) => match el.split_first() {
                            Some((f, args)) => match f {
                                MalType::Function(_) => f.apply(args.to_vec()),
                                MalType::MalFunction {
                                    params,
                                    ast: mfast,
                                    env: mfenv,
                                    ..
                                } => {
                                    let fn_env = Rc::new(Env::new(Some(Rc::clone(&mfenv))));
                                    fn_env.bind((**params).clone(), args.to_vec())?;
                                    ast = (**mfast).clone();
                                    env = fn_env;
                                    continue;
                                }
                                _ => Err(MalErr::Generic("Cannot apply non-function".to_string())),
                            },
                            _ => Err(MalErr::Generic("Something bad happened".to_string())),
                        },
                        _ => Err(MalErr::Generic("Expected a list".to_string())),
                    },
                }
            }
            _ => eval_ast(&ast, &env),
        };

        break;
    }

    res
}

fn print(ast: MalType) -> String {
    ast.pr_str(true)
}

fn rep(s: &str, env: &Rc<Env>) -> Result<String, MalErr> {
    let r = read(s)?;
    let e = eval(r, Rc::clone(&env))?;
    let p = print(e);
    Ok(p)
}

fn eval_ast(ast: &MalType, env: &Rc<Env>) -> Result<MalType, MalErr> {
    match ast {
        MalType::Symbol(s) => env.get(s.as_str()),
        MalType::List(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), Rc::clone(&env))?);
            }
            Ok(list!(results))
        }
        MalType::Vector(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), Rc::clone(&env))?);
            }
            Ok(vector!(results))
        }
        MalType::HashMap(hm, _) => {
            let mut results = Vec::new();
            for (k, v) in hm.iter() {
                results.push(k.clone());
                results.push(eval(v.clone(), Rc::clone(&env))?);
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

    let mut args = std::env::args();
    let arg1 = args.nth(1); // The preceding and returned elements are consumed from args

    let repl_env = Rc::new(Env::default());
    for (symbol, value) in core::ns() {
        repl_env.set(symbol.to_string(), value);
    }
    // Add the rest of the command line arguments to your REPL environment so that
    // programs that are run with load-file have access to their calling environment
    repl_env.set(
        "*ARGV*".to_string(),
        list!(args.map(MalType::Str).collect()),
    );

    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);
    let _ = rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        &repl_env,
    );

    // If called with arguments, treat the first argument as a filename and use rep to call load-file on that filename,
    // and finally exit/terminate execution
    if let Some(f) = arg1 {
        match rep(&format!("(load-file \"{}\")", f), &repl_env) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
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
