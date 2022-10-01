use std::io::{BufRead, Seek};
use std::path::PathBuf;

use dmi::icon::{Icon, IconState};
use enum_iterator::all;
use fixed_map::Map;
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};

use crate::modes::cutters::bitmask_slice::{
    BitmaskSlice, SideSpacing, SIZE_OF_CARDINALS, SIZE_OF_DIAGONALS,
};
use crate::modes::error::ProcessorResult;
use crate::modes::CutterModeConfig;
use crate::util::corners::{Corner, CornerType, Side};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BitmaskDirectionalVis {
    #[serde(flatten)]
    pub bitmask_slice_config: BitmaskSlice,
    pub mask_color: Option<String>,
    pub slice_point: Map<Side, u32>,
}

impl CutterModeConfig for BitmaskDirectionalVis {
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
    ) -> ProcessorResult<Vec<(String, Icon)>> {
        let mut img = image::load(input, ImageFormat::Png)?;
        let (corners, prefabs) = self.bitmask_slice_config.generate_corners(&mut img)?;

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.bitmask_slice_config.icon_size_y;

        let possible_states = if self.bitmask_slice_config.is_diagonal {
            SIZE_OF_DIAGONALS
        } else {
            SIZE_OF_CARDINALS
        };

        let assembled = self.bitmask_slice_config.generate_icons(
            &corners,
            &prefabs,
            num_frames,
            possible_states,
        );

        // fairly gnarly iterator chain; loops delay sequence and then takes number of frames
        let delay: Option<Vec<f32>> = self.bitmask_slice_config.delay.clone().map(|inner| {
            inner
                .iter()
                .cycle()
                .take(num_frames as usize)
                .copied()
                .collect()
        });

        let mut icon_states = vec![];

        for (adjacency, images) in assembled {
            for side in Side::dmi_cardinals() {
                let mut icon_state_frames = vec![];
                let slice_info = self.get_side_cuts(side);

                let (x, y, width, height) = if side.is_vertical() {
                    (
                        0,
                        slice_info.start,
                        self.bitmask_slice_config.icon_size_x,
                        slice_info.step(),
                    )
                } else {
                    (
                        slice_info.start,
                        0,
                        slice_info.step(),
                        self.bitmask_slice_config.icon_size_y,
                    )
                };

                for image in &images {
                    let mut cut_img = DynamicImage::new_rgba8(
                        self.bitmask_slice_config.icon_size_x,
                        self.bitmask_slice_config.icon_size_y,
                    );

                    let crop = image.crop_imm(x, y, width, height);

                    imageops::overlay(&mut cut_img, &crop, x as i64, y as i64);
                    icon_state_frames.push(cut_img);
                }
                icon_states.push(IconState {
                    name: format!("{}-{}", adjacency.bits(), side.byond_dir()),

                    dirs: 1,
                    frames: num_frames,
                    images: icon_state_frames,
                    delay: delay.clone(),
                    ..Default::default()
                });
            }

            for corner in all::<Corner>() {
                let mut icon_state_frames = vec![];

                let corner_images = corners
                    .get(corner)
                    .unwrap()
                    .get(CornerType::Concave)
                    .unwrap();
                for image in corner_images {
                    let mut cut_img = DynamicImage::new_rgba8(
                        self.bitmask_slice_config.icon_size_x,
                        self.bitmask_slice_config.icon_size_y,
                    );

                    let (horizontal, vertical) = corner.sides_of_corner();
                    let horizontal = self.bitmask_slice_config.get_side_info(horizontal).start;
                    let vertical = self.bitmask_slice_config.get_side_info(vertical).start;

                    imageops::overlay(&mut cut_img, image, horizontal as i64, vertical as i64);
                    icon_state_frames.push(cut_img);
                }

                icon_states.push(IconState {
                    name: format!("{}-{}", adjacency.bits(), corner.byond_dir()),
                    dirs: 1,
                    frames: num_frames,
                    images: icon_state_frames,
                    delay: delay.clone(),

                    ..Default::default()
                });
            }
        }

        let out_icon = Icon {
            version: Default::default(),
            width: self.bitmask_slice_config.output_icon_size_x,
            height: self.bitmask_slice_config.output_icon_size_y,
            states: icon_states,
        };
        Ok(vec![("".to_string(), out_icon)])
    }

    fn debug_output<R: BufRead + Seek>(
        &self,
        input: &mut R,
        output_dir: PathBuf,
    ) -> ProcessorResult<DynamicImage> {
        self.bitmask_slice_config.debug_output(input, output_dir)
    }
}

impl BitmaskDirectionalVis {
    /// Gets the side cutter info for a given side based on the slice point
    /// # Panics
    /// Can panic if the `slice_point` map is unpopulated, which shouldn't happen if initialized correctly
    #[must_use]
    pub fn get_side_cuts(&self, side: Side) -> SideSpacing {
        match side {
            Side::North => SideSpacing {
                start: 0,
                end: *self.slice_point.get(Side::North).unwrap(),
            },
            Side::South => SideSpacing {
                start: *self.slice_point.get(Side::South).unwrap(),
                end: self.bitmask_slice_config.icon_size_y,
            },
            Side::East => SideSpacing {
                start: *self.slice_point.get(Side::East).unwrap(),
                end: self.bitmask_slice_config.icon_size_x,
            },
            Side::West => SideSpacing {
                start: 0,
                end: *self.slice_point.get(Side::West).unwrap(),
            },
        }
    }
}
