# MemFS

基于 FUSE 实现的简易内存文件系统

repo: [https://github.com/ztelliot/memfs](https://github.com/ztelliot/memfs)

## 实现的 FUSE 接口

- lookup
- getattr
- setattr
- readlink
- mkdir
- unlink
- rmdir
- symlink
- rename
- link
- read
- write
- readdir
- statfs
- create


## 数据结构

```rust
type Ino = u64;

pub struct Node {
    children: BTreeMap<String, Ino>,
    parent: Ino,
}

pub struct MemFile {
    hardlink: u32,
    data: Vec<u8>,
}

pub struct MemFS {
    files: BTreeMap<Ino, MemFile>,
    links: BTreeMap<Ino, Ino>,
    attrs: BTreeMap<Ino, fuser::FileAttr>,
    tree: BTreeMap<Ino, Node>,
    next: Ino,
}
```
