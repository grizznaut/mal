use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
mod printer;
mod reader;
mod types;
use types::MalType;

fn read(s: String) -> Result<MalType, &'static str> {
    reader::read_str(s)
}

fn eval(mt: MalType) -> MalType {
    mt
}

fn print(mt: MalType) -> String {
    mt.pr_str()
}

fn rep(s: String) -> Result<String, &'static str> {
    let r = read(s)?;
    let e = eval(r);
    let p = print(e);
    Ok(p)
}

fn main() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                let output = rep(line);
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
