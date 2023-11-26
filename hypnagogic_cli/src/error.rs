use std::io;
use std::path::PathBuf;

use hypnagogic_core::config::error::ConfigError;
use thiserror::Error;
use user_error::UFE;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Input not found")]
    InputNotFound {
        source_config: String,
        expected: String,
        search_dir: PathBuf,
    },
    #[error("Invalid Config File")]
    InvalidConfig {
        source_config: String,
        config_error: ConfigError,
    },
    #[error("Template Not Found")]
    TemplateNotFound {
        source_config: String,
        template_string: String,
        expected_path: PathBuf,
    },
    #[error("No template folder")]
    NoTemplateFolder(PathBuf),
    #[error("Generic IO Error")]
    IO(#[from] io::Error),
}

impl UFE for Error {
    fn summary(&self) -> String {
        format!("{}", self)
    }

    fn reasons(&self) -> Option<Vec<String>> {
        match self {
            Error::InputNotFound {
                source_config,
                expected,
                search_dir,
            } => Some(vec![
                format!("Failed to find the input for a config ({source_config})"),
                format!("Searched in `{search_dir:?}`"),
                format!("Expected to find an input file named \"{expected}\""),
            ]),
            Error::InvalidConfig {
                source_config,
                config_error,
            } => Some(vec![
                format!("Error within config \"{source_config}\""),
                format!("{}", config_error),
            ]),
            Error::TemplateNotFound {
                source_config,
                template_string,
                expected_path,
            } => Some(vec![
                format!("Failed to find the template referenced in a config ({source_config})"),
                format!("Config string was \"{template_string}\""),
                format!("Expected to find a config at {expected_path:?}"),
            ]),
            Error::NoTemplateFolder(folder) => Some(vec![
                format!("Failed to find template folder"),
                format!("Expected template folder at {folder:?}"),
            ]),
            Error::IO(err) => Some(vec![format!(
                "Operation failed for reason of \"{:?}\"",
                err.kind()
            )]),
        }
    }

    fn helptext(&self) -> Option<String> {
        match self {
            Error::InputNotFound { expected, .. } => Some(format!(
                "Double check that the file \"{expected}\" exists, and if it does, that it's \
                     named correctly"
            )),
            Error::InvalidConfig { .. } => Some(
                "Make sure the config conforms to the schema, and that all values are valid"
                    .to_string(),
            ),
            Error::TemplateNotFound { .. } => Some(
                "Make sure you have spelled the template correctly, and that it exists".to_string(),
            ),
            Error::NoTemplateFolder(_) => Some(
                "Check that you have spelled your template dir correctly, and make sure it \
                     exists"
                    .to_string(),
            ),
            Error::IO(_) => Some(
                "Make sure the directories or files aren't in use, and you have permission to \
                     access them"
                    .to_string(),
            ),
        }
    }
}
