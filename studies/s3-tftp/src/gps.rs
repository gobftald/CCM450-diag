use embedded_sdmmc::{Timestamp, TimeSource};

static mut GPS_TIMESTAMP: Timestamp = Timestamp {
    year_since_1970: 56, zero_indexed_month: 0, zero_indexed_day: 1, hours: 0, minutes: 0, seconds: 0 
};

pub struct GpsTimeSource;

impl TimeSource for GpsTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        // don't care if not precise timestamp caused by potential race condition
        unsafe { GPS_TIMESTAMP }
    }
}