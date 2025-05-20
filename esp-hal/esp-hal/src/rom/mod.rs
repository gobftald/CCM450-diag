// 32
pub(crate) mod regi2c;

// 45
#[allow(unused)]
#[inline(always)]
pub(crate) fn ets_update_cpu_frequency_rom(ticks_per_us: u32) {
    unsafe extern "C" {
        fn ets_update_cpu_frequency(ticks_per_us: u32);
    }

    unsafe { ets_update_cpu_frequency(ticks_per_us) };
}
