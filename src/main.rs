use std::fs::File;
use std::io;
use std::path::Path;
use std::{env, io::Write};
mod parser;
use log::error;
mod builtin_cmd;
use std::sync::Mutex;

lazy_static::lazy_static! {
    /// 历史命令
    static ref HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn main() -> io::Result<()> {
    let err_log = File::create("hjpsh_err.log")?;
    env_logger::builder()
        .target(env_logger::fmt::Target::Pipe(Box::new(err_log)))
        .init();

    println!("########Welcom to hjpsh!########");
    loop {
        let cwd = env::current_dir()?;
        print!("[{}]> ", cwd.display());
        io::stdout().flush()?;

        // read command
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        match parser::parse_cmdline(line.trim()) {
            Ok(cmd) => {
                if let Err(err) = cmd.run() {
                    eprintln!("Error executing command: {:?}", err);
                    error!("Error executing command: {:?}", err);
                }
            }
            Err(err) => {
                eprintln!("Error parsing command: {:?}", err);
                error!("Error parsing command: {:?}", err);
            }
        }
        // record to history
        HISTORY.lock().unwrap().push(line);
    }
}
