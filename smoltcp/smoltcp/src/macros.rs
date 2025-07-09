// 26
macro_rules! net_trace {
    ($($arg:expr),*) => { trace!($($arg),*) };
}

// 30
macro_rules! net_debug {
    ($($arg:expr),*) => { debug!($($arg),*); }
}
