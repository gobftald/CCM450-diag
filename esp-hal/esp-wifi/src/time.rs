/// Current systimer count value
/// A tick is 1 / 1_000_000 seconds
/// This function must not be called in a critical section. Doing so may return
/// an incorrect value.
// 8
pub(crate) fn systimer_count() -> u64 {
    esp_hal::time::Instant::now()
        .duration_since_epoch()
        .as_micros()
}

// TODO: use an Instance type instead...
#[cfg(target_arch = "riscv32")]
// 16
pub(crate) fn time_diff(start: u64, end: u64) -> u64 {
    // 52-bit wrapping sub
    end.wrapping_sub(start) & 0x000f_ffff_ffff_ffff
}

/// Do not call this in a critical section!
// 48
pub(crate) fn elapsed_time_since(start: u64) -> u64 {
    let now = systimer_count();
    time_diff(start, now)
}
