#[cfg_attr(feature = "kwp2000", path = "kwp2000.rs")]
mod protocol_impl;
pub use protocol_impl::*;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EcuError(pub u32);

pub trait Protocols {
    fn format_request(&self, service_id: u8, param: &[u8], buf: &mut [u8]) -> usize;
    fn parse_response<'a>(&self, service_id: u8, response: &'a [u8]) -> Result<usize, EcuError>;
}
