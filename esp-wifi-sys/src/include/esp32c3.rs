#[repr(C)]
#[derive(Copy, Clone)]
// 1528
pub struct ets_timer {
    pub next: *mut timer_adpt,
    pub expire: u32,
    pub period: u32,
    pub func: ::core::option::Option<unsafe extern "C" fn(priv_: *mut crate::c_types::c_void)>,
    pub priv_: *mut crate::c_types::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
// 9117
pub struct timer_adpt {
    pub _address: u8,
}
