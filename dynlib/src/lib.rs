use std::sync::Once;

static START: Once = Once::new();

// Idempotent initialization, can be called any number of times.
fn lib_init() {
    START.call_once(|| {
        ffi_log::ExternCLog::init().unwrap();
        log::trace!("initialized logging in dynlib");
    });
}

#[no_mangle]
pub extern "C" fn f()
{
    lib_init();

    eprintln!("f() is called...");

    log::info!("log is working... yes !!");
}
