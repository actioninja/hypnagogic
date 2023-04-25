use crate::generation::text;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Failed to generate text: {0}")]
    TextError(#[from] text::TextError),
}
