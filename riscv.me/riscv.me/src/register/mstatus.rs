//! mstatus register

#[cfg(target_arch = "riscv32")]
read_write_csr! {
    /// mstatus register
    Mstatus: 0x300,
    mask: 0x807f_fffe,
}

read_write_csr_field! {
    Mstatus,
    /// Machine Interrupt Enable
    mie: 3,
}

csr_field_enum! {
    /// Machine Previous Privilege Mode
    MPP {
        default: User,
        User = 0,
        Supervisor = 1,
        Machine = 3,
    }
}

read_write_csr_field! {
    Mstatus,
    /// Machine Previous Privilege Mode
    mpp,
    MPP: [11:12],
}

read_write_csr_field! {
    Mstatus,
    /// Machine Previous Interrupt Enable
    mpie: 7,
}

set!(0x300);
clear!(0x300);

set_clear_csr!(
    /// Machine Interrupt Enable
    , set_mie, clear_mie, 1 << 3);

/// Machine Previous Privilege Mode
#[inline]
pub unsafe fn set_mpp(mpp: MPP) {
    let mut value = _read();
    value &= !(0x3 << 11); // clear previous value
    value |= (mpp as usize) << 11;
    _write(value);
}

set_csr!(
    /// Machine Previous Interrupt Enable
    , set_mpie, 1 << 7);
