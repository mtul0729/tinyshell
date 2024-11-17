use std::fs::File;
use std::io;
use std::path::Path;
use std::{env, io::Write};
mod parser;
// use parser::parse;
// use eval::ShellAST;
use log::{debug, error, info, log_enabled, Level};
mod builtin_cmd;

fn main() -> io::Result<()> {
    let err_log = File::create("hjpsh_err.log")?;
    env_logger::builder()
        .target(env_logger::fmt::Target::Pipe(Box::new(err_log)))
        .init();

    println!("########Welcom to hjpsh!########");

    let mut history = Vec::new();

    loop {
        let cwd = env::current_dir()?;
        print!("[{}]> ", cwd.display());
        io::stdout().flush()?;

        // read command
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        // let tokens = lex(line.as_str());
        // let ast = parse(&tokens);
        // let _ = ast.eval().map_err(|err| {
        //     eprintln!("{:?}", err);
        //     error!("{:?}", err);
        // });
        // record to history
        history.push(line);
    }
}

fn exec_cmd(args: &[&str], history: &Vec<String>) -> io::Result<()> {
    use builtin_cmd::*;
    match *args.first().unwrap() {
        "pwd2" => pwd2(),
        "cd2" => match args.get(1) {
            Some(path) => cd2(path),
            None => cd2(home::home_dir().unwrap()),
        },

        "ls2" => {
            let path = args.get(1).unwrap_or(&".");
            ls2(path)
        }
        "touch2" => {
            if args.len() == 1 {
                eprintln!("missing file operand");
            } else if args.len() == 2 {
                touch2(args[1])?;
            } else {
                eprintln!("too many arguments");
            }
            Ok(())
        }
        "echo2" => match args[1..] {
            [] => echo2(""),
            [s] => echo2(s),
            _ => {
                eprintln!("echo: too many arguments");
                Ok(())
            }
        },
        "cat2" => match args.get(1) {
            Some(path) => cat2(path),
            None => {
                eprintln!("missing file operand");
                Ok(())
            }
        },
        "cp2" => {
            match args[1..] {
                [] => {
                    eprintln!("missing file operand");
                }
                [from] => {
                    eprintln!("missing destination file operand after '{}'", from);
                }
                [from, to] => cp2(from, to)?,
                _ => {
                    eprintln!("too many arguments");
                }
            }
            Ok(())
        }
        "rm2" => {
            match &args[1..] {
                [] => {
                    eprintln!("missing file operand");
                }
                ["-r", ..] => {
                    rm_recursive(&args[2..])?;
                }
                files => {
                    rm(files)?;
                }
            }
            Ok(())
        }
        "rename2" => {
            if args.len() == 1 {
                eprintln!("missing file operand");
            } else if args.len() == 2 {
                eprintln!("missing destination file operand after '{}'", args[1]);
            } else if args.len() == 3 {
                rename2(args)?;
            } else {
                eprintln!("too many arguments");
            }
            Ok(())
        }
        "history2" => {
            history2(history);
            Ok(())
        }
        "quit" => {
            println!("########Quiting hjpsh########");
            std::process::exit(0)
        }
        cmd => {
            use std::process::Command;
            let status = Command::new(cmd).args(&args[1..]).status()?;
            if !status.success() {
                eprintln!("{}: command not found", cmd);
            }
            Ok(())
        }
    }
}
