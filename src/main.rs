use std::fs::File;
use std::io;
use std::path::Path;
use std::{env, io::Write};
mod lexer;
use lexer::lex;
mod parser;
use parser::parse;
mod eval;
use eval::ShellAST;
fn main() -> io::Result<()> {
    println!("########Welcom to hjpsh!########");

    let mut history = Vec::new();

    loop {
        let cwd = env::current_dir()?;
        print!("[{}]> ", cwd.display());
        io::stdout().flush()?;

        // read command
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        let args: Vec<_> = line.split_whitespace().collect();

        // push command to history
        let command = args.join(" ");
        history.push(command);

        let tokens = lex(line.as_str());
        let ast = parse(&tokens);
        let _ = ast.eval().map_err(|err| {
            eprintln!("{:?}", err);
        });
    }
}

mod buildin {
    use super::*;
    use env::set_current_dir;
    use io::{Read, Write};
    use std::fs;

    pub fn pwd2() -> io::Result<()> {
        let cwd = env::current_dir()?;
        println!("{}", cwd.display());
        Ok(())
    }
    pub fn cd2<P: AsRef<Path>>(path: P) -> io::Result<()> {
        set_current_dir(path)
    }
    pub fn ls2<P: AsRef<Path>>(path: P) -> io::Result<()> {
        // Read the directory
        let entries = fs::read_dir(path)?;

        // Iterate over the entries and print their names
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            println!("{}", file_name_str);
        }

        Ok(())
    }
    pub fn touch2<P: AsRef<Path>>(path: P) -> io::Result<File> {
        File::create(path)
    }
    pub fn echo2(str: &str) -> io::Result<()> {
        // whitespace is not allow
        io::stdout().write_all(str.as_bytes())?;
        io::stdout().flush()?;
        println!("");
        Ok(())
    }
    pub fn cat2<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        print!("{content}");
        Ok(())
    }
    pub fn cp2(from: &str, to: &str) -> io::Result<()> {
        fs::copy(from, to)?;
        Ok(())
    }
    pub fn rm(args: &[&str]) -> io::Result<()> {
        for file in args {
            fs::remove_file(file)?;
        }
        Ok(())
    }
    pub fn rm_recursive(dirs: &[&str]) -> io::Result<()> {
        for dir in dirs {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
    pub fn rename2(args: &[&str]) -> io::Result<()> {
        if args.len() == 3 {
            let old_path = &args[1];
            let new_path = &args[2];
            fs::rename(old_path, new_path)?;
        }
        Ok(())
    }
    pub fn history2(history: &Vec<String>) {
        for (i, cmd) in history.iter().enumerate() {
            println!("{} {}", i, cmd);
        }
    }
}
fn exec_cmd(args: &[&str], history: &Vec<String>) -> io::Result<()> {
    use buildin::*;
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
            Ok(())
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
