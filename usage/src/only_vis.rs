use mac::mac_internal;

macro_rules! mac_only_vis {
    ($visibility:vis) => {
        mac_internal!($visibility);
    };

    () => {
        mac_internal!();
        compile_error!("Empty branch chosen instead of branch with empty vis");
    };
}

mac_only_vis!(pub);
// Ok

#[cfg(none)]
mac_only_vis!();
// if cfg attribute is commented out, it produces:
//
// error: Empty branch chosen instead of branch with empty vis
//
// if empty branch is also commented out, then:
//
// error: unexpected end of macro invocation
// --> usage/src/only_vis.rs:17:1
// |
// 3  | macro_rules! mac_only_vis {
//    | ------------------------- when calling this macro
// ...
// 17 | mac_only_vis!();
//    | ^^^^^^^^^^^^^^^^ missing tokens in macro arguments
