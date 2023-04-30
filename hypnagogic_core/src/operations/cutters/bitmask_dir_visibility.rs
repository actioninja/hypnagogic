use std::io::{BufRead, Seek};

use crate::config::blocks::cutters::SlicePoint;
use crate::generation::icon::generate_map_icon;
use dmi::icon::{Icon, IconState};
use enum_iterator::all;
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};

use crate::operations::cutters::bitmask_slice::{
    BitmaskSlice, SideSpacing, SIZE_OF_CARDINALS, SIZE_OF_DIAGONALS,
};
use crate::operations::error::ProcessorResult;
use crate::operations::{IconOperationConfig, NamedIcon, OperationMode, ProcessorPayload};
use crate::util::adjacency::Adjacency;
use crate::util::corners::{Corner, Side};
use crate::util::repeat_for;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BitmaskDirectionalVis {
    #[serde(flatten)]
    pub bitmask_slice_config: BitmaskSlice,
    pub slice_point: SlicePoint,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mask_color: Option<String>,
}

impl IconOperationConfig for BitmaskDirectionalVis {
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload> {
        let mut img = image::load(input, ImageFormat::Png)?;
        let (corners, prefabs) = self.bitmask_slice_config.generate_corners(&mut img)?;

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.bitmask_slice_config.icon_size.y;

        let possible_states = if self.bitmask_slice_config.smooth_diagonally {
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

        let delay = self
            .bitmask_slice_config
            .animation
            .clone()
            .map(|x| repeat_for(&x.delays, num_frames as usize));

        let mut icon_states = vec![];

        for (adjacency, images) in &assembled {
            for side in Side::dmi_cardinals() {
                let mut icon_state_frames = vec![];
                let slice_info = self.get_side_cuts(side);

                let (x, y, width, height) = if side.is_vertical() {
                    (
                        0,
                        slice_info.start,
                        self.bitmask_slice_config.icon_size.x,
                        slice_info.step(),
                    )
                } else {
                    (
                        slice_info.start,
                        0,
                        slice_info.step(),
                        self.bitmask_slice_config.icon_size.y,
                    )
                };

                for image in images {
                    let mut cut_img = DynamicImage::new_rgba8(
                        self.bitmask_slice_config.icon_size.x,
                        self.bitmask_slice_config.icon_size.y,
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
        }

        let convex_images = assembled.get(&Adjacency::CARDINALS).unwrap();
        for corner in all::<Corner>() {
            let mut icon_state_frames = vec![];

            let (horizontal, vertical) = corner.sides_of_corner();

            let horizontal_side_info = self.bitmask_slice_config.get_side_info(horizontal);
            let x = horizontal_side_info.start;
            let width = horizontal_side_info.step();

            //todo: This is awful, maybe a better way to do this?
            let (y, height) = if vertical == Side::North {
                (0, self.slice_point.get(vertical).unwrap())
            } else {
                let slice_point = self.slice_point.get(vertical).unwrap();
                let end = self.bitmask_slice_config.icon_size.y;
                (slice_point, end - slice_point)
            };

            for image in convex_images {
                let mut cut_img = DynamicImage::new_rgba8(
                    self.bitmask_slice_config.icon_size.x,
                    self.bitmask_slice_config.icon_size.y,
                );

                let crop_img = image.crop_imm(x, y, width, height);

                imageops::overlay(&mut cut_img, &crop_img, x as i64, y as i64);
                icon_state_frames.push(cut_img);
            }

            icon_states.push(IconState {
                name: format!("innercorner-{}", corner.byond_dir()),
                dirs: 1,
                frames: num_frames,
                images: icon_state_frames,
                delay: delay.clone(),

                ..Default::default()
            });
        }

        if let Some(map_icon) = &self.bitmask_slice_config.map_icon {
            let icon = generate_map_icon(
                self.bitmask_slice_config.output_icon_size.x,
                self.bitmask_slice_config.output_icon_size.y,
                map_icon,
            )
            .unwrap();
            icon_states.push(IconState {
                name: map_icon.icon_state_name.clone(),
                dirs: 1,
                frames: 1,
                images: vec![icon],
                ..Default::default()
            });
        }

        let out_icon = Icon {
            version: dmi::icon::DmiVersion::default(),
            width: self.bitmask_slice_config.output_icon_size.x,
            height: self.bitmask_slice_config.output_icon_size.y,
            states: icon_states,
        };

        if mode == OperationMode::Debug {
            let mut out = self.bitmask_slice_config.generate_debug_icons(&corners);

            out.push(NamedIcon::from_icon(out_icon));
            Ok(ProcessorPayload::MultipleNamed(out))
        } else {
            Ok(ProcessorPayload::from_icon(out_icon))
        }
    }

    fn verify_config(&self) -> ProcessorResult<()> {
        //TODO: actually verify config
        Ok(())
    }
}

impl BitmaskDirectionalVis {
    /// Gets the side cutter info for a given side based on the slice point
    /// # Panics
    /// Can panic if the `slice_point` map is unpopulated, which shouldn't happen if initialized correctly
    /// Generally indicates a bad implementation of `BitmaskDirectionalVis`
    #[must_use]
    pub fn get_side_cuts(&self, side: Side) -> SideSpacing {
        match side {
            Side::North => SideSpacing {
                start: 0,
                end: self.slice_point.get(Side::North).unwrap(),
            },
            Side::South => SideSpacing {
                start: self.slice_point.get(Side::South).unwrap(),
                end: self.bitmask_slice_config.icon_size.y,
            },
            Side::East => SideSpacing {
                start: self.slice_point.get(Side::East).unwrap(),
                end: self.bitmask_slice_config.icon_size.x,
            },
            Side::West => SideSpacing {
                start: 0,
                end: self.slice_point.get(Side::West).unwrap(),
            },
        }
    }
}
