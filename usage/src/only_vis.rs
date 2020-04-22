use mac::mac_internal;

macro_rules! mac_only_vis {
    ($visibility:vis) => {
        mac_internal!($visibility);
    };

    () => {
        compile_error!("Empty branch chosen instead of branch with empty vis");
        mac_internal!();
    };
}

// mac_only_vis!(pub);
// mac_only_vis!();
