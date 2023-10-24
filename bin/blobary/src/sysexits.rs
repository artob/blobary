// This is free and unencumbered software released into the public domain.

use blobary::BlobStoreError;

#[allow(unused)]
pub type Result = std::result::Result<(), Sysexits>;

#[derive(Default)]
#[allow(non_camel_case_types, dead_code)]
pub enum Sysexits {
    #[default]
    EX_OK = 0,
    EX_USAGE = 64,
    EX_DATAERR = 65,
    EX_NOINPUT = 66,
    EX_NOUSER = 67,
    EX_NOHOST = 68,
    EX_UNAVAILABLE = 69,
    EX_SOFTWARE = 70,
    EX_OSERR = 71,
    EX_OSFILE = 72,
    EX_CANTCREAT = 73,
    EX_IOERR = 74,
    EX_TEMPFAIL = 75,
    EX_PROTOCOL = 76,
    EX_NOPERM = 77,
    EX_CONFIG = 78,
}

impl From<Box<dyn std::error::Error>> for Sysexits {
    fn from(_err: Box<dyn std::error::Error>) -> Self {
        Sysexits::EX_SOFTWARE
    }
}

impl From<std::io::Error> for Sysexits {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        match err.kind() {
            AddrInUse => Sysexits::EX_TEMPFAIL,
            AddrNotAvailable => Sysexits::EX_USAGE,
            AlreadyExists => Sysexits::EX_CANTCREAT,
            BrokenPipe => Sysexits::EX_IOERR,
            ConnectionAborted => Sysexits::EX_PROTOCOL,
            ConnectionRefused => Sysexits::EX_UNAVAILABLE,
            ConnectionReset => Sysexits::EX_PROTOCOL,
            Interrupted => Sysexits::EX_TEMPFAIL,
            InvalidData => Sysexits::EX_DATAERR,
            InvalidInput => Sysexits::EX_DATAERR,
            NotConnected => Sysexits::EX_PROTOCOL,
            NotFound => Sysexits::EX_NOINPUT,
            Other => Sysexits::EX_UNAVAILABLE,
            OutOfMemory => Sysexits::EX_TEMPFAIL,
            PermissionDenied => Sysexits::EX_NOPERM,
            TimedOut => Sysexits::EX_IOERR,
            UnexpectedEof => Sysexits::EX_IOERR,
            Unsupported => Sysexits::EX_SOFTWARE,
            WouldBlock => Sysexits::EX_IOERR,
            WriteZero => Sysexits::EX_IOERR,
            _ => Sysexits::EX_UNAVAILABLE, // catch-all
        }
    }
}

impl From<BlobStoreError> for Sysexits {
    fn from(err: BlobStoreError) -> Self {
        use BlobStoreError::*;
        match err {
            IO(err) => err.into(),
            Other(err) => err.into(),
        }
    }
}

pub fn exit(code: Sysexits) -> ! {
    std::process::exit(code as i32);
}

#[allow(unused_macros)]
macro_rules! abort {
    ($code:expr, $($t:tt)*) => {{
        eprintln!($($t)*);
        exit($code)
    }};
}
