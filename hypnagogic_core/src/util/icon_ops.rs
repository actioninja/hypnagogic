use dmi::icon::IconState;
use image::{DynamicImage, GenericImageView};

use crate::util::color::Color;

// Removes duplicate frames from the icon state's animation, if it has any
#[must_use]
pub fn dedupe_frames(icon_state: IconState) -> IconState {
    struct AccumulatedAnim {
        delays: Vec<f32>,
        frames: Vec<DynamicImage>,
        working_index: u32,
    }

    if icon_state.frames <= 1 {
        return icon_state;
    }
    let Some(current_delays) = &icon_state.delay else {
        return icon_state;
    };

    // As we walk through the frames in this icon state, we're going to keep track
    // of the ones that Are duplicates, and "dedupe" them by simply adding extra
    // frame delay and removing the extra frame
    let deduped_anim = current_delays.iter().zip(icon_state.images).fold(
        AccumulatedAnim {
            delays: Vec::new(),
            frames: Vec::new(),
            working_index: 0,
        },
        |mut acc, elem| {
            let (&current_delay, current_frame) = elem;
            if acc.frames.is_empty() {
                acc.delays.push(current_delay);
                acc.frames.push(current_frame);
                return acc;
            }
            let current_index = acc.working_index;
            if acc.frames[current_index as usize] == current_frame {
                acc.delays[current_index as usize] += current_delay;
            } else {
                acc.delays.push(current_delay);
                acc.frames.push(current_frame);
                acc.working_index += 1;
            }
            acc
        },
    );

    IconState {
        frames: deduped_anim.working_index + 1,
        images: deduped_anim.frames,
        delay: Some(deduped_anim.delays),
        ..icon_state
    }
}

#[must_use]
pub fn colors_in_image(image: &DynamicImage) -> Vec<Color> {
    let mut colors = Vec::new();
    for pixel in image.pixels() {
        let color = pixel.2;
        if !colors.contains(&color) {
            colors.push(color);
        }
    }
    colors
        .iter()
        .map(|c| Color::new(c.0[0], c.0[1], c.0[2], c.0[3]))
        .collect()
}

pub fn sort_colors_by_luminance(colors: &mut [Color]) {
    colors.sort_by(|a, b| a.luminance().partial_cmp(&b.luminance()).unwrap());
}

#[must_use]
pub fn pick_contrasting_colors(colors: &[Color]) -> (Color, Color) {
    let mut sorted_colors = colors.to_vec();
    sort_colors_by_luminance(&mut sorted_colors);
    let len_as_f32 = colors.len() as f32;
    let first = 0.10 * len_as_f32;
    let first_index = (first.floor() as usize).saturating_sub(1);
    let second = 0.90 * len_as_f32;
    let second_index = (second.floor() as usize).saturating_sub(1);
    (sorted_colors[first_index], sorted_colors[second_index])
}
