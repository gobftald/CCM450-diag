// 1
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File},
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

// 11
use esp_config::{generate_config, ConfigOption, Value};
use esp_metadata::{Chip, Config};

// 14
fn main() -> Result<(), Box<dyn Error>> {
    if let Ok(level) = std::env::var("OPT_LEVEL") {
        if level == "0" || level == "1" {
            let message = format!(
                "We *strongly* recommend using release profile when building esp-hal. \
                The dev profile (pr level 0 or level 1 can potentially be one or more orders \
                of magnitude slower than release, and may cause issues with timing-senstive \
                peripherals and/or devices."
            );
            println!("cargo:warning={message}");
        }
    }

    // Ensure that exactly one chip has been specified:
    let chip: Chip = Chip::from_cargo_feature()?;

    if chip.target() != std::env::var("TARGET").unwrap_or_default().as_str() {
        panic!("
        Seems you are building for an unsupported or wrong target (e.g. the host environment).
        Maybe you are missing the `target` in `.cargo/config.toml` or you have configs overriding it?

        See https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure
        ");
    }

    // Load the configuration file for the configured device:
    let config = Config::for_chip(&chip);

    // Define all necessary configuration symbols for the configured device:
    config.define_symbols();

    // Place all linker scripts in `OUT_DIR`, and instruct Cargo how to find these
    // files:
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    // emit config
    let cfg = generate_config(
        "esp_hal",
        &[
            ConfigOption::new(
                "place-spi-master-driver-in-ram",
                "Places the SPI master driver in RAM for better performance",
                false,
            ),
            ConfigOption::new(
                "place-switch-tables-in-ram",
                "Places switch-tables, some lookup tables and constants related to \
                interrupt handling into RAM - resulting in better performance but slightly more \
                RAM consumption.",
                true,
            ),
            ConfigOption::new(
                "place-anon-in-ram",
                "Places anonymous symbols into RAM - resulting in better performance \
                at the cost of significant more RAM consumption. Best to be combined with \
                `place-switch-tables-in-ram`.",
                false,
            ),
            // Ideally, we should be able to set any clock frequency for any chip. However,
            // currently only the 32 and C2 implements any sort of configurability, and
            // the rest have a fixed clock frequeny.
            ConfigOption::new(
                "xtal-frequency",
                "The frequency of the crystal oscillator, in MHz. Set to `auto` to \
                automatically detect the frequency. `auto` may not be able to identify the clock \
                frequency in some cases. Also, configuring a specific frequency may increase \
                performance slightly.",
                match chip {
                    Chip::Esp32 | Chip::Esp32c2 => "auto",
                    // The rest has only one option
                    Chip::Esp32c3 | Chip::Esp32c6 | Chip::Esp32s2 | Chip::Esp32s3 => "40",
                    Chip::Esp32h2 => "32",
                },
            )
            .active([Chip::Esp32, Chip::Esp32c2].contains(&chip)),
            ConfigOption::new(
                "spi-address-workaround",
                "Enables a workaround for the issue where SPI in \
                half-duplex mode incorrectly transmits the address on a single line if the \
                data buffer is empty.",
                true,
            )
            .active(chip == Chip::Esp32),
            ConfigOption::new(
                "flip-link",
                "Move the stack to start of RAM to get zero-cost stack overflow protection.",
                false,
            )
            .active([Chip::Esp32c6, Chip::Esp32h2].contains(&chip)),
            ConfigOption::new("psram-mode", "SPIRAM chip mode", "quad").active(
                config
                    .symbols()
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case("psram")),
            ),
            // Rust's stack smashing protection configuration
            ConfigOption::new(
                "stack-guard-offset",
                "The stack guard variable will be placed this many bytes from \
                the stack's end.",
                4096,
            ),
            ConfigOption::new(
                "stack-guard-value",
                "The value to be written to the stack guard variable.",
                0xDEED_BAAD,
            ),
            ConfigOption::new(
                "impl-critical-section",
                "Provide a `critical-section` implementation. Note that if disabled, \
                you will need to provide a `critical-section` implementation which is \
                using `restore-state-u32`.",
                true,
            ),
        ],
    );

    // RISC-V and Xtensa devices each require some special handling and processing
    // of linker scripts:

    #[allow(unused_mut)]
    let mut config_symbols = config.all().collect::<Vec<_>>();

    for (key, value) in &cfg {
        if let Value::Bool(true) = value {
            config_symbols.push(key);
        }
    }

    // RISC-V devices:

    preprocess_file(
        &config_symbols,
        &cfg,
        "ld/riscv/asserts.x",
        out.join("asserts.x"),
    )?;
    preprocess_file(
        &config_symbols,
        &cfg,
        "ld/riscv/debug.x",
        out.join("debug.x"),
    )?;
    preprocess_file(
        &config_symbols,
        &cfg,
        "ld/riscv/hal-defaults.x",
        out.join("hal-defaults.x"),
    )?;

    // With the architecture-specific linker scripts taken care of, we can copy all
    // remaining linker scripts which are common to all devices:
    copy_dir_all(&config_symbols, &cfg, "ld/sections", &out)?;
    copy_dir_all(&config_symbols, &cfg, format!("ld/{chip}"), &out)?;

    Ok(())
}

// 228
// ----------------------------------------------------------------------------
// Helper Functions

// 231
fn copy_dir_all(
    config_symbols: &[&str],
    cfg: &HashMap<String, Value>,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(
                config_symbols,
                cfg,
                entry.path(),
                dst.as_ref().join(entry.file_name()),
            )?;
        } else {
            preprocess_file(
                config_symbols,
                cfg,
                entry.path(),
                dst.as_ref().join(entry.file_name()),
            )?;
        }
    }
    Ok(())
}

/// A naive pre-processor for linker scripts
// 261
fn preprocess_file(
    config: &[&str],
    cfg: &HashMap<String, Value>,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={}", src.as_ref().display());

    let file = File::open(src)?;
    let mut out_file = File::create(dst)?;

    let mut take = Vec::new();
    take.push(true);

    for line in std::io::BufReader::new(file).lines() {
        let line = substitute_config(cfg, &line?);
        let trimmed = line.trim();

        if let Some(condition) = trimmed.strip_prefix("#IF ") {
            let should_take = take.iter().all(|v| *v);
            let should_take = should_take && config.contains(&condition);
            take.push(should_take);
            continue;
        } else if trimmed == "#ELSE" {
            let taken = take.pop().unwrap();
            let should_take = take.iter().all(|v| *v);
            let should_take = should_take && !taken;
            take.push(should_take);
            continue;
        } else if trimmed == "#ENDIF" {
            take.pop();
            continue;
        }

        if *take.last().unwrap() {
            out_file.write_all(line.as_bytes())?;
            let _ = out_file.write(b"\n")?;
        }
    }
    Ok(())
}

// 303
fn substitute_config(cfg: &HashMap<String, Value>, line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }

        let Some('{') = chars.peek() else {
            result.push(c);
            continue;
        };
        chars.next();

        let mut key = String::new();
        for c in chars.by_ref() {
            if c == '}' {
                break;
            }
            key.push(c);
        }
        match cfg
            .get(&key)
            .unwrap_or_else(|| panic!("missing config key: {key}"))
        {
            Value::Bool(true) => result.push('1'),
            Value::Bool(false) => result.push('0'),
            Value::Integer(value) => result.push_str(&value.to_string()),
            Value::String(value) => result.push_str(value),
        }
    }

    result
}
