use std::fmt::Debug;
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};

use cutters::bitmask_dir_visibility::BitmaskDirectionalVis;
use cutters::bitmask_slice::BitmaskSlice;
use cutters::bitmask_windows::BitmaskWindows;
use dmi::error::DmiError;
use dmi::icon::Icon;
use enum_dispatch::enum_dispatch;
use format_converter::bitmask_to_precut::BitmaskSliceReconstruct;
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

/// An output image, with a possible path hint and name hint.
#[derive(Clone)]
pub struct NamedIcon {
    /// A hint for the path of the resulting image, if it has one
    /// This is used to determine the relative path of the output, in relation
    /// to the input file.
    ///
    /// If the input file is `foo/bar.png`, and the path hint is `baz`, the
    /// output file will be `foo/baz/bar.png` (or `foo/baz/bar.dmi` if the
    /// output format is dmi)
    pub path_hint: Option<String>,
    /// A hint for the name of the resulting image. This is appended to the
    /// input file name before the extension.
    ///
    /// If the input file is `foo/bar.png`, and the name hint is `baz`, the
    /// output file will be `foo/bar-baz.png` (or `foo/bar-baz.dmi` if the
    /// output format is dmi)
    pub name_hint: Option<String>,
    /// The actual output image
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

/// Represents the possible actual outputs of an icon operation
#[derive(Clone)]
pub enum Output {
    Image(OutputImage),
    Text(OutputText),
}

impl Output {
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Output::Image(image) => image.extension(),
            Output::Text(text) => text.extension(),
        }
    }
}

/// Represents the possible actual output images of an icon operation
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

/// Represents the possible text outputs of an icon operation
#[derive(Clone)]
pub enum OutputText {
    PngConfig(String),
    DmiConfig(String),
}

impl OutputText {
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            OutputText::PngConfig(_) => "png.toml",
            OutputText::DmiConfig(_) => "dmi.toml",
        }
    }
}

/// Represents the result of an icon operation
/// It's entirely up to consumers to decide what to do with this
#[derive(Clone)]
pub enum ProcessorPayload {
    /// A single icon, with no name or path hint.
    /// This is the most common result, and generally is used to create a dmi
    /// from a png
    Single(Box<OutputImage>),
    /// A single icon, with a name and path hint. See [NamedIcon] for more info.
    SingleNamed(Box<NamedIcon>),
    /// Multiple named icons. See [NamedIcon] for more info.
    MultipleNamed(Vec<NamedIcon>),
    /// Payload of some sort with a config to produce inline with it
    ConfigWrapped(Box<ProcessorPayload>, Box<OutputText>),
}

impl ProcessorPayload {
    #[must_use]
    pub fn from_icon(icon: Icon) -> Self {
        Self::Single(Box::new(OutputImage::Dmi(icon)))
    }

    #[must_use]
    pub fn from_image(image: DynamicImage) -> Self {
        Self::Single(Box::new(OutputImage::Png(image)))
    }

    #[must_use]
    pub fn wrap_png_config(payload: ProcessorPayload, text: String) -> Self {
        Self::ConfigWrapped(Box::new(payload), Box::new(OutputText::PngConfig(text)))
    }

    #[must_use]
    pub fn wrap_dmi_config(payload: ProcessorPayload, text: String) -> Self {
        Self::ConfigWrapped(Box::new(payload), Box::new(OutputText::DmiConfig(text)))
    }
}

/// Possible generic modes of operation for an icon operation
/// What these actually do is entirely up to the implementor
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum OperationMode {
    Standard,
    Debug,
}

/// Implement this trait to create a new type of icon operation
///
/// Once implemented, it can be used in a processor by adding it to the
/// `IconOperation` enum.
#[enum_dispatch]
pub trait IconOperationConfig {
    /// Represents performing an icon operation as defined by the implementor
    /// Should generally not be called directly, preferring to call via
    /// `do_operation`
    ///
    /// # Errors
    ///
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
    BitmaskSliceReconstruct,
}
