use std::sync::Once;

use log::{info};

use ffi_log::*;

#[no_mangle]
pub extern "C" fn rust_log_enabled(metadata: ExternCMetadata) -> bool {
    let metadata = unsafe { metadata.as_metadata() };
    log::logger().enabled(&metadata)
}

#[no_mangle]
pub extern "C" fn rust_log_log(record: &ExternCRecord) {
    let mut builder = unsafe { record.as_record() };
    match format_args!("{}", unsafe { record.message.to_str() }) {
        args => {
            let record = builder.args(args).build();
            log::logger().log(&record);
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_log_flush() {
    log::logger().flush();
}

static START: Once = Once::new();

// Idempotent initialization, can be called any number of times.
#[no_mangle]
pub extern "C" fn my_cdylib_log_init() {
    START.call_once(|| {
        // set up shared logger
        simple_logger::SimpleLogger::new().init().unwrap();
        log::set_max_level(log::LevelFilter::Trace);
        info!("Initialized cdylib");
    });
}
