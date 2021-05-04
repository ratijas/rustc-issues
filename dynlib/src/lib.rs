#[no_mangle]
pub extern "C" fn f()
{
    eprintln!("f() is called...");

    // this is not printed... unfortunately
    log::info!("log is working... yes !!");
}
