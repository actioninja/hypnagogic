use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessorError {
    #[error("Error processing image:\n{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("Error generating icon for processor:\n{0}")]
    GenerationError(#[from] crate::generation::error::GenerationError),
    #[error("Error within image config:")]
    ConfigError,
}

pub type ProcessorResult<T> = Result<T, ProcessorError>;
