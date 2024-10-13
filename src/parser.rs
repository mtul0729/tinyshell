use crate::lexer::Token;
use crate::eval::ShellAST;

pub fn parse(tokens: &[Token]) -> ShellAST {
    let mut pos = 0;
    parse_pipeline(tokens, &mut pos)
}

// 解析管道表达式
fn parse_pipeline(tokens: &[Token], pos: &mut usize) -> ShellAST {
    let mut left = parse_command(tokens, pos);

    while *pos < tokens.len() {
        if tokens[*pos] == Token::Pipe {
            *pos += 1;
            if *pos >= tokens.len() {
                // 检查右边是否有命令
                panic!("Syntax error: missing command after '|'");
            }
            let right = parse_command(tokens, pos);
            left = ShellAST::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            };
        } else {
            break;
        }
    }

    left
}

// 解析命令和重定向
fn parse_command(tokens: &[Token], pos: &mut usize) -> ShellAST {
    if let Token::Command(cmd) = &tokens[*pos] {
        let mut args = Vec::new();
        *pos += 1;

        while *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Arg(arg) => {
                    args.push(arg.clone());
                    *pos += 1;
                }
                Token::RedirectOutput | Token::AppendOutput => {
                    let append = matches!(&tokens[*pos], Token::AppendOutput);
                    *pos += 1;

                    if let Token::File(file) = &tokens[*pos] {
                        *pos += 1;
                        return ShellAST::OutputRedirect {
                            command: Box::new(ShellAST::Command {
                                command: cmd.clone(),
                                args,
                            }),
                            file: file.clone(),
                            append,
                        };
                    }
                }
                _ => break,
            }
        }

        ShellAST::Command {
            command: cmd.clone(),
            args,
        }
    } else {
        panic!("Expected a command, found {:?}", tokens[*pos]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let tokens = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
        ];
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ShellAST::Command {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_command_with_output_redirect() {
        let tokens = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::RedirectOutput,
            Token::File("output.txt".to_string()),
        ];
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ShellAST::OutputRedirect {
                command: Box::new(ShellAST::Command {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                file: "output.txt".to_string(),
                append: false,
            }
        );
    }

    #[test]
    fn test_parse_command_with_append_output_redirect() {
        let tokens = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::AppendOutput,
            Token::File("output.txt".to_string()),
        ];
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ShellAST::OutputRedirect {
                command: Box::new(ShellAST::Command {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                file: "output.txt".to_string(),
                append: true,
            }
        );
    }

    #[test]
    fn test_parse_pipeline() {
        let tokens = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::Pipe,
            Token::Command("grep".to_string()),
            Token::Arg("h".to_string()),
        ];
        let ast = parse(&tokens);
        assert_eq!(
            ast,
            ShellAST::Pipe {
                left: Box::new(ShellAST::Command {
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }),
                right: Box::new(ShellAST::Command {
                    command: "grep".to_string(),
                    args: vec!["h".to_string()],
                }),
            }
        );
    }

    #[test]
    #[should_panic(expected = "Syntax error: missing command after '|'")]
    fn test_parse_pipeline_missing_command() {
        let tokens = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::Pipe,
        ];
        parse(&tokens);
    }

    #[test]
    #[should_panic(expected = "Expected a command, found")]
    fn test_parse_invalid_token() {
        let tokens = vec![Token::Arg("hello".to_string())];
        parse(&tokens);
    }
}
