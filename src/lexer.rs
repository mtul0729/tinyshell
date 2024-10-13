#[derive(Debug, PartialEq)]
pub enum Token {
    Command(String),
    Arg(String),
    RedirectOutput, // '>' 重定向
    AppendOutput,   // '>>' 追加重定向
    Pipe,           // '|' 管道
    File(String),
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut parts = input.split_whitespace().peekable();

    while let Some(part) = parts.next() {
        match part {
            ">" => tokens.push(Token::RedirectOutput),
            ">>" => tokens.push(Token::AppendOutput),
            "|" => tokens.push(Token::Pipe),
            _ => {
                // 如果前一个 token 是 '>' 或 '>>'，则当前部分是文件
                if let Some(prev) = tokens.last() {
                    match prev {
                        Token::RedirectOutput | Token::AppendOutput => {
                            tokens.push(Token::File(part.to_string()));
                        }
                        _ => {
                            // 否则它是命令或参数
                            if tokens.is_empty() || matches!(tokens.last(), Some(Token::Pipe)) {
                                tokens.push(Token::Command(part.to_string()));
                            } else {
                                tokens.push(Token::Arg(part.to_string()));
                            }
                        }
                    }
                } else {
                    tokens.push(Token::Command(part.to_string()));
                }
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_command() {
        let input = "ls";
        let expected = vec![Token::Command("ls".to_string())];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_command_with_arg() {
        let input = "echo hello";
        let expected = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_redirect_output() {
        let input = "echo hello > output.txt";
        let expected = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::RedirectOutput,
            Token::File("output.txt".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_append_output() {
        let input = "echo hello >> output.txt";
        let expected = vec![
            Token::Command("echo".to_string()),
            Token::Arg("hello".to_string()),
            Token::AppendOutput,
            Token::File("output.txt".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_pipe() {
        let input = "ls | grep txt";
        let expected = vec![
            Token::Command("ls".to_string()),
            Token::Pipe,
            Token::Command("grep".to_string()),
            Token::Arg("txt".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_complex_command() {
        let input = "cat file.txt | grep 'search' > result.txt";
        let expected = vec![
            Token::Command("cat".to_string()),
            Token::Arg("file.txt".to_string()),
            Token::Pipe,
            Token::Command("grep".to_string()),
            Token::Arg("'search'".to_string()),
            Token::RedirectOutput,
            Token::File("result.txt".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_multiple_args() {
        let input = "cp file1.txt file2.txt";
        let expected = vec![
            Token::Command("cp".to_string()),
            Token::Arg("file1.txt".to_string()),
            Token::Arg("file2.txt".to_string()),
        ];
        assert_eq!(lex(input), expected);
    }
}
