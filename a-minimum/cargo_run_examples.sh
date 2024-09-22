ESP_LOGLEVEL=trace cargo run --release
ESP_LOGLEVEL=error cargo run --release

ESP_LOGTARGETS=esp_hal ESP_LOGLEVEL=trace cargo run --release
ESP_LOGTARGETS=a_minimum ESP_LOGLEVEL=trace cargo run --release
ESP_LOGTARGETS=off ESP_LOGLEVEL=trace cargo run --release

ESP_LOGLEVEL=trace cargo run --release --features=no-op