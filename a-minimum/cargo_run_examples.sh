cargo run --release                     # default log level is error -> [ERROR] - error log: 2024

DEFMT_LOG=a_minimum cargo run --release # log level is trace ->         [ERROR] - error log: 2024
                                                                      # [WARN] - warn log: 2024
                                                                      # [INFO] - info log: 2024
                                                                      # [DEBUG] - debug log: 2024
                                                                      # [TRACE] - trace log: 2024

DEFMT_LOG=warn cargo run --release      # log level is warn ->        # [ERROR] - error log: 2024
                                                                      # [WARN] - warn log: 2024

DEFMT_LOG=off cargo run --release

# not(features = defmt), config.toml reset to no defmt settings
cargo run --release

# not(features = defmt), features = no-op, config.toml reset to no defmt settings
cargo run --release
