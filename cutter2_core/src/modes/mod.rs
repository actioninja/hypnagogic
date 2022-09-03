use cutters::bitmask_slice::BitmaskSlice;
use anyhow::Result;
use dmi::icon::Icon;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Seek};

pub mod cutters;

#[enum_dispatch]
pub trait CutterModeConfig {
    fn perform_operation<R: BufRead + Seek>(&self, input: &mut R) -> Result<Vec<(String, Icon)>>;
}

#[enum_dispatch(CutterModeConfig)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum CutterMode {
    BitmaskSlice,
}
