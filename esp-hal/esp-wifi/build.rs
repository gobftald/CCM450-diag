use std::error::Error;

use esp_metadata::{Chip, Config};

fn main() -> Result<(), Box<dyn Error>> {
    // Load the configuration file for the configured device:
    let chip = Chip::from_cargo_feature()?;
    let config = Config::for_chip(&chip);

    // Define all necessary configuration symbols for the configured device:
    config.define_symbols();

    if let Ok(level) = std::env::var("OPT_LEVEL") {
        if level != "2" && level != "3" && level != "s" {
            let message = format!(
                "esp-wifi should be built with optimization level 2, 3 or s - yours is {level}.
                See https://github.com/esp-rs/esp-wifi",
            );
            print_warning(message);
        }
    }

    Ok(())
}

fn print_warning(message: impl core::fmt::Display) {
    println!("cargo:warning={message}");
}
