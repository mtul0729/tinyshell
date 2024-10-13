use std::fs::OpenOptions;
use std::io;
use std::process::{Child, Command, Stdio};
#[derive(Debug, PartialEq)]
pub enum ShellAST {
    Command {
        command: String,
        args: Vec<String>,
    },
    OutputRedirect {
        command: Box<ShellAST>,
        file: String,
        append: bool, // true for >>, false for >
    },
    Pipe {
        left: Box<ShellAST>,
        right: Box<ShellAST>,
    },
}

impl ShellAST {
    pub fn eval(&self) -> io::Result<std::process::ExitStatus> {
        self.eval1(Stdio::inherit(), Stdio::inherit())?.wait()
        
    }
    fn eval1(&self, stdin: Stdio, stdout: Stdio) -> io::Result<Child> {
        match self {
            ShellAST::Command { command, args } => {
                // 执行简单命令
                Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn() // 运行命令
            }
            ShellAST::OutputRedirect {
                command,
                file,
                append,
            } => {
                // 执行重定向
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(*append)
                    .open(file)?;
                command.eval1(stdin, Stdio::from(file))
            }
            ShellAST::Pipe { left, right } => {
                // 执行管道
                let left_out = left.eval1(stdin, Stdio::piped())?
                .stdout.expect("Failed to open stdout");
                right.eval1(Stdio::from(left_out), stdout)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let ast = ShellAST::Command {
            command: "echo".to_string(),
            args: vec!["Hello, world!".to_string()],
        };
        let result = ast.eval();
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_redirect() {
        let ast = ShellAST::OutputRedirect {
            command: Box::new(ShellAST::Command {
                command: "echo".to_string(),
                args: vec!["Hello, file!".to_string()],
            }),
            file: "test_output.txt".to_string(),
            append: false,
        };
        let result = ast.eval();
        assert!(result.is_ok());
        let content = std::fs::read_to_string("test_output.txt").unwrap();
        assert_eq!(content.trim(), "Hello, file!");
    }

    #[test]
    fn test_output_append() {
        let ast = ShellAST::OutputRedirect {
            command: Box::new(ShellAST::Command {
                command: "echo".to_string(),
                args: vec!["Hello, append!".to_string()],
            }),
            file: "test_output_append.txt".to_string(),
            append: true,
        };
        let _ = std::fs::write("test_output_append.txt", "Existing content\n");
        let result = ast.eval();
        assert!(result.is_ok());
        let content = std::fs::read_to_string("test_output_append.txt").unwrap();
        assert!(content.contains("Existing content"));
        assert!(content.contains("Hello, append!"));
    }

    #[test]
    fn test_pipe() {
        let ast = ShellAST::Pipe {
            left: Box::new(ShellAST::Command {
                command: "echo".to_string(),
                args: vec!["Hello, pipe!".to_string()],
            }),
            right: Box::new(ShellAST::Command {
                command: "grep".to_string(),
                args: vec!["pipe".to_string()],
            }),
        };
        let result = ast.eval();
        assert!(result.is_ok());
    }
}
