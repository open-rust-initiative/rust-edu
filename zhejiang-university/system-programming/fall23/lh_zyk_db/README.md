# Rust DB

## Introduction

这个项目是由 lh & zyk 合作完成，可访问[github仓库](https://github.com/mirthfulLee/rust_db)查看最新版本。

数据管理系统是重要的系统软件，位于用户与操作系统之间的一层数据管理软件，它是一个大型复杂的软件系统，其主要功能是对数据库进行管理和维护，为用户提供各种数据管理服务。本项目使用Rust实现一个简单的数据库, 实现的功能包括：

* 基本的数据类型，例如int、string
* 至少支持 select、insert、update、delete 增删改查和 create、drop 数据表等基本操作
* 持久化存储引擎，能够将数据存储在磁盘上
* 执行引擎，能够读入SQL语句并执行

## Components

本数据库只是最最基本的实现, 因此仅简单地分为了三个部分:
* CLI/REPL: 交互式命令行用于访问系统
* SQL_analyzer: 实现SQL语句的解析
* executor: SQL语句的执行器
* storage: 实现数据的持久化

## References

* [Tutorial](https://johns.codes/blog/build-a-db/part01)
* [Parser](https://rustmagazine.github.io/rust_magazine_2021/chapter_6/parser-combinator.html)
* [Rules](https://github.com/dhcmrlchtdj/tree-sitter-sqlite/blob/main/grammar.js)
