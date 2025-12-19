// 2
use esp_config::{ConfigOption, generate_config};

// 4
fn main() {
    // Ensure that only a single chip is specified:
    let _ = esp_metadata::Chip::from_cargo_feature().unwrap();

    // emit config
    generate_config(
        "esp_backtrace",
        &[ConfigOption::new(
            "backtrace-frames",
            "The maximum number of frames that will be printed in a backtrace",
            10,
        )],
    );
}
