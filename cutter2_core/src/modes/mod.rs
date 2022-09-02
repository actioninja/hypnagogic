use std::io::{BufRead, Seek};
use enum_dispatch::enum_dispatch;
use dmi::icon::Icon;
use anyhow::Result;
use crate::modes::bitmask_slice::BitmaskSlice;
use serde::{Deserialize, Serialize};

pub mod bitmask_slice;

#[enum_dispatch]
pub trait CutterModeConfig {
    fn perform_operation<R: BufRead + Seek>(&self, input: &mut R) -> Result<Vec<(String, Icon)>>;
}

#[enum_dispatch(CutterModeConfig)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum CutterMode {
    BitmaskSlice,
}

