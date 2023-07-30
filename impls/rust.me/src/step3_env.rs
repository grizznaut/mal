use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
mod env;
use env::Env;
mod errors;
use errors::MalErr;
mod printer;
mod reader;
mod types;
use types::MalType;

fn read(s: String) -> Result<MalType, MalErr> {
    reader::read_str(s)
}

fn eval(ast: MalType, env: &mut Env) -> Result<MalType, MalErr> {
    match ast.clone() {
        MalType::List(l, _) => {
            if l.len() == 0 {
                return Ok(ast);
            }
            match l[0].to_string().as_str() {
                "def!" => {
                    let result = eval(l[2].clone(), env)?;
                    env.set(l[1].to_string(), result.clone());
                    Ok(result)
                }
                "let*" => {
                    let mut new_env = Env::new(Some(env.clone()));
                    match &l[1] {
                        MalType::List(binding_list, _) | MalType::Vector(binding_list, _) => {
                            if binding_list.len() % 2 != 0 {
                                return Err(MalErr::InvalidLet(
                                    "Odd number of parameters in the binding list".to_string(),
                                ));
                            }
                            for w in binding_list.chunks(2) {
                                new_env.set(
                                    w[0].to_string(),
                                    eval(w[1].clone(), &mut new_env.clone())?,
                                );
                            }
                        }
                        _ => {
                            return Err(MalErr::InvalidLet(
                                "let* expects a list or vector as the first parameter".to_string(),
                            ))
                        }
                    };
                    eval(l[2].clone(), &mut new_env.clone())
                }
                _ => match eval_ast(&ast, env)? {
                    MalType::List(ref el, _) => match el.split_first() {
                        Some((f, args)) => f.apply(args.to_vec()),
                        _ => Err(MalErr::Generic("Something bad happened".to_string())),
                    },
                    _ => Err(MalErr::Generic("Expected a list".to_string())),
                },
            }
        }
        _ => eval_ast(&ast, env),
    }
}

fn print(ast: MalType) -> String {
    ast.pr_str()
}

fn rep(s: String, env: &mut Env) -> Result<String, MalErr> {
    let r = read(s)?;
    let e = eval(r, env)?;
    let p = print(e);
    Ok(p)
}

fn eval_ast(ast: &MalType, env: &mut Env) -> Result<MalType, MalErr> {
    match ast {
        MalType::Symbol(s) => env.get(s.as_str()),
        MalType::List(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), env)?);
            }
            Ok(list!(results))
        }
        MalType::Vector(l, _) => {
            let mut results = Vec::new();
            for ast in l.iter() {
                results.push(eval(ast.clone(), env)?);
            }
            Ok(vector!(results))
        }
        MalType::HashMap(hm, _) => {
            let mut results = Vec::new();
            for (k, v) in hm.iter() {
                results.push(k.clone());
                results.push(eval(v.clone(), env)?);
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

    let mut repl_env: Env = Env::default();
    repl_env.set(
        "+".to_string(),
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc + x.clone())
        }),
    );
    repl_env.set(
        "-".to_string(),
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc - x.clone())
        }),
    );
    repl_env.set(
        "*".to_string(),
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc * x.clone())
        }),
    );
    repl_env.set(
        "/".to_string(),
        MalType::Function(|a| {
            a.iter()
                .skip(1)
                .fold(a[0].clone(), |acc, x| acc / x.clone())
        }),
    );

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                let output = rep(line, &mut repl_env);
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
