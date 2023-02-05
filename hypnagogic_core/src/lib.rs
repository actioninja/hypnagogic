#![warn(clippy::pedantic, clippy::cargo)]
// too many lines is a dumb metric
#![allow(clippy::too_many_lines)]
// as is fine, clippy is silly
#![allow(clippy::cast_lossless)]
// Not actually going to be a published crate, useless to add
#![allow(clippy::cargo_common_metadata)]
// Annoying
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod modes;
pub mod util;
