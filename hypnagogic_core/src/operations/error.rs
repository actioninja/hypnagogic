use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("Error processing image:\n{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("Error within image config:")]
    ConfigError,
}

pub type ProcessorResult<T> = Result<T, ProcessorError>;
