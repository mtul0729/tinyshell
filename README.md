# Linux技术课程大作业-hjpsh

## 外部库说明

- `home` 获取home目录
- `log`,`env_logger` 日志库，对不合法信息进行记录与输出
- `pest`,`pest_derive` PEG解析工具
- `anyhow` 错误处理
- `lazy_static` 惰性全局变量，用于保存历史命令

## TODO

- [x] 实现内置命令与基本的错误信息输出
- [x] 外部程序命令
- [x] 重定向文件流命令
- [x] 管道命令
- [ ] 为内置命令适配重定向与管道命令
- [x] 不合法信息处理（将使用外部库 `log`实现）
