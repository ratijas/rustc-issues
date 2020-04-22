# Warning

The branches in this repository, as well as the repository as a whole, may be removed in future. Even permalinks are subjects to expiration.

### Issues with macros and `:vis` meta-variable

I did a lot of experiments, and I think I found three issues at once:

1. Empty :vis meta-variable

    According to the [documentation][doc-vis], `:vis` entity in `macro_rules!` should match empty visibility modifier:
        `vis:` a possibly empty Visibility qualifier

    While this is the case when visibility modifier is followed by some more stuff, it fails on its own:

    ```rust
    macro_rules! q1 {
        ($visibility:vis) => {}
    }

    q1!(); // error: unexpected end of macro invocation
    ```

    Followed by stuff:
    ```
    macro_rules! q2 {
        ($visibility:vis $item:ident) => {}
    }

    q2!(foobar); // ok
    ```

    Even though, adding ident-only branch still matches the first one (with `:vis`):
    ```
    macro_rules! q3 {
        ($visibility:vis $item:ident) => {};
        ($item:ident) => {
            compile_error!("Ident-only branch chosen instead of branch with empty vis");
        };
    }

    q3!(foobar); // ok
    ```

    Live rust code of these examples can be found in `./mac/src/q.rs`.

2. Forwarding empty :vis meta-variable

    When rustc forwards empty `:vis` meta-variable, it creates an empty `proc_macro::TokenTree::Group` which has no delimiter (`proc_macro::Delimiter::None` aka Ø) nor inner stream content (empty `proc_macro::TokenStream`). While [`None`][doc-Delimiter] delimiters is a documented feature (quoted) "important to preserve operator priorities", they "may not survive roundtrip of a token stream through a string".

    That leads to a workaround, but at the cost of losing context and spans of tokens:
    ```rust
    let stream: proc_macro2::TokenStream = /* ... */;
    let stream =
        stream.to_string()
            .parse::<proc_macro2::TokenStream>()
            .unwrap();
    ```

    Live rust code using this workaround can be found in `./usage/src/main.rs` (at the bottom) and `./mac/src/lib.rs` (turn the `USE_ROUNDTRIP` switch on).

    Also, `quote!` macro does not produce empty groups like that, so it is higly inconsistent behavior.

    Unlike `None` delimiter, completely empty group is not documented as a useful feature.

3. Parsing empty group with syn crate

    Syn crate is somewhat inconsistent when it comes to parsing empty TokenTree Group.

    Steps to reproduce:

    1. Enable `mac_with_ident!(bar);` line in `./usage/src/with_ident.rs` by commenting out/removing `#[cfg!(none)]` attribute.
    2. Turn off the `USE_ROUNDTRIP` switch in proc macro.
    3. Try to build.

    As can be seen from the error message `TAG_B: Error("unexpected token")`, `syn::parse2` has no problems parsing `syn::Visibility` out of the ParseStream with single empty group, but it fails to "complete" parsing because `ParseBuffer` is not [considered empty][ParseBuffer::is_empty] (it sees some "tokens" left in the buffer).

### Proposal:

1. Rustc macro matching

    Fix rustc macro matching subsystem to accept calls without arguments (e.g. `q!()`) as matching arms with single `:vis` meta-variable (e.g. `macro_rules! q { ($v:vis) => {} }`)

2. Rust macro expansion

    Fix rustc macro expansion subsystem to stop forwarding empty `vis` tokens as an empty groups.

3. Syn parser robustness

    Regardless of fixed in rustc, syn/proc_macro2 crates should handle those empty groups gracefully AND uniformly.
    Probably, skip those blank groups altogether. It's no good that they handle a "blank group" and a "blank group which is followed by stuff" cases differently.

### Meta

`rustc --version --verbose`:

```
rustc 1.42.0 (b8cedc004 2020-03-09)
binary: rustc
commit-hash: b8cedc00407a4c56a3bda1ed605c6fc166655447
commit-date: 2020-03-09
host: x86_64-unknown-linux-gnu
release: 1.42.0
LLVM version: 9.0

OS: Arch Linux
Kernel: x86_64 Linux 5.6.4-arch1-1
```

[doc-vis]: https://doc.rust-lang.org/reference/macros-by-example.html#metavariables
[ParseBuffer::is_empty]: https://docs.rs/syn/1.0.17/syn/parse/struct.ParseBuffer.html#method.is_empty
[doc-Delimiter]: https://doc.rust-lang.org/nightly/proc_macro/enum.Delimiter.html#variant.None
