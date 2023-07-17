use std::fmt::Debug;
use std::io::{BufRead, Read, Seek};
use std::path::{Path, PathBuf};

use cutters::bitmask_dir_visibility::BitmaskDirectionalVis;
use cutters::bitmask_slice::BitmaskSlice;
use cutters::bitmask_windows::BitmaskWindows;
use dmi::error::DmiError;
use dmi::icon::Icon;
use enum_dispatch::enum_dispatch;
use image::{DynamicImage, ImageError, ImageFormat};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use crate::operations::error::ProcessorResult;

pub mod cutters;
pub mod error;
pub mod format_converter;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("This image format is unsupported:\n{0}")]
    UnsupportedFormat(String),
    #[error("Error reading the input as a dynamic image:\n{0}")]
    DynamicRead(#[from] ImageError),
    #[error("Error reading the input stream as a dmi image:\n{0}")]
    DmiRead(#[from] DmiError),
}

#[derive(Clone)]
pub enum InputIcon {
    DynamicImage(DynamicImage),
    Dmi(Icon),
}

impl InputIcon {
    pub fn from_reader<R: BufRead + Seek>(
        reader: &mut R,
        extension: &str,
    ) -> Result<Self, InputError> {
        match extension {
            "png" => Ok(Self::DynamicImage(image::load(reader, ImageFormat::Png)?)),
            "dmi" => Ok(Self::Dmi(Icon::load(reader)?)),
            _ => Err(InputError::UnsupportedFormat(extension.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct NamedIcon {
    pub path_hint: Option<String>,
    pub name_hint: Option<String>,
    pub image: OutputImage,
}

impl Debug for NamedIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedIcon")
            .field("path_hint", &self.path_hint)
            .field("name_hint", &self.name_hint)
            .field("image", &"[OutputImage]")
            .finish()
    }
}

impl NamedIcon {
    /// Create a new named icon with both a path and a name hint
    #[must_use]
    pub fn new(path_hint: &str, name_hint: &str, image: OutputImage) -> Self {
        Self {
            path_hint: Some(path_hint.to_string()),
            name_hint: Some(name_hint.to_string()),
            image,
        }
    }

    /// Create a new named icon from an icon without a path or name hint
    #[must_use]
    pub fn from_icon(icon: Icon) -> Self {
        Self {
            path_hint: None,
            name_hint: None,
            image: OutputImage::Dmi(icon),
        }
    }

    /// Assemble what the final relative path of the image should be
    #[must_use]
    #[tracing::instrument]
    pub fn build_path(&self, input_file: &Path) -> PathBuf {
        debug!(input_file = ?input_file, "Building path");
        let file_name = input_file
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut path = PathBuf::new();
        if let Some(path_hint) = &self.path_hint {
            path.push(format!("{file_name}-{path_hint}"));
        }
        if let Some(name_hint) = &self.name_hint {
            let result_name = format!("{file_name}-{name_hint}");
            debug!(result_name = ?result_name, "has name hint");
            path.push(result_name);
        } else {
            path.push(file_name);
        }
        path.set_extension(self.image.extension());
        debug!(path = ?path, "Built path");
        path
    }
}

#[derive(Clone)]
pub enum OutputImage {
    Png(DynamicImage),
    Dmi(Icon),
}

impl OutputImage {
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            OutputImage::Png(_) => "png",
            OutputImage::Dmi(_) => "dmi",
        }
    }
}

#[derive(Clone)]
pub enum ProcessorPayload {
    Single(Box<OutputImage>),
    SingleNamed(Box<NamedIcon>),
    MultipleNamed(Vec<NamedIcon>),
}

impl ProcessorPayload {
    #[must_use]
    pub fn from_icon(icon: Icon) -> Self {
        Self::Single(Box::new(OutputImage::Dmi(icon)))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum OperationMode {
    Standard,
    Debug,
}

/// Implement this trait to create a new type of icon operation
#[enum_dispatch]
pub trait IconOperationConfig {
    /// Represents performing an icon operation as defined by the implementor
    /// Should generally not be called directly, preferring to call via
    /// `do_operation` # Errors
    /// Possible errors vary based on implementor; should be some kind of
    /// `ProcessorError::ImageError`
    fn perform_operation(
        &self,
        input: &InputIcon,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload>;

    /// Verifies that current config values are valid within the context of the
    /// operation to be performed # Errors
    /// Possible errors vary based on implementor; should be some kind of
    /// `ProcessorError::InvalidConfig`
    fn verify_config(&self) -> ProcessorResult<()>;

    /// Helper function to call `verify_config` and `perform_operation` in
    /// sequence.
    ///
    /// This is what should be used in most cases, with trait implementations
    /// not needing to override this.
    /// # Errors
    /// Possible errors vary based on implementor
    /// Error type is potentially a `ProcessorError::InvalidConfig` from a call
    /// to `verify_config`, or a processor error from a call to
    /// `perform_operation`
    fn do_operation(
        &self,
        input: &InputIcon,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload> {
        self.verify_config()?;
        self.perform_operation(input, mode)
    }
}

#[enum_dispatch(IconOperationConfig)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "mode")]
pub enum IconOperation {
    BitmaskSlice,
    BitmaskDirectionalVis,
    BitmaskWindows,
}
