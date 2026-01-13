//1
use std::error::Error as StdError;

// 3
//use esp_config::{generate_config, ConfigOption, Value};
use esp_config::generate_config_from_yaml_definition;
//use esp_metadata::{Chip, Config};

// 6
fn main() -> Result<(), Box<dyn StdError>> {
    // Load the configuration file for the configured device:
    //let chip = Chip::from_cargo_feature()?;
    let chip = esp_metadata_generated::Chip::from_cargo_feature()?;
    //let config = Config::for_chip(&chip);
    chip.define_cfgs();

    /*
    // Define all necessary configuration symbols for the configured device:
    config.define_symbols();
    */

    // emit config
    println!("cargo:rerun-if-changed=./esp_config.yml");
    let cfg_yaml = std::fs::read_to_string("./esp_config.yml")
        .expect("Failed to read esp_config.yml for esp-hal");
    generate_config_from_yaml_definition(&cfg_yaml, true, true, Some(chip)).unwrap();

    /*
    // emit config
    let crate_config = generate_config(
        "esp_hal_embassy",
        &[
            ConfigOption::new(
                "low-power-wait",
                "Enables the lower-power wait if no tasks are ready to run on the \
                thread-mode executor. This allows the MCU to use less power if the workload allows. \
                Recommended for battery-powered systems. May impact analog performance.",
                true,
            ),
            ConfigOption::new(
                "timer-queue",
                "The flavour of the timer queue provided by this crate. Integrated \
                queues require the `executors` feature to be enabled.</p><p>If you use \
                embassy-executor, the `single-integrated` queue is recommended for ease of use, \
                while the `multiple-integrated` queue is recommended for performance. The \
                `multiple-integrated` option needs one timer per executor.</p><p>The `generic` \
                queue allows using embassy-time without the embassy executors.",
                if cfg!(feature = "executors") {
                    "single-integrated"
                } else {
                    "generic"
                },
            )
            .active(cfg!(feature = "executors")),
            ConfigOption::new(
                "generic-queue-size",
                "The capacity of the queue when the `generic` timer \
                queue flavour is selected.",
                64,
            ),
        ],
    );
    */

    println!("cargo:rustc-check-cfg=cfg(integrated_timers)");
    println!("cargo:rustc-check-cfg=cfg(single_queue)");
    println!("cargo:rustc-check-cfg=cfg(generic_timers)");

    /*
    match &crate_config["ESP_HAL_EMBASSY_CONFIG_TIMER_QUEUE"] {
        Value::String(s) if s.as_str() == "single-integrated" => {
            println!("cargo:rustc-cfg=integrated_timers");
            println!("cargo:rustc-cfg=single_queue");
        }
        Value::String(s) if s.as_str() == "multiple-integrated" => {
            println!("cargo:rustc-cfg=integrated_timers");
        }
        Value::String(s) if s.as_str() == "generic" => {
            println!("cargo:rustc-cfg=generic_timers");
            println!("cargo:rustc-cfg=single_queue");
        }
        _ => unreachable!(),
    }
    */

    println!("cargo:rustc-cfg=integrated_timers");
    println!("cargo:rustc-cfg=single_queue");

    Ok(())
}
