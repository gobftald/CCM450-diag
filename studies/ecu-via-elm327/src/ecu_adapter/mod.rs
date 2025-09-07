#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter;
pub use adapter::Adapter;
