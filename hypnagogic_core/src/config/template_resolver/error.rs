use serde_yaml::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Failed to find template: `{0}`")]
    FailedToFindTemplate(String),
    #[error("Generic yaml parse error while resolving template: {0}")]
    YAMLError(#[from] serde_yaml::Error),
    #[error("Generic IO Error when attempting to resolve template: {0}")]
    IOError(#[from] std::io::Error),
}

pub type TemplateResult = Result<Value, TemplateError>;
