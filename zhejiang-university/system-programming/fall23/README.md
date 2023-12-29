<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 47f140b (Update README.md)
# 课程项目

## 要求

1. 1-3人一组，合作完成课程项目
2. 使用 Git 来进行代码管理，同时作为组内贡献的凭据
3. 课程结尾答辩，15~18分钟（视组数而定）
4. 提交代码 + 文档（附在代码中的README与汇报时的PPT）+ GitHub 项目链接

课程项目选题不限，各位同学可以针对自己的研究方向并结合系统软件进行开发。选题应与数据库、操作系统或编译器等系统软件相关，并且**必须**以Rust作为主要的编程语言。
考虑到部分同学缺乏相应的系统软件开发经历，助教提供以下选题，供大家参考。

### 参考选题一：利用 Rust 实现一个简单的数据库

数据管理系统是重要的系统软件，位于用户与操作系统之间的一层数据管理软件，它是一个大型复杂的软件系统，其主要功能是对数据库进行管理和维护，为用户提供各种数据管理服务。
本选题要求同学们利用Rust实现一个简单的数据库，要求实现的功能包括：

- 基本的数据类型，例如`int`、`string`
- 至少支持 `select`、`insert`、`update`、`delete` 增删改查和 `create`、`drop` 数据表等基本操作
- 持久化存储引擎，能够将数据存储在磁盘上
- 执行引擎，能够读入SQL语句并执行

感兴趣的同学还可以考虑实现基于B+树的存储引擎和查询优化、事务管理等功能。

参考资料：

- [CMU 15445](https://15445.courses.cs.cmu.edu/fall2023/)
- [Tutorial](https://johns.codes/blog/build-a-db/part01)
- [B+树可视化](https://www.cs.usfca.edu/galles/visualization/BPlusTree.html)

### 参考选题二：利用 Rust 实现一个小型的文件系统

从系统角度来看，文件系统是对文件存储设备的空间进行组织和分配，负责文件存储并对存入的文件进行保护和检索的系统，是操作系统的重要组成部分。然而现代操作系统的文件系统（例如NTFS）过于庞大，因此对于本次课程设计，我们可以利用Rust实现一个小型但功能完备的文件系统。
如果同学对操作系统或者分布式系统感兴趣，这个选题将会是一个具有挑战性但又极其有趣的题目。从宏观上来讲，同样也可采用KV键值对的方式来存储文件信息。
同学们可以在进行这个选题的实验过程中考虑以下几个问题：

1. 文件系统是否实现了最基本的存储和读取文件的功能？
2. 文件系统的数据结构是如何设计的？
3. 文件系统的索引是如何实现的？
4. 文件系统的空闲空间是如何管理和维护的？
5. 文件系统是否支持复制、移动、剪切等基本操作？
6. 是否可以给文件系统开发一个GUI界面，让它变得更酷一点？

参考资料：

- [Tutorial](https://blog.carlosgaldino.com/writing-a-file-system-from-scratch-in-rust.html)

### 参考选题三：利用 Rust 实现一个有实际意义的区块链智能合约应用

智能合约（Smart Contract）是存储在区块链上，可实现一定功能（往往是验证与交易功能）的程序。
智能合约以区块链上的程序和数据来替代传统的纸质文件条款，具有良好的公开透明、不可篡改、安全保密性质，并由计算机强制执行，将具有更高的信任成本和更低的运营成本。
当前常用的智能合约开发语言有Solidity、Vyper、WASM等。随着ink!等开发框架的出现，开发者也可以使用Rust语言编写智能合约。
本选题即为使用Rust语言开发一个智能合约应用。要求具有一定的实际意义。

参考资料：

- [Ink](https://paritytech.github.io/ink/)
- [Magazine](https://rustmagazine.github.io/rust_magazine_2021/chapter_3/ink.html)

### 参考选题四：利用 Rust 实现一个简易的 Git

相信大家都经常使用git来进行版本控制和管理, 但是相信大多数同学对Git的原理还是有些陌生的, 本选题将实现一个简易版的 Git.

1. 支持 git init, git add, git rm, git commit
2. 支持分支 git branch, git checkout
3. 支持简单的合并 git merge

有兴趣的同学可以考虑继续实现远程操作(remote), 如 git fetch, git pull 等

参考资料：

- [Git 原理与使用](https://missing.csail.mit.edu/2020/version-control/)
- [Rust 实现 Git](https://github.com/Byron/gitoxide)
- [Java 实现的简易版 Git](https://sp21.datastructur.es/materials/proj/proj2/proj2)

### 参考选题五：利用Rust实现一个简单语言的编译器

编译器同样是重要的系统软件，它能够将我们编写的高级源代码转换成贴近底层的目标代码，本选课即为利用 Rust 实现一个简单语言的编译器。

1. 你可以自定义实现的语言，但它至少应该具有基本的数据类型、变量声明、函数声明、循环、条件判断等基本语法。
2. 实现的编译器至少应该具有语法分析和代码生成这两部分。
3. 在代码生成中，你可以选择基于 [LLVM](https://llvm.org/) 的技术将源代码编译成目标文件，或者类似于 Java, Python 等语言的字节码并实现一个虚拟机来执行它。

参考资料：

- [Compiler Book](https://craftinginterpreters.com/)
- [Lox implementation in Rust](https://github.com/Darksecond/lox)
- [Stanford Compiler Course](https://web.stanford.edu/class/cs143/)

### 参考选题六：利用 Rust 实现一个简单的 Debug 工具

相信大家都使用过 GDB 来调试自己的程序，那么你是否想过 GDB 是如何实现的呢？本选题即为利用 Rust 实现一个简单的 Debug 工具，要求具有基本的单步执行、断电、函数调用栈查看等功能。

- [DEET](https://web.stanford.edu/class/cs110l/assignments/project-1/)

### 参考选题七：利用 Rust 实现一个 Raft 共识协议

Raft是分布式系统领域一个重要的共识算法（consensus algorithm），其目标是使得一个集群的服务器组成复制状态机从而实现一致性。
本选题要求实现在Rust上实现Raft协议，最好能够实现一个简单的基于Raft协议的KV存储系统。

参考资料：

- [Raft Website](https://raft.github.io/)
- [Raft Paper](https://raft.github.io/raft.pdf)
- [知乎资料](https://zhuanlan.zhihu.com/p/91288179)
- [C++ 实现](https://github.com/logcabin/logcabin)
- [Go 实现](https://github.com/hashicorp/raft)
- [6.824](http://nil.csail.mit.edu/6.824/2022/schedule.html)
<<<<<<< HEAD
=======

>>>>>>> f6df9d7 (Create README.md)
=======
>>>>>>> 47f140b (Update README.md)
