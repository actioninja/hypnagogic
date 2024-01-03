use dmi::icon::IconState;
use image::{DynamicImage, GenericImage};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::blocks::cutters::StringMap;
use crate::operations::error::{ProcessorError, ProcessorResult};
use crate::operations::{IconOperationConfig, InputIcon, OperationMode, ProcessorPayload};

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct BitmaskSliceReconstruct {
    // List of icon states to extract
    pub extract: Vec<String>,
    // Map of state name -> state to insert as
    pub bespoke: Option<StringMap>,
    // Map of key -> value to set on the created config
    // Exists to let you set arbitrary values
    pub set: Option<StringMap>,
}

impl IconOperationConfig for BitmaskSliceReconstruct {
    #[tracing::instrument(skip(input))]
    fn perform_operation(
        &self,
        input: &InputIcon,
        mode: OperationMode,
    ) -> ProcessorResult<ProcessorPayload> {
        debug!("Starting bitmask slice reconstruction");
        let InputIcon::Dmi(icon) = input else {
            return Err(ProcessorError::FormatError(
                "This operation only accepts dmis".to_string(),
            ));
        };

        // First, pull out icon states from DMI
        let states = icon.states.clone();

        let bespoke = match self.bespoke.as_ref() {
            Some(bespoke) => bespoke.clone(),
            None => StringMap::default(),
        };

        // Try and work out the output prefix by pulling from the first frame
        let mut problem_entries: Vec<String> = vec![];
        let output_prefix = states
            .first()
            .and_then(|first_frame| first_frame.name.split('-').next());

        // Next, check if anything conflicts, if it does we'll error
        let frames_drop_prefix = states
            .clone()
            .into_iter()
            .map(|state| {
                let full_name = state.name.clone();
                let mut split_name = full_name.split('-');
                let prefix = split_name.next();
                if prefix != output_prefix {
                    problem_entries.push(full_name.clone());
                }
                let suffix = split_name.last().unwrap_or(prefix.unwrap_or_default());
                (state, suffix.to_string())
            })
            .collect::<Vec<(IconState, String)>>();

        if let Some(troublesome_states) = problem_entries
            .into_iter()
            .reduce(|acc, elem| format!("{acc}, {elem}"))
        {
            return Err(ProcessorError::DmiError(format!(
                "The following icon states are named with inconsistent prefixes (with the rest of \
                 the file) [{troublesome_states}]"
            )));
        }
        // Now, we remove the "core" frames, and dump them out
        let extract_length = self.extract.len();
        let iter_extract = self.extract.clone().into_iter();
        let mut bespoke_found: Vec<String> = vec![];
        // Extract just the bits we care about
        let mut trimmed_frames = frames_drop_prefix
            .clone()
            .into_iter()
            .filter_map(|(mut state, suffix)| {
                state.name = suffix.clone();
                if bespoke.get(suffix.as_str()).is_some() {
                    bespoke_found.push(suffix);
                    Some(state)
                } else if self.extract.contains(&suffix) {
                    Some(state)
                } else {
                    None
                }
            })
            .collect::<Vec<IconState>>();

        // Check for any states that aren't extracted and aren't entirely numbers
        // If we find any, error (cause we're dropping them here)
        let strings_caught = trimmed_frames
            .clone()
            .into_iter()
            .map(|state| state.name.clone())
            .collect::<Vec<String>>();
        let ignored_states = frames_drop_prefix
            .into_iter()
            .filter_map(|(_, suffix)| {
                if suffix.parse::<i32>().is_ok()
                    || strings_caught.iter().any(|caught| *caught == suffix)
                {
                    None
                } else {
                    Some(format!("({suffix})"))
                }
            })
            .reduce(|acc, elem| {
                format! {"{acc}, {elem}"}
            });


        if let Some(missed_suffixes) = ignored_states {
            let caught_text = strings_caught
                .into_iter()
                .reduce(|acc, entry| format!("{acc}, {entry}"))
                .unwrap_or_default();
            return Err(ProcessorError::DmiError(format!(
                "Restoration would fail to properly parse the following icon states \
                 [{missed_suffixes}] not parsed like [{caught_text}]"
            )));
        }

        // Alright next we're gonna work out the order of our insertion into the png
        // based off the order of the extract/bespoke maps Extract first, then
        // bespoke
        let bespoke_iter = bespoke_found.clone().into_iter();

        // I don't like all these clones but position() mutates and I don't want that so
        // I'm not sure what else to do
        let get_pos = |search_for: &String| {
            iter_extract
                .clone()
                .position(|name| name == *search_for)
                .unwrap_or(
                    if let Some(position) =
                        bespoke_iter.clone().position(|name| name == *search_for)
                    {
                        position + extract_length
                    } else {
                        usize::MAX
                    },
                )
        };
        trimmed_frames.sort_by(|a, b| {
            let a_pos = get_pos(&a.name);
            let b_pos = get_pos(&b.name);
            a_pos.cmp(&b_pos)
        });

        let frame_count = trimmed_frames.len();
        let longest_frame = trimmed_frames
            .clone()
            .into_iter()
            .map(|state| state.frames)
            .max()
            .unwrap_or(1);
        // We now have a set of frames that we want to draw, ordered as requested
        // So all we gotta do is make that png
        // We assume all states have the same animation length,
        let mut output_image =
            DynamicImage::new_rgba8(icon.width * frame_count as u32, icon.height * longest_frame);
        let delays: Option<Vec<f32>> = trimmed_frames
            .first()
            .and_then(|first_frame| first_frame.delay.clone());

        let text_delays = |textify: Vec<f32>, suffix: &str| -> String {
            format!(
                "[{}]",
                textify
                    .into_iter()
                    .map(|ds| format!("{ds}{suffix}"))
                    .reduce(|acc, text_ds| format!("{acc}, {text_ds}"))
                    .unwrap_or_default()
            )
        };
        for (x, state) in trimmed_frames.into_iter().enumerate() {
            if delays != state.delay {
                return Err(ProcessorError::DmiError(format!(
                    "Icon state {}'s delays {} do not match with the rest of the file {}",
                    state.name,
                    text_delays(state.delay.unwrap_or_default(), "ds"),
                    text_delays(delays.unwrap_or_default(), "ds")
                )));
            }
            for (y, frame) in state.images.into_iter().enumerate() {
                debug!("{} {} {}", state.name, x, y);
                output_image
                    .copy_from(&frame, (x as u32) * icon.width, (y as u32) * icon.height)
                    .unwrap_or_else(|_| {
                        panic!("Failed to copy frame (bad dmi?): {} #{}", state.name, y)
                    });
            }
        }

        let mut config: Vec<String> = vec![];
        if let Some(prefix_name) = output_prefix {
            config.push(format!("output_name = \"{prefix_name}\""));
        }
        if let Some(map) = &self.set {
            map.0.clone().into_iter().for_each(|entry| {
                config.push(format!("{} = {}", entry.0, entry.1));
            });
            config.push(String::new());
        }
        let mut count = frame_count - bespoke_found.len();
        if let Some(map) = &self.bespoke {
            config.push("[prefabs]".to_string());
            map.0.clone().into_iter().for_each(|entry| {
                config.push(format!("{} = {}", entry.1, count));
                count += 1;
            });
            config.push(String::new());
        }
        if let Some(actual_delay) = delays {
            config.push("[animation]".to_string());
            config.push(format!("delays = {}", text_delays(actual_delay, "")));
            config.push(String::new());
        };
        config.push("[icon_size]".to_string());
        config.push(format!("x = {}", icon.width));
        config.push(format!("y = {}", icon.height));
        config.push(String::new());
        config.push("[output_icon_size]".to_string());
        config.push(format!("x = {}", icon.width));
        config.push(format!("y = {}", icon.height));
        config.push(String::new());
        config.push("[cut_pos]".to_string());
        config.push(format!("x = {}", icon.width / 2));
        config.push(format!("y = {}", icon.height / 2));
        // Newline gang
        config.push(String::new());
        Ok(ProcessorPayload::wrap_png_config(
            ProcessorPayload::from_image(output_image),
            config.join("\n"),
        ))
    }

    fn verify_config(&self) -> ProcessorResult<()> {
        // TODO: Actual verification
        Ok(())
    }
}
