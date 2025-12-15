// 1
use std::error::Error;

// 3
//use esp_config::{generate_config, ConfigOption};
use esp_config::generate_config_from_yaml_definition;
//use esp_metadata::{Chip, Config};

// 6
fn main() -> Result<(), Box<dyn Error>> {
    // Load the configuration file for the configured device:
    //let chip = Chip::from_cargo_feature()?;
    let chip = esp_metadata_generated::Chip::from_cargo_feature()?;
    //let config = Config::for_chip(&chip);
    chip.define_cfgs();

    // emit config
    println!("cargo:rerun-if-changed=./esp_config.yml");
    let cfg_yaml = std::fs::read_to_string("./esp_config.yml")
        .expect("Failed to read esp_config.yml for esp-hal");
    generate_config_from_yaml_definition(&cfg_yaml, true, true, Some(chip)).unwrap();

    /*
    // Define all necessary configuration symbols for the configured device:
    config.define_symbols();
    */

    /*
    assert!(
        !cfg!(feature = "wifi") || config.contains("wifi"),
        r#"

        WiFi is not supported on this target.

        "#
    );
    */

    if let Ok(level) = std::env::var("OPT_LEVEL") {
        if level != "2" && level != "3" && level != "s" {
            let message = format!(
                "esp-wifi should be built with optimization level 2, 3 or s - yours is {level}. \
                See https://github.com/esp-rs/esp-wifi",
            );
            println!("cargo:warning={message}");
        }
    }

    println!("cargo:rustc-check-cfg=cfg(coex)");

    /*
    // emit config
    //
    // keep the defaults aligned with `esp_wifi_sys::include::*` e.g.
    // `esp_wifi_sys::include::CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM`
    generate_config(
        "esp_wifi",
        &[
            ConfigOption::new("rx_queue_size", "Size of the RX queue in frames", 5),
            ConfigOption::new("tx_queue_size", "Size of the TX queue in frames", 3),
            ConfigOption::new(
                "static_rx_buf_num",
                "WiFi static RX buffer number. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                10,
            ),
            ConfigOption::new(
                "dynamic_rx_buf_num",
                "WiFi dynamic RX buffer number. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                32,
            ),
            ConfigOption::new(
                "static_tx_buf_num",
                "WiFi static TX buffer number. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                0,
            ),
            ConfigOption::new(
                "dynamic_tx_buf_num",
                "WiFi dynamic TX buffer number. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                32,
            ),
            ConfigOption::new(
                "ampdu_rx_enable",
                "WiFi AMPDU RX feature enable flag. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                true,
            ),
            ConfigOption::new(
                "ampdu_tx_enable",
                "WiFi AMPDU TX feature enable flag. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                true,
            ),
            ConfigOption::new(
                "amsdu_tx_enable",
                "WiFi AMSDU TX feature enable flag. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                false,
            ),
            ConfigOption::new(
                "rx_ba_win",
                "WiFi Block Ack RX window size. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/\
                 network/esp_wifi.html#_CPPv418wifi_init_config_t)",
                6,
            ),
            ConfigOption::new(
                "max_burst_size",
                "See [smoltcp's documentation]\
                 (https://docs.rs/smoltcp/0.10.0/smoltcp/phy/struct.DeviceCapabilities.html\
                 #structfield.max_burst_size)",
                1,
            ),
            ConfigOption::new(
                "country_code",
                "Country code. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/\
                 wifi.html#wi-fi-country-code)",
                "HU",
            ),
            ConfigOption::new(
                "country_code_operating_class",
                "If not 0: Operating Class table number. See [ESP-IDF Programming Guide]\
                 (https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/\
                 wifi.html#wi-fi-country-code)",
                0,
            ),
            ConfigOption::new(
                "mtu",
                "MTU, see [smoltcp's documentation]\
                 (https://docs.rs/smoltcp/0.10.0/smoltcp/phy/struct.DeviceCapabilities.html\
                 #structfield.max_transmission_unit)",
                1492,
            ),
            ConfigOption::new(
                "tick_rate_hz",
                "Tick rate of the internal task scheduler in hertz",
                100,
            ),
            ConfigOption::new(
                "listen_interval",
                "Interval for station to listen to beacon from AP.
                 The unit of listen interval is one beacon interval.
                 For example, if beacon interval is 100 ms and listen interval is 3,
                 the interval for station to listen to beacon is 300 ms",
                3,
            ),
            ConfigOption::new(
                "beacon_timeout",
                "For Station, If the station does not receive a beacon frame
                 from the connected SoftAP during the  inactive time, disconnect from SoftAP.
                 Default 6s. Range 6-30",
                6,
            ),
            ConfigOption::new(
                "ap_beacon_timeout",
                "For SoftAP, If the SoftAP doesn't receive any data from the connected STA
                 during inactive time, the SoftAP will force deauth the STA. Default is 300s",
                300,
            ),
            ConfigOption::new(
                "failure_retry_cnt",
                "Number of connection retries station will do before moving to next AP.
                scan_method should be set as WIFI_ALL_CHANNEL_SCAN to use this config.
                Note: Enabling this may cause connection time to increase incase best AP
                doesn't behave properly. Defaults to 1",
                1,
            ),
            ConfigOption::new(
                "scan_method",
                "0 = WIFI_FAST_SCAN, 1 = WIFI_ALL_CHANNEL_SCAN, defaults to 0",
                0,
            ),
            ConfigOption::new(
                "dump_packets",
                "Dump packets via an info log statement",
                true,
            ),
            ConfigOption::new(
                "phy_enable_usb",
                "Keeps USB running when using WiFi.
                 This allows debugging and log messages via USB Serial JTAG.
                 Turn off for best WiFi performance.",
                true,
            ),
            ConfigOption::new(
                "phy_skip_calibration_after_deep_sleep",
                "Use PHY_RF_CAL_NONE after deep sleep.",
                false,
            ),
            ConfigOption::new(
                "phy_full_calibration",
                "Use PHY_RF_CAL_FULL instead of PHY_RF_CAL_PARTIAL.",
                true,
            ),
        ],
    );
    */

    Ok(())
}
