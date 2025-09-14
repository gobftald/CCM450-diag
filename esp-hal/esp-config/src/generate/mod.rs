// 2
use std::{collections::HashMap, env, fmt, io::Write};

// 10
pub(crate) mod value;

// 6
use crate::generate::value::Value;

/// Configuration errors.
#[derive(Debug, Clone)]
// 14
pub enum Error {
    /// Parse errors.
    Parse(String),
    /*
    /// Validation errors.
    Validation(String),
    */
}

// 21
impl Error {
    /// Convenience function for creating parse errors.
    // 23
    pub fn parse<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Parse(message.into())
    }
}

// 39
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(message) => write!(f, "{message}"),
            //Error::Validation(message) => write!(f, "{message}"),
        }
    }
}

// 71
pub fn generate_config(crate_name: &str, config: &[ConfigOption]) -> HashMap<String, Value> {
    let configs = generate_config_internal(std::io::stdout(), crate_name, config);

    // Remove the ConfigOptions from the output
    configs.into_iter().map(|(k, _, v)| (k, v)).collect()
}

// 104
pub fn generate_config_internal<'a>(
    mut stdout: impl Write,
    crate_name: &str,
    config: &'a [ConfigOption],
) -> Vec<(String, &'a ConfigOption, Value)> {
    // Only rebuild if `build.rs` changed. Otherwise, Cargo will rebuild if any
    // other file changed.
    writeln!(stdout, "cargo:rerun-if-changed=build.rs").ok();

    // Ensure that the prefix is `SCREAMING_SNAKE_CASE`:
    let prefix = format!("{}_CONFIG_", screaming_snake_case(crate_name));

    let mut configs = create_config(&prefix, config);
    capture_from_env(&prefix, &mut configs);

    emit_configuration(&mut stdout, &configs);

    configs
}

/// A configuration option.
#[derive(Debug, Clone, PartialEq, Eq)]
// 194
pub struct ConfigOption {
    /// The name of the configuration option.
    ///
    /// The associated environment variable has the format of
    /// `<PREFIX>_CONFIG_<NAME>`.
    pub name: String,

    /// The description of the configuration option.
    ///
    /// The description will be included in the generated markdown
    /// documentation.
    pub description: String,

    /// The default value of the configuration option.
    pub default_value: Value,

    /// An optional validator for the configuration option.
    //pub constraint: Option<Validator>,

    /*
    /// The stability of the configuration option.
    pub stability: Stability,
    */

    /// Whether the config option should be offered to the user.
    ///
    /// Inactive options are not included in the documentation, and accessing
    /// them provides the default value.
    pub active: bool,
    /*
    /// A display hint (for tooling)
    pub display_hint: DisplayHint,
    */
}

// 226
impl ConfigOption {
    // Create a new config option.
    ///
    /// Unstable, active, no display-hint and not constrained by default.
    // 230
    pub fn new(name: &str, description: &str, default_value: impl Into<Value>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            default_value: default_value.into(),
            //constraint: None,
            //stability: Stability::Unstable,
            active: true,
            //display_hint: DisplayHint::None,
        }
    }

    /// Sets the active flag of this config option
    // 261
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    // 272
    fn env_var(&self, prefix: &str) -> String {
        format!("{}{}", prefix, screaming_snake_case(&self.name))
    }

    // 276
    fn cfg_name(&self) -> String {
        snake_case(&self.name)
    }
}

// 285
fn create_config<'a>(
    prefix: &str,
    config: &'a [ConfigOption],
) -> Vec<(String, &'a ConfigOption, Value)> {
    let mut configs = Vec::with_capacity(config.len());

    for option in config {
        configs.push((option.env_var(prefix), option, option.default_value.clone()));
    }

    configs
}

// 298
fn capture_from_env(prefix: &str, configs: &mut Vec<(String, &ConfigOption, Value)>) {
    let mut unknown = Vec::new();
    let mut failed = Vec::new();

    // Try and capture input from the environment:
    for (var, value) in env::vars() {
        if var.starts_with(prefix) {
            let Some((_, option, cfg)) = configs.iter_mut().find(|(k, _, _)| k == &var) else {
                unknown.push(var);
                continue;
            };

            if !option.active {
                unknown.push(var);
                continue;
            }

            if let Err(e) = cfg.parse_in_place(&value) {
                failed.push(format!("{var}: {e}"));
            }
        }
    }

    if !failed.is_empty() {
        panic!("Invalid configuration options detected: {failed:?}");
    }

    if !unknown.is_empty() {
        panic!("Unknown configuration options detected: {unknown:?}");
    }
}

// 348
fn emit_configuration(mut stdout: impl Write, configs: &[(String, &ConfigOption, Value)]) {
    for (env_var_name, option, value) in configs.iter() {
        let cfg_name = option.cfg_name();

        // Output the raw configuration as an env var. Values that haven't been seen
        // will be output here with the default value. Also trigger a rebuild if config
        // environment variable changed.
        writeln!(stdout, "cargo:rustc-env={env_var_name}={value}").ok();
        writeln!(stdout, "cargo:rerun-if-env-changed={env_var_name}").ok();

        // Emit known config symbol:
        writeln!(stdout, "cargo:rustc-check-cfg=cfg({cfg_name})").ok();

        // Emit specially-handled values:
        if let Value::Bool(true) = value {
            writeln!(stdout, "cargo:rustc-cfg={cfg_name}").ok();
        }
    }
}

// 379
fn snake_case(name: &str) -> String {
    let mut name = name.replace("-", "_");
    name.make_ascii_lowercase();

    name
}

// 386
fn screaming_snake_case(name: &str) -> String {
    let mut name = name.replace("-", "_");
    name.make_ascii_uppercase();

    name
}
