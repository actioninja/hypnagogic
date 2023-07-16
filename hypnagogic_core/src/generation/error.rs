use thiserror::Error;

use crate::generation::text;

#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Failed to generate text: {0}")]
    TextError(#[from] text::TextError),
    #[error("Text too long: \"{0}\", max length for size is (around) {1}")]
    TextTooLong(String, u32),
    #[error("Text has too many lines: {0}; max lines for size is {1}")]
    TooManyLines(u32, u32),
}
