use thiserror::Error;

use crate::config::template_resolver::error::TemplateError;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Error while resolving template:\n{0}")]
    Template(#[from] TemplateError),
    #[error("Error while parsing config into toml:\n{0}")]
    Toml(#[from] toml::de::Error),
    #[error("error in config")]
    Config(String),
    #[error("Generic IO Error: {0}")]
    IO(#[from] std::io::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
