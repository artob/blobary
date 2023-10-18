// This is free and unencumbered software released into the public domain.

#[allow(unused)]
type Result = std::result::Result<(), Sysexits>;

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
