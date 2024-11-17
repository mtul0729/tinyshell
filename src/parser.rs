use anyhow::Ok;
use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "shell.pest"]
struct ShellParser;

pub struct SimpleCommand<'a> {
    pub command_name: &'a str,
    pub args: Vec<&'a str>,
}

struct Pipeline<'a> {
    commands: Vec<SimpleCommand<'a>>,
}
use std::process::{Child, Command, Stdio};
fn run_outer_cmd(
    cmds: &[SimpleCommand],
    input: Stdio,
    tail_stdout: Stdio,
) -> std::io::Result<Child> {
    match cmds {
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
            run_outer_cmd(rest, output.into(), tail_stdout)
        }
        _ => unreachable!(),
    }
}

struct RedirectFile<'a> {
    file: &'a str,
    append: bool,
}

pub struct OuterCommandLine<'a> {
    pipeline: Pipeline<'a>,
    redirect: Option<RedirectFile<'a>>,
}

impl OuterCommandLine<'_> {
    pub fn run(self) -> std::io::Result<()> {
        let input = Stdio::inherit();
        let tail_stdout = match self.redirect {
            Some(redirect) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(redirect.append)
                    .open(redirect.file)?;
                Stdio::from(file)
            }
            None => Stdio::inherit(),
        };
        let pipeline = self.pipeline.commands.as_slice();
        let mut child = run_outer_cmd(pipeline, input, tail_stdout)?;
        child.wait()?;
        std::io::Result::Ok(())
    }
}

pub enum CommandLine<'a> {
    Outer(OuterCommandLine<'a>),
    Builtin(SimpleCommand<'a>),
}

use crate::builtin_cmd::exec_builtin;
impl CommandLine<'_> {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            CommandLine::Outer(outer_cmd) => outer_cmd.run(),
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
        .unwrap(); // buildin_cmd or outer_cmd

    match cmd_line.as_rule() {
        Rule::builtin_cmd => {
            let mut parts = cmd_line.into_inner();
            let command_name = parts.next().unwrap().as_str();
            let args = parts.map(|arg| arg.as_str()).collect();
            Ok(CommandLine::Builtin(SimpleCommand { command_name, args }))
        }
        Rule::outer_cmd => {
            let mut parts = cmd_line.into_inner();
            let pipeline = parts.next().unwrap();
            // if pipeline.as_rule() != Rule::pipeline {
            //     bail!(
            //         "命令类型不匹配, 期望 pipeline, 实际 {:?}",
            //         pipeline.as_rule()
            //     );
            // };
            let commands = pipeline.into_inner();
            let mut pipeline = Pipeline {
                commands: Vec::new(),
            };
            for command in commands {
                let mut command = command.into_inner();
                let command_name = command.next().unwrap().as_str();
                let args = command.map(|arg| arg.as_str()).collect();
                pipeline.commands.push(SimpleCommand { command_name, args });
            }

            let redirect = parts.next().map(|redirect| {
                let append = redirect.as_rule() == Rule::append_redirect;
                let file = redirect.into_inner().next().unwrap().as_str();
                RedirectFile { file, append }
            });

            Ok(CommandLine::Outer(OuterCommandLine { pipeline, redirect }))
        }
        _ => unreachable!(),
    }
}
