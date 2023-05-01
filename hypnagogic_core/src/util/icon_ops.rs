use dmi::icon::IconState;
use image::DynamicImage;

// Removes duplicate frames from the icon state's animation, if it has any
pub fn dedupe_frames(icon_state: IconState) -> IconState {
    if icon_state.frames <= 1 {
        return icon_state;
    }
    let Some(current_delays) = &icon_state.delay else {
        return icon_state;
    };

    struct AccumulatedAnim {
        delays: Vec<f32>,
        frames: Vec<DynamicImage>,
        working_index: u32,
    }
    
    // As we walk through the frames in this icon state, we're going to keep track of the ones that
    // Are duplicates, and "dedupe" them by simply adding extra frame delay and removing the extra frame
    let deduped_anim = current_delays.iter().zip(icon_state.images.into_iter())
        .fold(AccumulatedAnim {
            delays: Vec::new(), 
            frames: Vec::new(),
            working_index: 0,
        }, |mut acc, elem| {
            let (&current_delay, current_frame) = elem;
            if acc.frames.len() == 0 {
                acc.delays.push(current_delay);
                acc.frames.push(current_frame);
                return acc
            }
            let current_index = acc.working_index;
            if acc.frames[current_index as usize] == current_frame {
                acc.delays[current_index as usize] += current_delay;
            }
            else {
                acc.delays.push(current_delay);
                acc.frames.push(current_frame);
                acc.working_index += 1;
            }
            acc
        });
    
    return IconState {
        frames: deduped_anim.working_index + 1,
        images: deduped_anim.frames,
        delay: Some(deduped_anim.delays),
        ..icon_state
    };
}
