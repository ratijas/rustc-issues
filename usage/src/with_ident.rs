use mac::mac_internal;

macro_rules! mac_with_ident {
    ($visibility:vis $item:ident) => {
        mac_internal!($visibility);
    };

    // ($item:ident) => {
    //     compile_error!("Ident-only branch chosen instead of branch with empty vis");
    //     mac_internal!();
    // };

}

mac_with_ident!(pub foo);
// mac_with_ident!(bar);
