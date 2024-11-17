use super::*;
use env::set_current_dir;
use io::Read;
use std::fs;

/// 显示当前工作路径
fn pwd2() -> io::Result<()> {
    let cwd = env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

fn cd2<P: AsRef<Path>>(path: P) -> io::Result<()> {
    set_current_dir(path)
}
/// 显示路径path下（path为空时，则指当前工作目录）的所有文件和文件夹
fn ls2<P: AsRef<Path>>(path: P) -> io::Result<()> {
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
/// 若文件filename不存在，则新建文件filename；否则什么都不做
fn touch2<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::create(path)
}
/// 输出字符串str
fn echo2(str: &str) -> io::Result<()> {
    // whitespace is not allow
    println!("{str}");
    Ok(())
}
/// 显示文件filename内容，并在每一行前添加行号。
fn cat2<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    for (line_num, str) in content.split_inclusive("\n").enumerate() {
        print!("{line_num:02} {str}", line_num = line_num + 1);
    }
    Ok(())
}
/// 将filename1的内容拷贝到filename2中
fn cp2(from: &str, to: &str) -> io::Result<()> {
    fs::copy(from, to)?;
    Ok(())
}
/// 删除文件
fn rm(args: &[&str]) -> io::Result<()> {
    for file in args {
        fs::remove_file(file)?;
    }
    Ok(())
}
/// 删除文件夹
fn rm_recursive(dirs: &[&str]) -> io::Result<()> {
    for dir in dirs {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}
/// 将文件filename1改名为filename2
fn rename2(from: &str, to: &str) -> io::Result<()> {
    fs::rename(from, to)
}
/// 打印使用过的所有历史命令
fn history2() {
    for (i, cmd) in HISTORY.lock().unwrap().iter().enumerate() {
        print!("{} {}", i, cmd);
    }
}

pub fn exec_builtin(cmd_name: &str, args: &[&str]) -> io::Result<()> {
    use builtin_cmd::*;
    match cmd_name {
        "pwd2" => pwd2(),
        "cd2" => match args.first() {
            Some(path) => cd2(path),
            // 默认切换到home目录
            None => cd2(home::home_dir().unwrap()),
        },

        "ls2" => {
            let path = args.first().unwrap_or(&".");
            ls2(path)
        }
        "touch2" => {
            match args[..] {
                [] => {
                    eprintln!("missing file operand");
                }
                [files] => {
                    touch2(files)?;
                }
                _ => {
                    eprintln!("too many arguments");
                }
            }
            Ok(())
        }
        "echo2" => match args[..] {
            [] => echo2(""),
            [s] => echo2(s),
            _ => {
                eprintln!("echo: too many arguments");
                Ok(())
            }
        },
        "cat2" => match args.first() {
            Some(path) => cat2(path),
            None => {
                eprintln!("missing file operand");
                Ok(())
            }
        },
        "cp2" => {
            match args[..] {
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
            match args {
                [] => {
                    eprintln!("missing file operand");
                }
                ["-r", ..] => {
                    rm_recursive(&args[1..])?;
                }
                files => {
                    rm(files)?;
                }
            }
            Ok(())
        }
        "rename2" => {
            match args {
                [] => {
                    eprintln!("missing file operand");
                }
                [from] => {
                    eprintln!("missing destination file operand after '{}'", from);
                }
                [from, to] => rename2(from, to)?,
                _ => {
                    eprintln!("too many arguments");
                }
            }
            Ok(())
        }
        "history2" => {
            history2();
            Ok(())
        }
        "quit" => {
            println!("########Quiting hjpsh########");
            std::process::exit(0)
        }
        cmd => {
            eprintln!("{cmd}: command not found");
            Ok(())
        }
    }
}
