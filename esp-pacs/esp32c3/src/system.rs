#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0xf8],
    cpu_intr_from_cpu_3: CPU_INTR_FROM_CPU_3,
}
impl RegisterBlock {
    /// 0x34 - interrupt generate register
    #[inline(always)]
    pub const fn cpu_intr_from_cpu_3(&self) -> &CPU_INTR_FROM_CPU_3 {
        &self.cpu_intr_from_cpu_3
    }
}

// CPU_INTR_FROM_CPU_3 (rw) register accessor: interrupt generate register
// 303
pub type CPU_INTR_FROM_CPU_3 = crate::Reg<cpu_intr_from_cpu_3::CPU_INTR_FROM_CPU_3_SPEC>;
pub mod cpu_intr_from_cpu_3;
