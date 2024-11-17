use super::*;
use env::set_current_dir;
use io::Read;
use std::fs;

/// 显示当前工作路径
pub fn pwd2() -> io::Result<()> {
    let cwd = env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

pub fn cd2<P: AsRef<Path>>(path: P) -> io::Result<()> {
    set_current_dir(path)
}
/// 显示路径path下（path为空时，则指当前工作目录）的所有文件和文件夹
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
/// 若文件filename不存在，则新建文件filename；否则什么都不做
pub fn touch2<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::create(path)
}
/// 输出字符串str
pub fn echo2(str: &str) -> io::Result<()> {
    // whitespace is not allow
    println!("{str}");
    Ok(())
}
/// 显示文件filename内容，并在每一行前添加行号。
pub fn cat2<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    for (line_num, str) in content.split_inclusive("\n").enumerate() {
        print!("{line_num:02} {str}", line_num = line_num + 1);
    }
    Ok(())
}
/// 将filename1的内容拷贝到filename2中
pub fn cp2(from: &str, to: &str) -> io::Result<()> {
    fs::copy(from, to)?;
    Ok(())
}
/// 删除文件
pub fn rm(args: &[&str]) -> io::Result<()> {
    for file in args {
        fs::remove_file(file)?;
    }
    Ok(())
}
/// 删除文件夹
pub fn rm_recursive(dirs: &[&str]) -> io::Result<()> {
    for dir in dirs {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}
/// 将文件filename1改名为filename2
pub fn rename2(args: &[&str]) -> io::Result<()> {
    if args.len() == 3 {
        let old_path = &args[1];
        let new_path = &args[2];
        fs::rename(old_path, new_path)?;
    }
    Ok(())
}
/// 打印使用过的所有历史命令
pub fn history2(history: &Vec<String>) {
    for (i, cmd) in history.iter().enumerate() {
        println!("{} {}", i, cmd);
    }
}
