use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, Seek};

use crate::config::blocks::cutters::{
    Animation, CutPosition, IconSize, OutputIconPosition, OutputIconSize, Positions,
    PrefabOverlays, Prefabs,
};
use crate::config::blocks::generators::MapIcon;
use crate::generation::icon::generate_map_icon;
use dmi::icon::{Icon, IconState};
use enum_iterator::all;
use fixed_map::Map;
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::operations::error::ProcessorResult;
use crate::operations::{
    IconOperationConfig, NamedIcon, OperationMode, OutputImage, ProcessorPayload,
};
use crate::util::adjacency::Adjacency;
use crate::util::corners::{Corner, CornerType, Side};
use crate::util::repeat_for;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SideSpacing {
    pub start: u32,
    pub end: u32,
}

impl SideSpacing {
    #[must_use]
    pub fn step(self) -> u32 {
        self.end - self.start
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct BitmaskSlice {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub output_name: Option<String>,
    pub produce_dirs: bool,
    pub smooth_diagonally: bool,
    pub icon_size: IconSize,
    pub output_icon_pos: OutputIconPosition,
    pub output_icon_size: OutputIconSize,
    pub positions: Positions,
    pub cut_pos: CutPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prefabs: Option<Prefabs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prefab_overlays: Option<PrefabOverlays>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub map_icon: Option<MapIcon>,
}

impl IconOperationConfig for BitmaskSlice {
    #[tracing::instrument(skip(input))]
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload> {
        debug!("Starting bitmask slice icon op");
        let mut img = image::load(input, ImageFormat::Png)?;
        let (corners, prefabs) = self.generate_corners(&mut img)?;

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.icon_size.y;

        let possible_states = if self.smooth_diagonally {
            SIZE_OF_DIAGONALS
        } else {
            SIZE_OF_CARDINALS
        };

        let icon_directions = if self.produce_dirs {
            Adjacency::dmi_cardinals().to_vec()
        } else {
            vec![Adjacency::S]
        };

        //First phase: generate icons
        let assembled = self.generate_icons(&corners, &prefabs, num_frames, possible_states);

        // Second phase: map to byond icon states and produce dirs if need
        // Even though this is the same loop as what happens in generate_icons,
        // all states need to be generated first for the
        // Rotation to work correctly, so it must be done as a second loop.
        let mut icon_states = vec![];

        let delay = self
            .animation
            .clone()
            .map(|x| repeat_for(&x.delays, num_frames as usize));

        for signature in 0..possible_states {
            let mut icon_state_frames = vec![];

            let adjacency = Adjacency::from_bits(signature as u8).unwrap();

            for icon_state_dir in &icon_directions {
                let rotated_sig = adjacency.rotate_to(*icon_state_dir);
                trace!(sig = ?icon_state_dir, rotated_sig = ?rotated_sig, "Rotated");
                icon_state_frames.extend(assembled[&rotated_sig].clone());
            }

            let name = if let Some(prefix_name) = &self.output_name {
                format!("{prefix_name}-{signature}")
            } else {
                format!("{signature}")
            };
            icon_states.push(IconState {
                name,
                dirs: icon_directions.len() as u8,
                frames: num_frames,
                images: icon_state_frames,
                delay: delay.clone(),
                ..Default::default()
            });
        }

        if let Some(map_icon) = &self.map_icon {
            let icon =
                generate_map_icon(self.output_icon_size.x, self.output_icon_size.y, map_icon)
                    .unwrap();
            icon_states.push(IconState {
                name: map_icon.icon_state_name.clone(),
                dirs: 1,
                frames: 1,
                images: vec![icon],
                ..Default::default()
            });
        }

        let output_icon = Icon {
            version: dmi::icon::DmiVersion::default(),
            width: self.output_icon_size.x,
            height: self.output_icon_size.y,
            states: icon_states,
        };

        if mode == OperationMode::Debug {
            debug!("Starting debug output");
            let mut out = self.generate_debug_icons(&corners);

            out.push(NamedIcon::from_icon(output_icon));
            Ok(ProcessorPayload::MultipleNamed(out))
        } else {
            Ok(ProcessorPayload::from_icon(output_icon))
        }
    }

    fn verify_config(&self) -> ProcessorResult<()> {
        //TODO: Actual verification
        Ok(())
    }
}

type CornerPayload = Map<CornerType, Map<Corner, Vec<DynamicImage>>>;
type PrefabPayload = HashMap<Adjacency, Vec<DynamicImage>>;

//possible icon set is the powerset of the possible directions
//the size of a powerset is always 2^n where n is number of discrete elements
pub const SIZE_OF_CARDINALS: usize = usize::pow(2, 4);
pub const SIZE_OF_DIAGONALS: usize = usize::pow(2, 8);

impl BitmaskSlice {
    #[tracing::instrument(skip(img))]
    pub fn build_corner(
        &self,
        img: &DynamicImage,
        position: u32,
        num_frames: u32,
    ) -> Map<Corner, Vec<DynamicImage>> {
        let mut out = Map::new();

        for corner in all::<Corner>() {
            out.insert(corner, vec![]);
            for frame_num in 0..num_frames {
                let frame_vec = out.get_mut(corner).unwrap();

                let (x_side, y_side) = corner.sides_of_corner();

                let x_spacing = self.get_side_info(x_side);
                let y_spacing = self.get_side_info(y_side);
                let x_offset = x_spacing.start;
                let y_offset = y_spacing.start;

                let x = (position * self.icon_size.x) + x_offset;
                let y = (frame_num * self.icon_size.y) + y_offset;

                let width = x_spacing.step();
                let height = y_spacing.step();
                trace!(
                    corner = ?corner,
                    x = ?x,
                    y = ?y,
                    width = ?width,
                    height = ?height,
                    "Ready to generate image"
                );
                let corner_img = img.crop_imm(x, y, width, height);
                frame_vec.push(corner_img);
            }
        }
        out
    }

    /// Generates corners
    /// # Errors
    /// Errors on malformed image
    /// # Panics
    /// Shouldn't panic
    #[tracing::instrument(skip(img))]
    pub fn generate_corners(
        &self,
        img: &mut DynamicImage,
    ) -> ProcessorResult<(CornerPayload, PrefabPayload)> {
        let (_width, height) = img.dimensions();

        let num_frames = height / self.icon_size.y;

        let corner_types = if self.smooth_diagonally {
            CornerType::diagonal()
        } else {
            CornerType::cardinal()
        };

        let mut corner_map: CornerPayload = Map::new();

        for corner_type in &corner_types[..] {
            let position = self.positions.get(*corner_type).unwrap();

            let corners = self.build_corner(img, position, num_frames);

            corner_map.insert(*corner_type, corners);
        }

        let mut prefabs: PrefabPayload = HashMap::new();

        if let Some(prefabs_config) = &self.prefabs {
            for (adjacency_bits, position) in &prefabs_config.0 {
                let mut frame_vector = vec![];
                for frame in 0..num_frames {
                    let x = position * self.icon_size.x;
                    let y = frame * self.icon_size.y;
                    let img = img.crop_imm(x, y, self.icon_size.x, self.icon_size.y);

                    frame_vector.push(img);
                }
                prefabs.insert(Adjacency::from_bits(*adjacency_bits).unwrap(), frame_vector);
            }
        }

        Ok((corner_map, prefabs))
    }

    /// Blah
    /// # Panics
    /// Whatever
    #[must_use]
    pub fn generate_icons(
        &self,
        corners: &CornerPayload,
        prefabs: &PrefabPayload,
        num_frames: u32,
        possible_states: usize,
    ) -> BTreeMap<Adjacency, Vec<DynamicImage>> {
        let mut assembled: BTreeMap<Adjacency, Vec<DynamicImage>> = BTreeMap::new();
        for signature in 0..possible_states {
            let adjacency = Adjacency::from_bits(signature as u8).unwrap();
            let mut icon_state_images = vec![];
            for frame in 0..num_frames {
                if prefabs.contains_key(&adjacency) {
                    let mut frame_image =
                        DynamicImage::new_rgba8(self.output_icon_size.x, self.output_icon_size.y);
                    imageops::replace(
                        &mut frame_image,
                        prefabs
                            .get(&adjacency)
                            .unwrap()
                            .get(frame as usize)
                            .unwrap(),
                        self.output_icon_pos.x as i64,
                        self.output_icon_pos.y as i64,
                    );

                    icon_state_images.push(frame_image);
                } else {
                    let mut frame_image =
                        DynamicImage::new_rgba8(self.output_icon_size.x, self.output_icon_size.y);

                    for corner in all::<Corner>() {
                        let corner_type = adjacency.get_corner_type(corner);
                        let corner_img = &corners
                            .get(corner_type)
                            .unwrap()
                            .get(corner)
                            .unwrap()
                            .get(frame as usize)
                            .unwrap();

                        let (horizontal, vertical) = corner.sides_of_corner();
                        let horizontal = self.get_side_info(horizontal);
                        let vertical = self.get_side_info(vertical);

                        imageops::overlay(
                            &mut frame_image,
                            *corner_img,
                            horizontal.start as i64,
                            vertical.start as i64,
                        );
                    }
                    icon_state_images.push(frame_image);
                }
            }
            assembled.insert(adjacency, icon_state_images);
        }
        assembled
    }

    /// Generates debug outputs for bitmask slice
    /// # Panics
    /// Shouldn't panic, unless the passed in corners are malformed
    #[must_use]
    pub fn generate_debug_icons(&self, corners: &CornerPayload) -> Vec<NamedIcon> {
        let mut out = vec![];
        let mut corners_image =
            DynamicImage::new_rgba8(corners.len() as u32 * self.icon_size.x, self.icon_size.y);

        for (corner_type, map) in corners.iter() {
            let position = self.positions.get(corner_type).unwrap();
            for (corner, vec) in map.iter() {
                // output each corner as it's own file
                out.push(NamedIcon::new(
                    "DEBUGOUT/CORNERS/",
                    &format!("CORNER-{corner_type:?}-{corner:?}"),
                    OutputImage::Png(vec.get(0).unwrap().clone()),
                ));
                // Reassemble the input image from corners (minus prefabs and frames)
                let (horizontal, vertical) = corner.sides_of_corner();
                let horizontal = self.get_side_info(horizontal);
                let vertical = self.get_side_info(vertical);
                let frame = vec.get(0).unwrap();
                imageops::replace(
                    &mut corners_image,
                    frame,
                    ((position * self.icon_size.x) + horizontal.start) as i64,
                    vertical.start as i64,
                );
            }
        }
        out.push(NamedIcon::new(
            "DEBUGOUT",
            "ASSEMBLED-CORNERS",
            OutputImage::Png(corners_image),
        ));
        out
    }

    #[must_use]
    pub fn get_side_info(&self, side: Side) -> SideSpacing {
        match side {
            Side::North => SideSpacing {
                start: 0,
                end: self.cut_pos.y,
            },
            Side::South => SideSpacing {
                start: self.cut_pos.y,
                end: self.icon_size.y,
            },
            Side::East => SideSpacing {
                start: self.cut_pos.x,
                end: self.icon_size.x,
            },
            Side::West => SideSpacing {
                start: 0,
                end: self.cut_pos.x,
            },
        }
    }
}
