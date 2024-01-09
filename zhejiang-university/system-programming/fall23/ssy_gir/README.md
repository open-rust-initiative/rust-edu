
##支持功能
- 基础: `init`, `config`, `add`, `rm`,`commit`, `status`.
- 分支: `branch`, `checkout`, `merge`.

##使用实例

###构建
```bash
$ cargo build --release
$ cd target/release
$ ./gir
```

###使用
```bash
$ mkdir repo
$ cd repo
$ .././gir init
Initialized empty Git repository in .git
$ .././gir config --add user.name "ssy"
$ .././gir config --add user.email "ssy@com"

$ echo 'Hello world!' > file_a
$ .././gir status
new: file_a
$ .././gir add file_a
$ .././gir commit -m "first commit"
[master e91657f] first commit


$ .././gir branch new_b
$ .././gir checkout new_b
Switched to branch new_b
$ echo 'new line' >> file_a
$ .././gir status
modified: file_a
$ .././gir add file_a
$ .././gir commit -m "second commit"
[new_b 25462c6] second commit
$ .././gir checkout master
Switched to branch master
$ .././gir branch -l 
* master
  new_b

$ echo 'new file' >> file_b
$ .././gir add file_b
$ .././gir commit -m "third commit"
[master 4497bf0] third commit
$ .././gir merge new_b
Merge new_b into master
[master 2c4739a] Merge new_b into master
$ .././gir rm file_b
$ .././gir status 
deleted: file_b
```
