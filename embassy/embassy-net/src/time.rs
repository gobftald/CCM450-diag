// 3
use embassy_time::{Duration, Instant};
use smoltcp::time::{Duration as SmolDuration, Instant as SmolInstant};

// 6
pub(crate) fn instant_to_smoltcp(instant: Instant) -> SmolInstant {
    SmolInstant::from_micros(instant.as_micros() as i64)
}

// 14
pub(crate) fn duration_to_smoltcp(duration: Duration) -> SmolDuration {
    SmolDuration::from_micros(duration.as_micros())
}
