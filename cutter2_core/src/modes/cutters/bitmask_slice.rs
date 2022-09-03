use crate::util::adjacency::Adjacency;
use anyhow::Result;
use dmi::icon::Icon;
use enum_iterator::all;
use fixed_map::Map;
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use std::collections::HashMap;
use std::io::{BufRead, Read, Seek};

use crate::util::corners::{Corner, CornerData, CornerType, Side};
use crate::modes::CutterModeConfig;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Shrinkwrap)]
#[serde(transparent)]
pub struct CornerConfig(Map<CornerType, u32>);

impl Default for CornerConfig {
    fn default() -> Self {
        let mut out = Map::new();

        out.insert(CornerType::Convex, 0);
        out.insert(CornerType::Concave, 1);
        out.insert(CornerType::Horizontal, 2);
        out.insert(CornerType::Vertical, 3);

        CornerConfig(out)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Shrinkwrap)]
#[serde(transparent)]
pub struct SideConfig(Map<Side, SideSpacing>);

impl Default for SideConfig {
    fn default() -> Self {
        let mut out = Map::new();

        out.insert(
            Side::North,
            SideSpacing {
                start: 0,
                end: 16,
                output_start: 0,
            },
        );
        out.insert(
            Side::West,
            SideSpacing {
                start: 0,
                end: 0,
                output_start: 0,
            },
        );
        out.insert(
            Side::East,
            SideSpacing {
                start: 16,
                end: 32,
                output_start: 16,
            },
        );
        out.insert(
            Side::South,
            SideSpacing {
                start: 16,
                end: 32,
                output_start: 16,
            },
        );

        SideConfig(out)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SideSpacing {
    pub start: u32,
    pub end: u32,
    pub output_start: u32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BitmaskSlice {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub output_name: Option<String>,

    pub icon_size_x: u32,
    pub icon_size_y: u32,

    pub output_icon_size_x: u32,
    pub output_icon_size_y: u32,

    pub positions: CornerConfig,

    pub sides: SideConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub delay: Option<Vec<f32>>,

    pub produce_dirs: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prefabs: Option<HashMap<Adjacency, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prefabs_overlays: Option<HashMap<Adjacency, Vec<u32>>>,

    pub is_diagonal: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub dmi_version: Option<String>,
}

impl Default for BitmaskSlice {
    fn default() -> Self {
        BitmaskSlice {
            output_name: None,
            icon_size_x: 32,
            icon_size_y: 32,

            output_icon_size_x: 32,
            output_icon_size_y: 32,

            positions: CornerConfig::default(),

            sides: SideConfig::default(),

            delay: None,

            produce_dirs: false,

            prefabs: None,
            prefabs_overlays: None,

            is_diagonal: false,

            dmi_version: None,
        }
    }
}

impl CutterModeConfig for BitmaskSlice {
    fn perform_operation<R: BufRead + Seek>(&self, input: &mut R) -> Result<Vec<(String, Icon)>> {
        let (corners, prefabs) = self.generate_corners(input)?;
        todo!()
    }
}

type Corners = Map<Corner, Map<CornerType, Vec<DynamicImage>>>;
type Prefabs = HashMap<Adjacency, Vec<DynamicImage>>;

//possible icon set is the powerset of the possible directions
//the size of a powerset is always 2^n where n is number of discrete elements
const SIZE_OF_CARDINALS: usize = usize::pow(2, 4);
const SIZE_OF_DIAGONALS: usize = usize::pow(2, 8);

impl BitmaskSlice {
    fn get_dir_step(&self, side: Side) -> u32 {
        let side_info = self.sides.get(side).unwrap();
        side_info.end - side_info.start
    }

    /// Generates corners
    /// # Errors
    /// Errors on malformed image
    /// # Panics
    /// Shouldn't panic
    pub fn generate_corners<R: BufRead + Seek>(&self, input: &mut R) -> Result<(Corners, Prefabs)> {
        let img = image::load(input, ImageFormat::Png)?;

        let (_width, height) = img.dimensions();

        let num_frames = height / self.icon_size_y;

        let corner_types = if self.is_diagonal {
            CornerType::diagonal()
        } else {
            CornerType::cardinal()
        };

        let mut corners: Corners = Map::new();
        for corner in all::<Corner>() {
            corners.insert(corner, Map::new());
            for corner_type in &corner_types[..] {
                let dir_map = corners.get_mut(corner).unwrap();
                dir_map.insert(*corner_type, vec![]);
                for frame_num in 0..num_frames {
                    let frame_vec = dir_map.get_mut(*corner_type).unwrap();

                    let position = self.positions.get(*corner_type).unwrap();

                    let (x_side, y_side) = corner.sides_of_corner();

                    let x_offset = self.sides.get(x_side).unwrap().start;
                    let y_offset = self.sides.get(y_side).unwrap().start;

                    let x = (position * self.icon_size_x) + x_offset;
                    let y = (frame_num * self.icon_size_y) + y_offset;

                    let width = self.get_dir_step(x_side);
                    let height = self.get_dir_step(y_side);
                    let corner_img = img.crop_imm(x, y, width, height);
                    frame_vec.push(corner_img);
                }
            }
        }

        let mut prefabs: Prefabs = HashMap::new();

        if let Some(prefabs_config) = &self.prefabs {
            for (adjacency_bits, position) in prefabs_config {
                let mut frame_vector = vec![];
                for frame in 0..num_frames {
                    let x = position * self.icon_size_x;
                    let y = frame * self.icon_size_y;
                    let img = img.crop_imm(x, y, self.icon_size_x, self.icon_size_y);

                    frame_vector.push(img);
                }
                prefabs.insert(*adjacency_bits, frame_vector);
            }
        }

        Ok((corners, prefabs))
    }
}