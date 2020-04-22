macro_rules! q1 {
    ($visibility:vis) => {}
}

#[cfg(none)]
q1!(); // error: unexpected end of macro invocation



macro_rules! q2 {
    ($visibility:vis $item:ident) => {}
}

q2!(foobar); // ok


macro_rules! q3 {
    ($visibility:vis $item:ident) => {};
    ($item:ident) => {
        compile_error!("Ident-only branch chosen instead of branch with empty vis");
    };
}

q3!(foobar); // ok
