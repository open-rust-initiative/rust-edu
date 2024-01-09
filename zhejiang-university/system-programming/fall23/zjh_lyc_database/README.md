基于parser、B-Tree等技术使用Rust实现一个数据库，可执行创建表、插入数据等操作。具体命令可见提供的帮助信息。

仓库链接：https://github.com/Akihito233/Database-Rust

小组成员：zjh(Akihito233)，lyc(Beatlesso)

```
Database-Rust 项目目录

├─sample 用于测试的SQL样例

└─src 源码目录

    ├─meta_command 数据库系统元命令模块

    ├─repl 命令行交互模块

    └─sql 数据库、数据表等定义与SQL语句解析、执行的相关模块

        ├─db 数据库、数据包定义及创建插入等操作

        └─parser SQL语句解析模块
```

cargo run 即可运行