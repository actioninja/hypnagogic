use dmi::icon::IconState;

// Removes duplicate frames from the icon state's animation, if it has any
pub fn dedupe_frames(mut icon_state: IconState) -> IconState {
    if icon_state.frames <= 1 {
        return icon_state;
    }
    let current_delays = match icon_state.delay {
        Some(delay) => delay,
        None => return icon_state,
    };

    // As we walk through the frames in this icon state, we're going to keep track of the ones that
    // Are duplicates, and "dedupe" them by simply adding extra frame delay and removing the extra frame
    let mut delay_index = 0;
    let mut current_delay = current_delays[delay_index];
    let mut frame_count = icon_state.frames;
    let mut previous_bytes = None;
    // List of concrete delays. We'll push to this once we're sure we're happy with the current value
    let mut new_delays = Vec::new();
    let mut new_images = Vec::new();
    for image in icon_state.images {
        let image_bytes = image.clone().into_bytes();
        if let Some(previous) = previous_bytes {
            if previous == image_bytes {
                previous_bytes = Some(previous);
                frame_count -= 1;
                delay_index += 1;
                current_delay += current_delays[delay_index];
                continue;
            }
            new_delays.push(current_delay);
            delay_index += 1;
            current_delay = current_delays[delay_index];
        }
        previous_bytes = Some(image_bytes);
        new_images.push(image);
    }
    new_delays.push(current_delay);

    icon_state.frames = frame_count;
    icon_state.images = new_images;
    icon_state.delay = Some(new_delays);
    return icon_state;
}
