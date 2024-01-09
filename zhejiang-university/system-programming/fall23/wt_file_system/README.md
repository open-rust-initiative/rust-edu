# rust_file_system
使用Rust语言写的一个虚拟文件系统
<br>
github地址：https://github.com/qrqeea/rust_file_system

## 使用说明
使用命令行交互界面，支持以下命令：
* `cd <dirname>`: 更改当前目录
* `touch <filename>`: 创建文件
* `ls` : 查看当前目录下的所有文件
* `cat <filename>`: 查看文件内容
* `mkdir <dirname>`: 新建目录
* `cp <filename> <new_filename>` : 复制文件
* `rename <raw_name> <new_name>` : 重命名文件
* `rm <filename>`: 删除文件
* `mv <filename> <path>` : 移动文件
* `save` : 保存文件系统
* `diskinfo` : 查看磁盘使用情况
* `exit` : 退出程序

## 设计说明
本文件系统不涉及多用户、权限管理等功能，重点在于文件存储，记录各个文件分别使用了哪些磁盘块

### 文件系统布局
真实文件系统建立在一个磁盘分区上，分区分为多个部分，包括引导块、超级快、空闲空间管理、根目录、文件和目录等，因为只是实现一个模拟文件系统，并不需要这么多信息，只需要FAT和文件数据区两部分

### 关键点
* 使用一个真实文件模拟磁盘，在这个真实文件中存储虚拟文件，对磁盘读写 —> 对文件读写
* 文件属性和内容分开存储
* 单个文件不连续存储（文件大小大于一个block）
* 使用文件分配表（FAT）记录各个文件分别使用哪些磁盘块

### 数据结构设计
普通文件数据：包括文件属性和内容
<br>
目录文件数据：包括目录目录和目录项（目录包含哪些文件）
* FCB（文件控制块）
```rust
enum FileType {
    File,
    Directory,
}

struct Fcb {
    name: String,         // 文件名
    file_type: FileType,  // 文件or目录
    first_block: usize,   // 起始块号
    length: usize,        // 文件大小
}
```
* 虚拟磁盘
```rust
pub enum FatStatus {
    UnUsed,           // 未使用的块
    NextBlock(usize), // 下一块块号
    EOF,              // 结束标志
}

struct VirtualDisk {
    pub fat: Vec<FatStatus>,    // 文件分配表
    data: Vec<u8>,              // 存储文件数据
}
```
* 目录结构
```rust
pub struct Directory {
    name: String,       // 目录名
    files: Vec<Fcb>,    // 目录项是文件控制块
}
```
## 功能实现说明
`cd`
<br>
* 程序持有一个Directory类型的对象用于表示当前目录，切换目录时从磁盘读取新目录的数据更新Directory对象

`touch`
<br>
* 在磁盘数据区分配block用于存储文件数据，然后更新FAT并记录首块块号
* 创建一个Fcb对象保存文件属性，其中包括首块块号，然后在表示当前目录的Directory中加入该Fcb对象


`ls`
<br>
* Directory对象的files属性存储当前目录下的所有文件属性

`cat`
<br>
* 在Directory对象的files中查找该文件的FCB获得首块块号，然后通过FAT表可以获取存储该文件的所有块，从数据区读取数据即可

`mkdir`
<br>
* 创建一个Directory对象，files包括父目录和自身
* 在磁盘数据区分配block用于存储Directory对象，然后更新FAT并记录首块块号
* 创建一个Fcb对象记录目录属性，其中包括首块块号，然后在表示当前目录的Directory中加入该Fcb对象

`cp`
<br>
* 读取文件数据（同cat）拷贝一份数据
* 用拷贝的数据创建新文件（同touch）

`rename`
<br>
* 在Directory对象的files中查找该文件的FCB
* 修改FCB中的文件名

`rm`
<br>
* 删除文件数据：在Directory对象的files中查找该文件的FCB获得首块块号，然后修改FAT表释放分配的数据块，无需修改数据区
* 删除文件属性：在Directory对象的files中查找该文件的FCB然后删除

`mv`
<br>
* 在当前目录对应的Directory对象的files中查找该文件的FCB，从当前目录移除
* 在新目录对应的Directory对象的files中添加该文件对应的FCB
* 无需修改文件数据部分

`diskinfo`
<br>
* 用虚拟磁盘的FAT属性统计即可

## 局限
* 仅支持最基本的文件存储功能
* 只能通过虚拟文件系统访问虚拟磁盘中的文件，其他程序无法读写虚拟文件