use crate::modes::cutters::bitmask_slice::{BitmaskSlice, CornerConfig, SIZE_OF_DIAGONALS};
use crate::modes::cutters::delay_repeat;
use crate::modes::error::ProcessorResult;
use crate::modes::CutterModeConfig;
use crate::util::adjacency::Adjacency;
use crate::util::corners::CornerType;
use dmi::icon::{Icon, IconState};

use fixed_map::Map;
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Seek};

use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BitmaskWindows {
    pub icon_size_x: u32,
    pub icon_size_y: u32,
    pub output_icon_pos_x: u32,
    pub output_icon_pos_y: u32,
    pub output_icon_size_x: u32,
    pub output_icon_size_y: u32,
    pub delay: Option<Vec<f32>>,
}

impl CutterModeConfig for BitmaskWindows {
    #[tracing::instrument(skip(input))]
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
    ) -> ProcessorResult<Vec<(Option<String>, Icon)>> {
        let mut img = image::load(input, ImageFormat::Png)?;

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.icon_size_y;

        let mut positions = Map::new();
        positions.insert(CornerType::Convex, 0);
        positions.insert(CornerType::Concave, 1);
        positions.insert(CornerType::Horizontal, 2);
        positions.insert(CornerType::Vertical, 3);
        positions.insert(CornerType::Flat, 4);

        let bitmask_config = BitmaskSlice {
            output_name: None,
            icon_size_x: self.icon_size_x,
            icon_size_y: self.icon_size_y,
            output_icon_pos_x: self.output_icon_pos_x,
            output_icon_pos_y: self.output_icon_pos_y,
            output_icon_size_x: self.icon_size_x,
            output_icon_size_y: self.icon_size_y,
            positions: CornerConfig(positions),
            cut_position_x: self.icon_size_x / 2,
            cut_position_y: self.icon_size_y / 2,
            delay: self.delay.clone(),
            produce_dirs: false,
            prefabs: None,
            prefabs_overlays: None,
            is_diagonal: true,
            dmi_version: None,
        };

        let (corners, prefabs) = bitmask_config.generate_corners(&mut img)?;
        let assembled =
            bitmask_config.generate_icons(&corners, &prefabs, num_frames, SIZE_OF_DIAGONALS);

        let mut alt_config = bitmask_config.clone();

        let mut positions = Map::new();
        positions.insert(CornerType::Convex, 5);
        positions.insert(CornerType::Concave, 6);
        positions.insert(CornerType::Horizontal, 7);
        positions.insert(CornerType::Vertical, 8);
        positions.insert(CornerType::Flat, 9);

        alt_config.positions = CornerConfig(positions);

        let (corners_alt, prefabs_alt) = alt_config.generate_corners(&mut img)?;
        let assembled_alt =
            alt_config.generate_icons(&corners_alt, &prefabs_alt, num_frames, SIZE_OF_DIAGONALS);

        let delay = delay_repeat(&self.delay, num_frames as usize);

        let mut states = vec![];
        for signature in 0..SIZE_OF_DIAGONALS {
            let adjacency = Adjacency::from_bits(signature as u8).unwrap();

            let mut states_from_assembled = |prefix: &str,
                                             assembled_set: &HashMap<
                Adjacency,
                Vec<DynamicImage>,
            >| {
                let mut upper_frames = vec![];
                let mut lower_frames = vec![];
                for frame in 0..num_frames {
                    let uncut_img = assembled_set
                        .get(&adjacency)
                        .unwrap()
                        .get(frame as usize)
                        .unwrap();

                    let upper_img =
                        uncut_img.crop_imm(0, 0, self.output_icon_size_x, self.output_icon_size_y);
                    upper_frames.push(upper_img);
                    let lower_img = uncut_img.crop_imm(
                        0,
                        self.icon_size_y / 2,
                        self.output_icon_size_x,
                        self.output_icon_size_y,
                    );
                    lower_frames.push(lower_img);
                }

                states.push(IconState {
                    name: format!("{prefix}{signature}-upper"),
                    dirs: 1,
                    frames: num_frames,
                    images: upper_frames,
                    delay: delay.clone(),
                    ..Default::default()
                });
                states.push(IconState {
                    name: format!("{prefix}{signature}-lower"),
                    dirs: 1,
                    frames: num_frames,
                    images: lower_frames,
                    delay: delay.clone(),
                    ..Default::default()
                });
            };
            states_from_assembled("", &assembled);
            states_from_assembled("alt-", &assembled_alt);
        }

        let icon = Icon {
            width: self.output_icon_size_x,
            height: self.output_icon_size_y,
            states,
            ..Default::default()
        };

        Ok(vec![(None, icon)])
    }

    fn debug_output<R: BufRead + Seek>(
        &self,
        _input: &mut R,
        _output_dir: PathBuf,
    ) -> ProcessorResult<DynamicImage> {
        Ok(DynamicImage::new_rgb8(1, 1))
    }
}
