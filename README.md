A Rust library that tries to make kernel exploits simpler. Potentially handy for CTF competitions.

想尝试一下用 Rust 来做 Linux kernel pwn 题目的感觉吗？快来试试吧。

## Examples

See [examples/](./examples/).

```sh
cargo build --release --example qwb2018-core
```

## Start your own exploit

```sh
git clone https://github.com/Kazurin-775/libkpwn-rs.git kpwn
cp -R kpwn/template exp

cd exp
vim src/main.rs
cargo build --release
```
