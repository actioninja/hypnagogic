use serde_yaml::Value;

use crate::config::template_resolver::error::TemplateResult;

pub mod error;
pub mod file_resolver;

pub trait TemplateResolver {
    /// Determines how exactly to resolve template strings. Primarily for the ability to manually
    /// pass them in without accessing FS in tests
    /// # Errors
    /// Throws an error if resolution fails
    fn resolve(&self, input: &str) -> TemplateResult;
}

/// Simple resolver that always returns default templatedconfig
/// For testing or otherwise situations where you want to not actually do resolution
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NullResolver;

impl TemplateResolver for NullResolver {
    fn resolve(&self, _: &str) -> TemplateResult {
        Ok(Value::default())
    }
}
