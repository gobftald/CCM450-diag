use core::str::FromStr;

use log::LevelFilter;

use super::println;

const LOG_TARGETS: Option<&'static str> = option_env!("ESP_LOGTARGETS");

pub fn init_logger_from_env() {
    unsafe {
        log::set_logger_racy(&EspLogger).unwrap();
    }

    const LEVEL: Option<&'static str> = option_env!("ESP_LOGLEVEL");

    if let Some(lvl) = LEVEL {
        let level = LevelFilter::from_str(lvl).unwrap_or_else(|_| LevelFilter::Off);
        unsafe { log::set_max_level_racy(level) };
    }
}

struct EspLogger;

impl log::Log for EspLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // check enabled log targets if any
        if let Some(targets) = LOG_TARGETS {
            if targets
                .split(",")
                .find(|v| record.target().starts_with(v))
                .is_none()
            {
                return;
            }
        }

        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
