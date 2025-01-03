use anyhow::Ok;
use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;
use std::process::{Child, Command, Stdio};

#[derive(Parser)]
#[grammar = "shell.pest"]
struct ShellParser;

#[derive(Debug)]
pub struct SimpleCommand<'a> {
    pub command_name: &'a str,
    pub args: Vec<&'a str>,
}

#[derive(Debug)]
struct RedirectFile<'a> {
    file: &'a str,
    append: bool,
}

#[derive(Debug)]
pub struct ExternCommandLine<'a> {
    piped_cmds: Vec<SimpleCommand<'a>>,
    redirect: Option<RedirectFile<'a>>,
}

impl ExternCommandLine<'_> {
    pub fn run(self) -> std::io::Result<()> {
        fn run_extern_cmd(
            piped_cmds: &[SimpleCommand],
            input: Stdio,
            tail_stdout: Stdio,
        ) -> std::io::Result<Child> {
            match piped_cmds {
                [cmd] => Command::new(cmd.command_name)
                    .args(&cmd.args)
                    .stdin(input)
                    .stdout(tail_stdout)
                    .spawn(),
                [cmd, rest @ ..] => {
                    let mut child = Command::new(cmd.command_name)
                        .args(&cmd.args)
                        .stdin(input)
                        .stdout(Stdio::piped())
                        .spawn()?;
                    let output = child.stdout.take().unwrap();
                    run_extern_cmd(rest, output.into(), tail_stdout)
                }
                _ => unreachable!(),
            }
        }
        let input = Stdio::inherit();
        let tail_stdout = match self.redirect {
            Some(redirect) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(redirect.append)
                    .write(true)
                    .open(redirect.file)?;
                Stdio::from(file)
            }
            None => Stdio::inherit(),
        };
        let piped_cmds = self.piped_cmds.as_slice();
        let mut child = run_extern_cmd(piped_cmds, input, tail_stdout)?;
        child.wait()?;
        Result::Ok(())
    }
}

#[derive(Debug)]
pub enum CommandLine<'a> {
    Extern(ExternCommandLine<'a>),
    Builtin(SimpleCommand<'a>),
}

use crate::builtin_cmd::exec_builtin;
impl CommandLine<'_> {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            CommandLine::Extern(extern_cmd) => extern_cmd.run(),
            CommandLine::Builtin(builtin_cmd) => {
                exec_builtin(builtin_cmd.command_name, &builtin_cmd.args)
            }
        }
    }
}

pub fn parse_cmdline(cmd: &str) -> Result<CommandLine> {
    let cmd_line = ShellParser::parse(Rule::command_line, cmd)?
        .next()
        .ok_or(anyhow::Error::msg("不合法的命令行"))? // None implies legal command_line not found
        .into_inner()
        .next()
        .unwrap(); // builtin_cmd or outer_cmd

    match cmd_line.as_rule() {
        Rule::builtin_cmd => {
            let mut parts = cmd_line.into_inner();
            let command_name = parts.next().unwrap().as_str();
            let args = parts.map(|arg| arg.as_str()).collect();
            Ok(CommandLine::Builtin(SimpleCommand { command_name, args }))
        }
        Rule::outer_cmd => {
            let mut parts = cmd_line.into_inner();
            let commands = parts.next().unwrap().into_inner();
            let mut piped_cmds = Vec::new();
            for command in commands {
                let mut command = command.into_inner();
                let command_name = command.next().unwrap().as_str();
                let args = command.map(|arg| arg.as_str()).collect();
                piped_cmds.push(SimpleCommand { command_name, args });
            }

            let redirect = parts.next().map(|redirect| {
                let append = redirect.as_rule() == Rule::append_redirect;
                let file = redirect.into_inner().next().unwrap().as_str();
                RedirectFile { file, append }
            });

            Ok(CommandLine::Extern(ExternCommandLine {
                piped_cmds,
                redirect,
            }))
        }
        _ => unreachable!(),
    }
}
