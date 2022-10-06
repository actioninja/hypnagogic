use std::io::{BufRead, Seek};
use std::path::PathBuf;

use dmi::icon::Icon;
use enum_dispatch::enum_dispatch;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use cutters::bitmask_dir_visibility::BitmaskDirectionalVis;
use cutters::bitmask_slice::BitmaskSlice;
use cutters::bitmask_windows::BitmaskWindows;

use crate::modes::error::ProcessorResult;

pub mod cutters;
pub mod error;
pub mod format_converter;

/// Implement this trait to create a new type of icon operation
#[enum_dispatch]
pub trait CutterModeConfig {
    /// Represents performing an icon operation as defined by the implementor
    /// # Errors
    /// Possible errors vary based on implementor
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
    ) -> ProcessorResult<Vec<(Option<String>, Icon)>>;

    /// Represents performing debug output as defined by implementor
    /// # Errors
    /// Possible errors vary based on implementor
    fn debug_output<R: BufRead + Seek>(
        &self,
        input: &mut R,
        output_dir: PathBuf,
    ) -> ProcessorResult<DynamicImage>;
}

#[enum_dispatch(CutterModeConfig)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum CutterMode {
    BitmaskSlice,
    BitmaskDirectionalVis,
    BitmaskWindows,
}
