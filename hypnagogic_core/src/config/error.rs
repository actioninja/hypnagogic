use thiserror::Error;

use crate::config::template_resolver::error::TemplateError;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Error while resolving template:\n{0}")]
    Template(#[from] TemplateError),
    #[error("Error while parsing config into yaml:\n{0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("error in config")]
    Config(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
