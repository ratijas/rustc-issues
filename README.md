# Warning

The branches in this repository, as well as the repository as a whole, may be removed in future. Even permalinks are subjects to expiration.

### How to use [log] crate within dynamic libraries?

This repository/branch contains implementation of ideas discussed in the [issue log#421][issue].

This is my attempt at solving the problem of logging through `log!` macros from dynamically loaded libraries like cdylib runtime plugins.

The code was reorganized as a cargo workspace, but the main binary does not explicitly depend on dynamic libraries via cargo means, so
1. run `cargo build` to compile .so/.dll;
2. then `cargo run --bin main` to test executable.

### Solution

Below is a copy of my answer posted as a comment on aforementioned [issue]. And the rest of this repository/branch is a copy of what was attached there in zip archive at the bottom.

***

Since the question was about using `log` with dynamic libraries, not implementing your own logger, I removed custom logger implementation from your example and replaced it with a popular [`simple_logger`](https://crates.io/crates/simple_logger) backend.

Also, I packed two crates into a workspace, so it can be built and run with a single cargo command. Now, because they are members of the same workspace, we can replace complex relative paths with a simple `let path = "libdynlib.so";` Or, if you want to be extra paranoid, use `std::env::current_exe()`:

```rust
let exe = std::env::current_exe().unwrap();

#[cfg(any(target_os = "unix", target_os = "linux"))]
let path = "libdynlib.so";
#[cfg(target_os = "macos")]
let path = "libdynlib.dylib";

let lib_full_path = exe.parent().expect("executable must be in a parent directory").join(path);
```

[dynlog - stripped but still doesn't work.zip](https://github.com/rust-lang/log/files/6399910/dynlog.-.stripped.but.still.doesn.t.work.zip) (update: [repository link](https://github.com/ratijas/rustc-issues/tree/dynlog-stripped))

Alter all that trouble, we can verify, that logging is indeed does not work in cdylib:

```
❯ cargo run --bin main
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/main`
INFO  [main] Hello, world!
INFO  [main] dyn lib is loaded
INFO  [main] f() is loaded
f() is called...
INFO  [main] all is done...
```

This line in `dynlib/src/lib.rs` does nothing:

```rust
// this is not printed... unfortunately
log::info!("log is working... yes !!");
```

There's a good reason for that. They are being compiled absolutely separately, with very little in common. In fact, compiler pulls libraries and generates duplicate code for all types, functions and other stuff. Thus, `log` in lib has nothing to do with `log` in main.  If you don't initialize logging in lib with some backend, it won't automagically cross FFI and dlsym boundaries and won't print anything.

Take a look here:

https://github.com/rust-lang/log/blob/9d4206770dd93f07cb27c3e1f41dc21c45031302/src/lib.rs#L347-L367

If these variables were global/exported/no mangle, only then it would be possible to share runtime logging preferences among shared and dlopen-loaded libraries. But then it would've created various compatibility issues between different versions of `log` crate and between incompatible (undocumented and non-existent) rust ABIs.

Note that such problems do not exist e.g. in Java world, where everything runs on the same instance of JVM, and a mere idea of separately loading packages with all their dependencies into their own memory and address space is absurd. Unless you manage to spin off a second JVM instance (perhaps a different implementation, since VMs often do use global variables). In which case we would be back at square one: not only logging, but any objects would be impossible to pass back and forth without marshaling through JNI (Java Native Interface) glue.

So, unless you are willing to do some weird unsound ffi, it is totally up to a you as a developer to ensure consistent logging in Rust `cdylib`s.

The best you could do, it to add a normal Rust crate to your workspace, which contains all logging initialization. Add this crate as a dependency to each `cdylib` crate, and include an initialization routine. For example, in Qt QML extension plugin the initialization could be done at [type registration time](https://github.com/woboq/qmetaobject-rs/blob/ad28e6cc968707b2ff5c99871a1a1f193223d242/examples/qmlextensionplugins/src/lib.rs#L77-L87):

```rust
impl QQmlExtensionPlugin for QExampleQmlPlugin {
    fn register_types(&mut self, uri: &std::ffi::CStr) {
        my_shared_logging::init().unwrap();  // <- added initialization
        qml_register_type::<TimeModel>(uri, 1, 0, cstr!("Time"));
    }
}
```

Once final thought. We could go one layer deeper by factoring out common logging settings in an actual dcylib, while each plugin and exe would merely forward logging requests through well-defined FFI. Some [research](https://github.com/Michael-F-Bryan/rust-ffi-guide/blob/master/book/fun/problem_4/logging.rs) has been done in this direction already.

You gonna need:
- log forwarder with stable extern "C" API, and a Rust wrappers with `log::Log` implementation;
- micro crate (rlib) with initialization routine, which sets up (cdylib-local) log with a forwarder;
- actual shared cdylib which implements log forwarder extern functions, and initializes any fancy logger you want.

Now binary and plugins only need to dynamically (but not via dlopen) link to the shared logging cdylib. All the logic was extracted from them, and only forwarding stubs remain. Shared lib is, on the other hand, well, shared — along with its log preferences — once per process. The only trick is to design FFI-compatible log forwarder.

Let's try it out! With a project tree as follow:

```
.
├── Cargo.lock
├── Cargo.toml
├── dynlib
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── ffi-log
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── main
│   ├── Cargo.toml
│   └── src
│       └── main.rs
└── my-cdylib-log
    ├── Cargo.toml
    └── src
        └── lib.rs
```

Main app executes, sets up log forwarder for itself (which in turn sets up log receiver in "my-cdylib-log"), and then loads "dynlib" which also sets up log forwarder instance in its own memory space. Both forwarders are proxying messages through stable FFI thanks to "ffi-log" types and functions declarations.

```txt
❯ cargo build
❯ cd target/debug
❯ LD_LIBRARY_PATH=$(pwd) ./main
INFO  [my_cdylib_log] Initialized cdylib
INFO  [main] Hello, world!
INFO  [main] dyn lib is loaded
INFO  [main] f() is loaded
TRACE [dynlib] initialized logging in dynlib
f() is called...
INFO  [dynlib] log is working... yes !!
INFO  [main] all is done...
```

[dynlog - just works.zip](https://github.com/rust-lang/log/files/6400865/dynlog.-.just.works.zip) (update: [repository link](https://github.com/ratijas/rustc-issues/tree/dynlog-works))

As a bonus, valgrind didn't detect any memory leaks.

I've also done some FFI forwarding myself between Rust and Qt in [qmetaobject-rs](https://github.com/woboq/qmetaobject-rs/blob/master/qmetaobject/src/log.rs) crate. Feel free to check it out :)

PS Actually, it was discussed in #66.

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
