use libloading::*;
use log::*;

fn main() {
    ffi_log::ExternCLog::init().unwrap();

    info!("Hello, world!");

    let exe = std::env::current_exe().unwrap();

    #[cfg(any(target_os = "unix", target_os = "linux"))]
        let path = "libdynlib.so";
    #[cfg(target_os = "macos")]
        let path = "libdynlib.dylib";

    let lib_full_path = exe.parent().expect("executable must be in a parent directory").join(path);

    let lib = Library::new(&lib_full_path).expect("can’t load dyn lib");
    info!("dyn lib is loaded");
    unsafe {
        let f: libloading::Symbol<unsafe extern fn()> = lib.get(b"f\0").expect("can’t find f()");
        info!("f() is loaded");
        f();
    }
    info!("all is done...");
}
