use anyhow::Result;
use cutters::bitmask_dir_visibility::BitmaskDirectionalVis;
use cutters::bitmask_slice::BitmaskSlice;
use dmi::icon::Icon;
use enum_dispatch::enum_dispatch;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Seek};

pub mod cutters;
pub mod format_converter;

#[enum_dispatch]
pub trait CutterModeConfig {
    /// Represents performing an icon operation as defined by the implementor
    /// # Errors
    /// Possible errors vary based on implementor
    fn perform_operation<R: BufRead + Seek>(&self, input: &mut R) -> Result<Vec<(String, Icon)>>;

    /// Represents performing debug output as defined by implementor
    /// # Errors
    /// Possible errors vary based on implementor
    fn debug_output<R: BufRead + Seek>(&self, input: &mut R) -> Result<DynamicImage>;
}

#[enum_dispatch(CutterModeConfig)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum CutterMode {
    BitmaskSlice,
    BitmaskDirectionalVis,
}
