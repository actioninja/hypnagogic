use crate::operations::cutters::bitmask_slice::{BitmaskSlice, SIZE_OF_DIAGONALS};
use crate::operations::error::{ProcessorError, ProcessorResult};
use crate::operations::{IconOperationConfig, InputIcon, OperationMode, ProcessorPayload};
use crate::util::adjacency::Adjacency;
use crate::util::corners::CornerType;
use crate::util::icon_ops::dedupe_frames;
use crate::util::repeat_for;
use dmi::icon::{Icon, IconState};

use fixed_map::Map;
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{BufRead, Seek};

use crate::config::blocks::cutters::{
    Animation, CutPosition, IconSize, OutputIconPosition, OutputIconSize, Positions,
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BitmaskWindows {
    pub icon_size: IconSize,
    pub output_icon_pos: OutputIconPosition,
    pub output_icon_size: OutputIconSize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub animation: Option<Animation>,
}

impl IconOperationConfig for BitmaskWindows {
    #[tracing::instrument(skip(input))]
    fn perform_operation(
        &self,
        input: &InputIcon,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload> {
        let InputIcon::DynamicImage(img) = input else {
            return Err(ProcessorError::FormatError("This operation only accepts raw images".to_string()));
        };

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.icon_size.y;

        let mut positions = Positions::default();
        positions.0.insert(CornerType::Flat, 4);

        let bitmask_config = BitmaskSlice {
            output_name: None,
            icon_size: self.icon_size,
            output_icon_pos: self.output_icon_pos,
            output_icon_size: OutputIconSize {
                x: self.icon_size.x,
                y: self.icon_size.y,
            },
            positions,
            cut_pos: CutPosition {
                x: self.icon_size.x / 2,
                y: self.icon_size.y / 2,
            },
            animation: self.animation.clone(),
            produce_dirs: false,
            prefabs: None,
            prefab_overlays: None,
            smooth_diagonally: true,
            map_icon: None,
        };

        let (corners, prefabs) = bitmask_config.generate_corners(&img)?;
        let assembled =
            bitmask_config.generate_icons(&corners, &prefabs, num_frames, SIZE_OF_DIAGONALS);

        let mut alt_config = bitmask_config.clone();

        let mut positions = Map::new();
        positions.insert(CornerType::Convex, 5);
        positions.insert(CornerType::Concave, 6);
        positions.insert(CornerType::Horizontal, 7);
        positions.insert(CornerType::Vertical, 8);
        positions.insert(CornerType::Flat, 9);

        alt_config.positions = Positions(positions);

        let (corners_alt, prefabs_alt) = alt_config.generate_corners(&img)?;
        let assembled_alt =
            alt_config.generate_icons(&corners_alt, &prefabs_alt, num_frames, SIZE_OF_DIAGONALS);

        let delay = self
            .animation
            .clone()
            .map(|x| repeat_for(&x.delays, num_frames as usize));

        let mut states = vec![];

        let states_to_gen = (0..SIZE_OF_DIAGONALS)
            .map(|x| Adjacency::from_bits(x as u8).unwrap())
            .filter(Adjacency::ref_has_no_orphaned_corner);
        for adjacency in states_to_gen {
            let mut states_from_assembled = |prefix: &str,
                                             assembled_set: &BTreeMap<
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
                        uncut_img.crop_imm(0, 0, self.output_icon_size.x, self.output_icon_size.y);
                    upper_frames.push(upper_img);
                    let lower_img = uncut_img.crop_imm(
                        0,
                        self.icon_size.y / 2,
                        self.output_icon_size.x,
                        self.output_icon_size.y,
                    );
                    lower_frames.push(lower_img);
                }

                let signature = adjacency.bits();
                states.push(dedupe_frames(IconState {
                    name: format!("{prefix}{signature}-upper"),
                    dirs: 1,
                    frames: num_frames,
                    images: upper_frames,
                    delay: delay.clone(),
                    ..Default::default()
                }));
                states.push(dedupe_frames(IconState {
                    name: format!("{prefix}{signature}-lower"),
                    dirs: 1,
                    frames: num_frames,
                    images: lower_frames,
                    delay: delay.clone(),
                    ..Default::default()
                }));
            };
            states_from_assembled("", &assembled);
            states_from_assembled("alt-", &assembled_alt);
        }

        let icon = Icon {
            width: self.output_icon_size.x,
            height: self.output_icon_size.y,
            states,
            ..Default::default()
        };

        Ok(ProcessorPayload::from_icon(icon))
    }

    fn verify_config(&self) -> ProcessorResult<()> {
        //TODO: Actually verify config
        Ok(())
    }
}
