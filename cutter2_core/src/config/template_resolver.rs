use crate::config::TemplatedConfig;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub trait TemplateResolver {
    fn resolve(&self, input: &str) -> anyhow::Result<TemplatedConfig>;
}

/// Simple resolver that always returns default templatedconfig
/// For testing or otherwise situations where you want to not actually do resolution
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NullResolver;

impl TemplateResolver for NullResolver {
    fn resolve(&self, _: &str) -> anyhow::Result<TemplatedConfig> {
        Ok(TemplatedConfig::default())
    }
}

/// Loads templates from a folder on the filesystem.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileResolver {
    path: PathBuf,
}

impl FileResolver {
    /// Creates a new FileResolver with the given path
    /// # Errors
    /// Returns an error if `path` does not exist.
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let pathbuf = fs::canonicalize(path)?;
        Ok(FileResolver { path: pathbuf })
    }
}

impl TemplateResolver for FileResolver {
    fn resolve(&self, input: &str) -> anyhow::Result<TemplatedConfig> {
        let mut pathbuf = self.path.clone();
        pathbuf.push(Path::new(input));

        let file = File::open(pathbuf.as_path())?;
        let mut reader = BufReader::new(file);

        let deserialized: TemplatedConfig = serde_yaml::from_reader(reader)?;
        Ok(deserialized)
    }
}
