use std::alloc::System;
use std::mem::ManuallyDrop;

use log::{Level, LevelFilter, Log, Metadata, Record, RecordBuilder, SetLoggerError};

#[global_allocator]
static GLOBAL: System = System;

#[link(name = "my_cdylib_log")]
extern "C" {
    pub fn my_cdylib_log_init();
    pub fn rust_log_enabled(metadata: ExternCMetadata) -> bool;
    pub fn rust_log_log(record: &ExternCRecord);
    pub fn rust_log_flush();
}

pub struct ExternCLog;

/// FFI-safe borrowed Rust &str. Can represents `Option<&str>` by setting ptr to null.
#[repr(C)]
pub struct RustStr {
    pub ptr: *const u8,
    pub len: usize,
}

/// FFI-safe owned Rust String.
#[repr(C)]
pub struct RustString {
    pub ptr: *mut u8,
    pub cap: usize,
    pub len: usize,
}

#[repr(C)]
pub struct ExternCMetadata {
    pub level: Level,
    pub target: RustStr,
}

#[repr(C)]
pub struct ExternCRecord {
    pub metadata: ExternCMetadata,
    /// fmt::Arguments<'a> are not FFI-safe, so we have no option but to format them beforehand.
    pub message: RustString,
    pub module_path: RustStr, // None points to null
    pub file: RustStr, // None points to null
    pub line: i64, // None maps to -1, everything else should fit in u32.
}

static LOGGER: ExternCLog = ExternCLog;

impl ExternCLog {
    pub fn init() -> Result<(), SetLoggerError> {
        unsafe { my_cdylib_log_init() };
        // don't decide here, just forward everything.
        log::set_max_level(LevelFilter::Trace);
        log::set_logger(&LOGGER)
    }
}

impl Log for ExternCLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let metadata = ExternCMetadata::from(metadata);
        unsafe { rust_log_enabled(metadata) }
    }

    fn log(&self, record: &Record) {
        let record = ExternCRecord::from(record);
        // message string ownership is passed onto callee.
        unsafe { rust_log_log(&record); }
    }

    fn flush(&self) {
        unsafe { rust_log_flush(); }
    }
}

impl<'a> From<&'a str> for RustStr {
    fn from(s: &'a str) -> Self {
        let bytes = s.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}

impl<'a> From<Option<&'a str>> for RustStr {
    fn from(o: Option<&'a str>) -> Self {
        match o {
            None => Self { ptr: std::ptr::null(), len: 0 },
            Some(s) => Self::from(s),
        }
    }
}

impl RustStr {
    pub unsafe fn to_str<'a>(&self) -> &'a str {
        let bytes = std::slice::from_raw_parts(self.ptr, self.len);
        std::str::from_utf8_unchecked(bytes)
    }

    pub unsafe fn to_opt_str<'a>(&self) -> Option<&'a str> {
        if self.ptr.is_null() {
            None
        } else {
            Some(self.to_str())
        }
    }
}

impl From<String> for RustString {
    fn from(s: String) -> Self {
        let mut me = ManuallyDrop::new(s);
        let (ptr, len, cap) = (me.as_mut_ptr(), me.len(), me.capacity());
        Self { ptr, len, cap }
    }
}

impl RustString {
    pub unsafe fn to_str<'a>(&self) -> &'a str {
        RustStr {
            ptr: self.ptr as _,
            len: self.len,
        }.to_str()
    }
    pub unsafe fn into_string(self) -> String {
        String::from_raw_parts(self.ptr, self.len, self.cap)
    }
}

impl Drop for RustString {
    fn drop(&mut self) {
        unsafe {
            String::from_raw_parts(self.ptr, self.len, self.cap);
        }
    }
}

impl<'a> From<&Metadata<'a>> for ExternCMetadata {
    fn from(m: &Metadata<'a>) -> Self {
        Self {
            level: m.level(),
            target: m.target().into()
        }
    }
}

impl ExternCMetadata {
    pub unsafe fn as_metadata(&self) -> Metadata {
        let level = self.level;
        let target = self.target.to_str();
        Metadata::builder()
            .level(level)
            .target(target)
            .build()
    }
}

impl<'a> From<&Record<'a>> for ExternCRecord {
    fn from(r: &Record<'a>) -> Self {
        let msg = r.args().to_string();
        Self {
            metadata: ExternCMetadata::from(r.metadata()),
            message: RustString::from(msg),
            module_path: RustStr::from(r.module_path()),
            file: RustStr::from(r.file()),
            line: r.line().map(|u| u as i64).unwrap_or(-1_i64),
        }
    }
}

impl ExternCRecord {
    pub unsafe fn as_record(&self) -> RecordBuilder {
        let mut builder = Record::builder();
        builder
            // .args(format_args!("{}", self.message))
            .metadata(self.metadata.as_metadata())
            .module_path(self.module_path.to_opt_str())
            .file(self.file.to_opt_str())
            .line(if self.line == -1 { None } else { Some(self.line as _) });
        builder
    }
}

pub unsafe fn f() {
    let ecr = ExternCRecord {
        metadata: ExternCMetadata { level: Level::Error, target: RustStr::from("main") },
        message: RustString::from(format!("yahaha! {}", "you found me!")),
        module_path: RustStr::from("lib::hyrule"),
        file: RustStr::from("Rito.rs"),
        line: 42,
    };
    let mut builder = ecr.as_record();
    match format_args!("{}", ecr.message.to_str()) {
        args => {
            let _record = builder.args(args).build();
        }
    }
}

#[test]
fn test_drop() {
    unsafe { f(); }
}
