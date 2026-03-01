use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, Write},
    path::Path,
};

use esp_config::{Value, generate_config_from_yaml_definition};
use esp_metadata_generated::Chip;

// 6
fn main() -> Result<(), Box<dyn Error>> {
    // Load the configuration file for the configured device:
    let chip = Chip::from_cargo_feature()?;

    // Define all necessary configuration symbols for the configured device:
    chip.define_cfgs();

    if let Ok(level) = std::env::var("OPT_LEVEL")
        && level != "2"
        && level != "3"
        && level != "s"
    {
        let message = format!(
            "esp-wifi should be built with optimization level 2, 3 or s - yours is {level}. \
                See https://github.com/esp-rs/esp-wifi",
        );
        println!("cargo:warning={message}");
    }

    println!("cargo:rustc-check-cfg=cfg(coex)");

    // Manually register and import this shared cfg used by esp-rtos, esp-radio
    // and esp-radio-rtos-driver.
    // It enables task name string support in the RTOS scheduler, significantly
    // facilitate tracing in eps-rtos.
    println!("cargo:rustc-check-cfg=cfg(esp_rtos_task_name_str)");
    if std::env::var("ESP_RTOS_TASK_NAME_STR").map(|v| v == "true").unwrap_or(false) {
        println!("cargo:rustc-cfg=esp_rtos_task_name_str");
    }

    // emit config
    println!("cargo:rerun-if-changed=./esp_config.yml");
    let cfg_yaml = std::fs::read_to_string("./esp_config.yml")
        .expect("Failed to read esp_config.yml for esp-hal");
    let cfg = generate_config_from_yaml_definition(&cfg_yaml, true, true, Some(chip)).unwrap();

    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=./ld/");

    let config = [
        #[cfg(feature = "wifi")]
        "wifi",
    ];
    preprocess_file(
        &config,
        &cfg,
        format!("./ld/{}_provides.x", chip.name()),
        out.join("libesp-radio.a"),
    )?;
    // exploit the fact that linkers treat an unknown library format as a linker
    // script
    println!("cargo:rustc-link-lib=esp-radio");

    Ok(())
}

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
