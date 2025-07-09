#[allow(unused_macros)]
// 26
macro_rules! net_trace {
    ($($arg:expr),*) => { trace!($($arg),*) };
}

#[allow(unused_macros)]
// 30
macro_rules! net_debug {
    ($($arg:expr),*) => { debug!($($arg),*); }
}
