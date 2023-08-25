A Rust library that tries to make kernel exploits simpler. Potentially handy for CTF competitions.

想尝试一下用 Rust 来做 Linux kernel pwn 题目的感觉吗？快来试试吧。

## Examples

The [examples/](./examples/) directory contains solutions for several entry-level kernel CTF challenges, as listed below:

```sh
cargo build --release --example qwb2018-core
cargo build --release --example ciscn2017-babydriver-easy
cargo build --release --example ciscn2017-babydriver-hard
cargo build --release --example minilctf2022-kgadget
```

Please read through their source code to gain a basic understanding of what this crate could do for you.

## Start your own exploit

```sh
# Clone this repo
git clone https://github.com/Kazurin-775/libkpwn-rs.git kpwn
# Copy exploit template
cp -R kpwn/template exp

# Start writing your exploit!
cd exp
vim src/main.rs

# Build and test
cargo build --release
ln -s ./target/x86_64-unknown-linux-musl/release/exp ./exp
# Send the exploit to VM via network
nc -lNvp 5678 < ./exp
# Inside the VM, run: nc $HOST_IP 5678 > /exp
```
