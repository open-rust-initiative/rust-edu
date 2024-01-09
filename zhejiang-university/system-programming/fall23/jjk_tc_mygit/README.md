### mygit， a Git implementation in Rust

#### Usage

Set the git config in `main.rs`.

Build the `mygit` binary and add it to your PATH:

```sh
$ cargo build
```

Add it to your PATH.

Finally, initialize a Git repo and create commit:

```sh
$ mygit init
$ mygit add .
$ mygit rm
$ mygit commit
# Currently, this waits for your input. Type in your commit message
# and hit Ctrl+D
$ mygit branch
$ mygit checkout
```

#### Acknowledgement

James Coglan's book <Building Git>

https://github.com/samrat/rug