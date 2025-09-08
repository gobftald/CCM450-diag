#[cfg_attr(feature = "kwp2000", path = "kwp2000.rs")]
mod protocol_implementation;
pub use protocol_implementation::Protocol;
