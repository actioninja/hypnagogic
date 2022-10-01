use crate::util::adjacency::Adjacency;
use dmi::icon::{Icon, IconState};
use enum_iterator::all;
use fixed_map::Map;
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use std::collections::HashMap;
use std::io::{BufRead, Seek};
use std::path::PathBuf;
use tracing::{debug, trace};

use crate::modes::error::ProcessorResult;
use crate::modes::CutterModeConfig;
use crate::util::corners::{Corner, CornerType, Side};

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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BitmaskSlice {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub output_name: Option<String>,

    pub icon_size_x: u32,
    pub icon_size_y: u32,

    pub output_icon_pos_x: u32,
    pub output_icon_pos_y: u32,
    pub output_icon_size_x: u32,
    pub output_icon_size_y: u32,

    pub positions: CornerConfig,

    #[serde(default)]
    pub cut_position_x: u32,
    #[serde(default)]
    pub cut_position_y: u32,

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

            output_icon_pos_x: 0,
            output_icon_pos_y: 0,
            output_icon_size_x: 32,
            output_icon_size_y: 32,

            positions: CornerConfig::default(),

            cut_position_x: 16,
            cut_position_y: 16,

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
    #[tracing::instrument(skip(input))]
    fn perform_operation<R: BufRead + Seek>(
        &self,
        input: &mut R,
    ) -> ProcessorResult<Vec<(String, Icon)>> {
        debug!("Starting icon op");
        let mut img = image::load(input, ImageFormat::Png)?;
        let (corners, prefabs) = self.generate_corners(&mut img)?;

        let (_in_x, in_y) = img.dimensions();
        let num_frames = in_y / self.icon_size_y;

        let possible_states = if self.is_diagonal {
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
        // Even though this is the same loop as above, all states need to be generated first for the
        // Rotation to work correctly, so it must be done as a second loop.
        let mut icon_states = vec![];

        // fairly gnarly iterator chain; loops delay sequence and then takes number of frames
        let delay: Option<Vec<f32>> = self.delay.clone().map(|inner| {
            inner
                .iter()
                .cycle()
                .take(num_frames as usize)
                .copied()
                .collect()
        });

        for signature in 0..possible_states {
            let mut icon_state_frames = vec![];

            let adjacency = Adjacency::from_bits(signature as u8).unwrap();

            for icon_state_dir in &icon_directions {
                let rotated_sig = adjacency.rotate_to(*icon_state_dir);
                trace!(sig = ?icon_state_dir, rotated_sig = ?rotated_sig, "Rotated");
                icon_state_frames.extend(assembled[&rotated_sig].clone());
            }

            icon_states.push(IconState {
                name: format!("{}", signature),
                dirs: icon_directions.len() as u8,
                frames: num_frames,
                images: icon_state_frames,
                delay: delay.clone(),
                ..Default::default()
            });
        }

        let output_icon = Icon {
            version: Default::default(),
            width: self.output_icon_size_x,
            height: self.output_icon_size_y,
            states: icon_states,
        };

        Ok(vec![("".to_string(), output_icon)])
    }

    #[tracing::instrument(skip(input))]
    fn debug_output<R: BufRead + Seek>(
        &self,
        input: &mut R,
        output_dir: PathBuf,
    ) -> ProcessorResult<DynamicImage> {
        debug!("Starting debug output");
        let mut img = image::load(input, ImageFormat::Png)?;
        let (corners, _prefabs) = self.generate_corners(&mut img)?;

        let num_types = corners.get(Corner::NorthEast).unwrap().len() as u32;
        trace!(number = ?num_types, "found types");

        let mut corners_image =
            DynamicImage::new_rgb8(num_types * self.icon_size_x, self.icon_size_y);

        for (corner, map) in corners.iter() {
            let (horizontal, vertical) = corner.sides_of_corner();
            let horizontal = self.get_side_info(horizontal);
            let vertical = self.get_side_info(vertical);
            trace!(corner = ?corner, horizontal = ?horizontal, vertical = ?vertical, "Starting corner");
            for (corner_type, vec) in map.iter() {
                let position = *self.positions.get(corner_type).unwrap();
                let frame = vec.get(0).unwrap();
                let output_file = output_dir
                    .as_path()
                    .join(format!("{corner:?}-{corner_type:?}.png"));
                frame.save(output_file)?;
                imageops::replace(
                    &mut corners_image,
                    frame,
                    ((position * self.icon_size_x) + horizontal.start) as i64,
                    vertical.start as i64,
                );
            }
        }

        Ok(corners_image)
    }
}

type Corners = Map<Corner, Map<CornerType, Vec<DynamicImage>>>;
type Prefabs = HashMap<Adjacency, Vec<DynamicImage>>;

//possible icon set is the powerset of the possible directions
//the size of a powerset is always 2^n where n is number of discrete elements
pub const SIZE_OF_CARDINALS: usize = usize::pow(2, 4);
pub const SIZE_OF_DIAGONALS: usize = usize::pow(2, 8);

impl BitmaskSlice {
    /// Generates corners
    /// # Errors
    /// Errors on malformed image
    /// # Panics
    /// Shouldn't panic
    #[tracing::instrument(skip(img))]
    pub fn generate_corners(&self, img: &mut DynamicImage) -> ProcessorResult<(Corners, Prefabs)> {
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

                    let x_spacing = self.get_side_info(x_side);
                    let y_spacing = self.get_side_info(y_side);
                    let x_offset = x_spacing.start;
                    let y_offset = y_spacing.start;

                    let x = (position * self.icon_size_x) + x_offset;
                    let y = (frame_num * self.icon_size_y) + y_offset;

                    let width = x_spacing.step();
                    let height = y_spacing.step();
                    trace!(
                        corner = ?corner,
                        corner_type = ?corner_type,
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

    /// Blah
    /// # Panics
    /// Whatever
    #[must_use]
    pub fn generate_icons(
        &self,
        corners: &Corners,
        prefabs: &Prefabs,
        num_frames: u32,
        possible_states: usize,
    ) -> HashMap<Adjacency, Vec<DynamicImage>> {
        let mut assembled: HashMap<Adjacency, Vec<DynamicImage>> = HashMap::new();
        for signature in 0..possible_states {
            let adjacency = Adjacency::from_bits(signature as u8).unwrap();
            let mut icon_state_images = vec![];
            for frame in 0..num_frames {
                if prefabs.contains_key(&adjacency) {
                    let mut frame_image =
                        DynamicImage::new_rgb8(self.output_icon_size_x, self.output_icon_size_y);
                    imageops::replace(
                        &mut frame_image,
                        prefabs
                            .get(&adjacency)
                            .unwrap()
                            .get(frame as usize)
                            .unwrap(),
                        self.output_icon_pos_x as i64,
                        self.output_icon_pos_y as i64,
                    );

                    icon_state_images.push(frame_image);
                } else {
                    let mut frame_image =
                        DynamicImage::new_rgb8(self.output_icon_size_x, self.output_icon_size_y);

                    for corner in all::<Corner>() {
                        let corner_type = adjacency.get_corner_type(corner);
                        let corner_img = &corners
                            .get(corner)
                            .unwrap()
                            .get(corner_type)
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

    #[must_use]
    pub fn get_side_info(&self, side: Side) -> SideSpacing {
        match side {
            Side::North => SideSpacing {
                start: 0,
                end: self.cut_position_y,
            },
            Side::South => SideSpacing {
                start: self.cut_position_y,
                end: self.icon_size_y,
            },
            Side::East => SideSpacing {
                start: self.cut_position_x,
                end: self.icon_size_x,
            },
            Side::West => SideSpacing {
                start: 0,
                end: self.cut_position_x,
            },
        }
    }
}
