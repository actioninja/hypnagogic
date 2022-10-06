use std::fmt::Formatter;
use std::path::PathBuf;
use thiserror::__private::PathAsDisplay;

use thiserror::Error;

use crate::config::template_resolver::error::TemplateError;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Error while resolving template:\n{0}")]
    TemplateError(#[from] TemplateError),
    #[error("Error while parsing config into yaml:\n{0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Failed to find template directory (expected at `{0:?}`)")]
    NoTemplateDir(PathBuf),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
