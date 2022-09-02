use crate::adjacency::Adjacency;
use anyhow::Result;
use dmi::icon::Icon;
use enum_iterator::all;
use fixed_map::Map;
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use serde_with::with_prefix;
use shrinkwraprs::Shrinkwrap;
use std::collections::HashMap;
use std::io::{BufRead, Read, Seek};

use crate::config::Sides;
use crate::corners::{Corner, CornerData, CornerType, Side};
use crate::modes::CutterModeConfig;

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, Shrinkwrap)]
pub struct CornerConfig(Map<CornerType, u32>);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SideSpacing {
    pub start: u32,
    pub end: u32,
    pub output_start: u32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BitmaskSlice {
    pub output_name: Option<String>,

    pub icon_size_x: u32,
    pub icon_size_y: u32,

    pub output_icon_size_x: u32,
    pub output_icon_size_y: u32,

    pub positions: CornerConfig,

    pub sides: Map<Side, SideSpacing>,

    pub delay: Option<Vec<f32>>,

    pub produce_dirs: bool,

    pub prefabs: Option<HashMap<Adjacency, u32>>,
    pub prefabs_overlays: Option<HashMap<Adjacency, Vec<u32>>>,

    pub is_diagonal: bool,

    pub dmi_version: Option<String>,
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
