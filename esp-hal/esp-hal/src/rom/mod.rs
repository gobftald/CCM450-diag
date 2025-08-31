// 32
pub(crate) mod regi2c;

#[inline(always)]
// 35
pub(crate) fn ets_delay_us(us: u32) {
    unsafe extern "C" {
        fn ets_delay_us(us: u32);
    }

    unsafe { ets_delay_us(us) };
}

#[allow(unused)]
#[inline(always)]
// 45
pub(crate) fn ets_update_cpu_frequency_rom(ticks_per_us: u32) {
    unsafe extern "C" {
        fn ets_update_cpu_frequency(ticks_per_us: u32);
    }

    unsafe { ets_update_cpu_frequency(ticks_per_us) };
}

#[inline(always)]
// 54
pub(crate) fn rtc_get_reset_reason(cpu_num: u32) -> u32 {
    unsafe extern "C" {
        unsafe fn rtc_get_reset_reason(cpu_num: u32) -> u32;
    }

    unsafe { rtc_get_reset_reason(cpu_num) }
}
