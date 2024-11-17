use anyhow::bail;
use anyhow::Ok;
use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "shell.pest"]
pub struct ShellParser;

struct SimpleCommand<'a> {
    command_name: &'a str,
    args: Vec<&'a str>,
}

struct Pipeline<'a> {
    commands: Vec<SimpleCommand<'a>>,
}

enum RedirectFile<'a> {
    Redirect(&'a str),
    AppendRedirect(&'a str),
}

struct OuterCommandLine<'a> {
    pipeline: Pipeline<'a>,
    redirect: Option<RedirectFile<'a>>,
}

struct BuiltinCommandLine<'a>(SimpleCommand<'a>);

pub enum CommandLine<'a> {
    Outer(OuterCommandLine<'a>),
    Builtin(BuiltinCommandLine<'a>),
}

pub fn parse_cmdline<'a>(cmd: &'a str) -> Result<CommandLine<'a>> {
    let cmd_line = ShellParser::parse(Rule::command_line, cmd)?
        .next()
        .unwrap() // command_line
        .into_inner()
        .next()
        .unwrap(); // buildin_cmd or outer_cmd

    match cmd_line.as_rule() {
        Rule::builtin_cmd => {
            todo!("内置命令暂未实现");
        }
        Rule::outer_cmd => {
            let mut parts = cmd_line.into_inner();
            let pipeline = parts.next().unwrap();
            if pipeline.as_rule() != Rule::pipeline {
                bail!(
                    "命令类型不匹配, 期望 pipeline, 实际 {:?}",
                    pipeline.as_rule()
                );
            };
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

            let redirect = parts.next().map(|redirect| match redirect.as_rule() {
                Rule::redirect => {
                    let file = redirect.into_inner().next().unwrap().as_str();
                    RedirectFile::Redirect(file)
                }
                Rule::append_redirect => {
                    let file = redirect.into_inner().next().unwrap().as_str();
                    RedirectFile::AppendRedirect(file)
                }
                _ => unreachable!(),
            });

            Ok(CommandLine::Outer(OuterCommandLine { pipeline, redirect }))
        }
        _ => unreachable!(),
    }
}
