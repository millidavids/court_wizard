use thiserror::Error;

/// Errors that can occur when working with configuration files.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read the configuration file from disk.
    #[error("Failed to read config file: {0}")]
    Read(#[from] std::io::Error),

    /// Failed to parse the TOML configuration file.
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    /// Failed to serialize configuration to TOML.
    #[error("Failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// Type alias for Results that can return ConfigError.
pub type ConfigResult<T> = Result<T, ConfigError>;
