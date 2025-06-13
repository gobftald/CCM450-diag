#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0xb0],
    data: DATA,
}

impl RegisterBlock {
    /// 0xb0 - Random number data
    #[inline(always)]
    pub const fn data(&self) -> &DATA {
        &self.data
    }
}

/// DATA (r) register accessor: Random number data
pub type DATA = crate::Reg<data::DATA_SPEC>;
pub mod data;
