use std::sync::OnceLock;

use strum::IntoEnumIterator;

// 7
macro_rules! include_toml {
    ($type:ty, $file:expr) => {{
        static LOADED_TOML: OnceLock<$type> = OnceLock::new();
        LOADED_TOML.get_or_init(|| basic_toml::from_str(include_str!($file)).unwrap())
    }};
}

/// Supported device architectures.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    strum::AsRefStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
// 32
pub enum Arch {
    /// RISC-V architecture
    RiscV,
    /// Xtensa architecture
    Xtensa,
}

/// Device core count.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    strum::AsRefStr,
)]
// 55
pub enum Cores {
    /// Single CPU core
    #[serde(rename = "single_core")]
    #[strum(serialize = "single_core")]
    Single,
    /// Two or more CPU cores
    #[serde(rename = "multi_core")]
    #[strum(serialize = "multi_core")]
    Multi,
}

/// Supported devices.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    strum::AsRefStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
// 86
pub enum Chip {
    /// ESP32
    Esp32,
    /// ESP32-C2, ESP8684
    Esp32c2,
    /// ESP32-C3, ESP8685
    Esp32c3,
    /// ESP32-C6
    Esp32c6,
    /// ESP32-H2
    Esp32h2,
    /// ESP32-S2
    Esp32s2,
    /// ESP32-S3
    Esp32s3,
}

// 103
impl Chip {
    // 130
    pub fn target(&self) -> &str {
        use Chip::*;

        match self {
            Esp32 => "xtensa-esp32-none-elf",
            Esp32c2 | Esp32c3 => "riscv32imc-unknown-none-elf",
            Esp32c6 | Esp32h2 => "riscv32imac-unknown-none-elf",
            Esp32s2 => "xtensa-esp32s2-none-elf",
            Esp32s3 => "xtensa-esp32s3-none-elf",
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
// 180
pub struct MemoryRegion {
    name: String,
    start: u32,
    end: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
// 187
struct Device {
    pub name: String,
    pub arch: Arch,
    pub cores: Cores,
    pub peripherals: Vec<String>,
    pub symbols: Vec<String>,
    pub memory: Vec<MemoryRegion>,
}

/// Device configuration file format.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
// 198
pub struct Config {
    device: Device,
}

// 202
impl Config {
    /// The configuration for the specified chip.
    // 204
    pub fn for_chip(chip: &Chip) -> &Self {
        match chip {
            Chip::Esp32 => include_toml!(Config, "../devices/esp32.toml"),
            Chip::Esp32c2 => include_toml!(Config, "../devices/esp32c2.toml"),
            Chip::Esp32c3 => include_toml!(Config, "../devices/esp32c3.toml"),
            Chip::Esp32c6 => include_toml!(Config, "../devices/esp32c6.toml"),
            Chip::Esp32h2 => include_toml!(Config, "../devices/esp32h2.toml"),
            Chip::Esp32s2 => include_toml!(Config, "../devices/esp32s2.toml"),
            Chip::Esp32s3 => include_toml!(Config, "../devices/esp32s3.toml"),
        }
    }

    /// Memory regions.
    ///
    /// Will be available as env-variables `REGION-<NAME>-START` /
    /// `REGION-<NAME>-END`
    // 259
    pub fn memory(&self) -> &[MemoryRegion] {
        &self.device.memory
    }

    /// All configuration values for the device.
    // 264
    pub fn all(&self) -> impl Iterator<Item = &str> + '_ {
        [
            self.device.name.as_str(),
            self.device.arch.as_ref(),
            self.device.cores.as_ref(),
        ]
        .into_iter()
        .chain(self.device.peripherals.iter().map(|s| s.as_str()))
        .chain(self.device.symbols.iter().map(|s| s.as_str()))
    }

    /// Define all symbols for a given configuration.
    // 281
    pub fn define_symbols(&self) {
        define_all_possible_symbols();
        // Define all necessary configuration symbols for the configured device:
        for symbol in self.all() {
            println!("cargo:rustc-cfg={symbol}");
        }

        // Define env-vars for all memory regions
        for memory in self.memory() {
            println!("cargo:rustc-cfg=has_{}_region", memory.name.to_lowercase());

            println!(
                "cargo::rustc-env=ESP_METADATA_REGION_{}_START={}",
                memory.name.to_uppercase(),
                memory.start
            );
            println!(
                "cargo::rustc-env=ESP_METADATA_REGION_{}_END={}",
                memory.name.to_uppercase(),
                memory.end
            );
        }
    }
}

/// Defines all possible symbols that _could_ be output from this crate
/// regardless of the chosen configuration.
///
/// This is required to avoid triggering the unexpected-cfgs lint.
// 310
fn define_all_possible_symbols() {
    // Used by our documentation builds to prevent the huge red warning banner.
    println!("cargo:rustc-check-cfg=cfg(not_really_docsrs)");

    for chip in Chip::iter() {
        let config = Config::for_chip(&chip);
        for symbol in config.all() {
            // https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-check-cfg
            println!("cargo:rustc-check-cfg=cfg({})", symbol);
        }
    }
}
