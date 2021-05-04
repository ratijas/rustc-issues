# Warning

The branches in this repository, as well as the repository as a whole, may be removed in future. Even permalinks are subjects to expiration.

### How to use [log] crate within dynamic libraries?

This repository/branch contains implementation of ideas discussed in the [issue log#421][issue].

This particular commit represents topic starter's code, original but slightly modified for readability and easier management. It still doesn't do what it's supposed to, but at least now it's something we can work with.

The code was reorganized as a cargo workspace, but the main binary does not explicitly depend on a dynamic library via cargo means, so
1. run `cargo build` to compile .so/.dll;
2. then `cargo run --bin main` to test executable.

### Meta

`rustc --version --verbose`:

```
rustc 1.51.0 (2fd73fabe 2021-03-23)
binary: rustc
commit-hash: 2fd73fabe469357a12c2c974c140f67e7cdd76d0
commit-date: 2021-03-23
host: x86_64-unknown-linux-gnu
release: 1.51.0
LLVM version: 11.0.1

OS: Arch Linux
Kernel: x86_64 Linux 5.11.16-arch1-1
```

[log]: https://github.com/rust-lang/log
[issue]: https://github.com/rust-lang/log/issues/421
