use crate::config::TemplatedConfig;
use anyhow::{bail, Result};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

pub trait TemplateResolver {
    fn resolve(&self, input: &str) -> Result<TemplatedConfig>;
}

/// Simple resolver that always returns default templatedconfig
/// For testing or otherwise situations where you want to not actually do resolution
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NullResolver;

impl TemplateResolver for NullResolver {
    fn resolve(&self, _: &str) -> Result<TemplatedConfig> {
        Ok(TemplatedConfig::default())
    }
}

/// Loads templates from a folder on the filesystem.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileResolver {
    path: PathBuf,
}

impl FileResolver {
    /// Creates a new `FileResolver` with the given path
    /// # Errors
    /// Returns an error if `path` does not exist.
    pub fn new(path: &Path) -> Result<Self> {
        let pathbuf = fs::canonicalize(path)?;
        Ok(FileResolver { path: pathbuf })
    }
}

impl Default for FileResolver {
    fn default() -> Self {
        FileResolver::new(Path::new("templates")).expect("templates folder does not exist")
    }
}

#[derive(Error, Debug)]
pub enum FileResolverError {
    #[error("Template not found: {0:?}")]
    TemplateNotFound(PathBuf),
}

impl TemplateResolver for FileResolver {
    #[tracing::instrument(skip(input))]
    fn resolve(&self, input: &str) -> Result<TemplatedConfig> {
        let mut pathbuf = self.path.clone();
        pathbuf.push(Path::new(input));

        debug!("Full path: {:?}", pathbuf);

        let yaml_path = pathbuf.with_extension("yml");
        let yml_path = pathbuf.with_extension("yaml");

        pathbuf = if yaml_path.exists() {
            yaml_path
        } else if yml_path.exists() {
            yml_path
        } else {
            return Err(FileResolverError::TemplateNotFound(pathbuf).into());
        };

        debug!("Found template at {:?}", pathbuf);

        let file = File::open(pathbuf.as_path())?;
        let mut reader = BufReader::new(file);

        let deserialized: TemplatedConfig = serde_yaml::from_reader(reader)?;
        Ok(deserialized)
    }
}
