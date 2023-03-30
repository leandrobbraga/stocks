#![allow(unused)]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("\x1b[91mERROR\x1b[0m: {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("\x1b[92mINFO\x1b[0m: {}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("\x1b[93mWARNING\x1b[0m: {}", format!($($arg)*))
    };
}
