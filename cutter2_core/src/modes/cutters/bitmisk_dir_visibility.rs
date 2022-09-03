use crate::modes::cutters::bitmask_slice::BitmaskSlice;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BitmaskDirectionalVis {
    #[serde(flatten)]
    pub bitmask_slice_config: BitmaskSlice,
}
