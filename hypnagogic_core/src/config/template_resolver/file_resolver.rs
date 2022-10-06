use core::default::Default;
use core::result::Result::{Err, Ok};
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde_yaml::value::Value;
use tracing::{debug, trace};


use crate::config::template_resolver::error::{TemplateError};
use crate::config::template_resolver::TemplateResolver;

/// Loads templates from a folder on the filesystem.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileResolver {
    path: PathBuf,
}

#[derive(Debug)]
pub struct NoTemplateDirError(PathBuf);

impl std::fmt::Display for NoTemplateDirError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Template dir not found while creating FileResolver: {:?}",
            self.0
        )
    }
}

impl std::error::Error for NoTemplateDirError {}

impl FileResolver {
    /// Creates a new `FileResolver` with the given path
    /// # Errors
    /// Returns an error if `path` does not exist.
    pub fn new(path: &Path) -> Result<Self, NoTemplateDirError> {
        let pathbuf =
            fs::canonicalize(path).map_err(|_e| NoTemplateDirError(path.to_path_buf()))?;
        Ok(FileResolver { path: pathbuf })
    }
}

impl Default for FileResolver {
    fn default() -> Self {
        FileResolver::new(Path::new("templates")).expect("templates folder does not exist")
    }
}

impl TemplateResolver for FileResolver {
    #[tracing::instrument(skip(input))]
    fn resolve(&self, input: &str) -> Result<Value, TemplateError> {
        let mut pathbuf = self.path.clone();
        pathbuf.push(Path::new(input));

        debug!(canon = ?pathbuf, "Full path parsed");

        let yaml_path = pathbuf.with_extension("yml");
        let yml_path = pathbuf.with_extension("yaml");

        pathbuf = if yaml_path.exists() {
            yaml_path
        } else if yml_path.exists() {
            yml_path
        } else {
            return Err(TemplateError::FailedToFindTemplate(
                input.to_string(),
                yaml_path.with_extension("yml|yaml"),
            ));
        };

        trace!("Found template at {:?}", pathbuf);

        let file = File::open(pathbuf.as_path())?;
        let reader = BufReader::new(file);

        let deserialized: Value = serde_yaml::from_reader(reader)?;
        debug!(deserialized = ?deserialized, "Deserialized template");
        Ok(deserialized)
    }
}
