use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dmi::icon::Icon;
use image::DynamicImage;
use thiserror::Error;
use tracing::error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum DmiCompareError {
    #[error("Different icon sizes: {0:?} vs {1:?}")]
    DifferentIconSizes((u32, u32), (u32, u32)),
    #[error("Different icon states: {0:?} vs {1:?}")]
    DifferentIconStates(Vec<String>, Vec<String>),
    #[error("Different icon state order: {0:?} vs {1:?}")]
    DifferentIconStateOrder(Vec<String>, Vec<String>),
    #[error("Different icon state pixel data")]
    DifferentIconStatePixelData(HashMap<String, Vec<(DynamicImage, DynamicImage)>>),
}

pub fn compare_dmi(dmi1: &Icon, dmi2: &Icon) -> Result<(), DmiCompareError> {
    if dmi1.width != dmi2.width || dmi1.height != dmi2.height {
        return Err(DmiCompareError::DifferentIconSizes(
            (dmi1.width, dmi1.height),
            (dmi2.width, dmi2.height),
        ));
    }

    let states_equal = dmi1
        .states
        .iter()
        .zip(dmi2.states.iter())
        .all(|(state1, state2)| state1.name == state2.name);
    if !states_equal {
        let mut state_names1: Vec<String> =
            dmi1.states.iter().map(|state| state.name.clone()).collect();
        let mut state_names2: Vec<String> =
            dmi2.states.iter().map(|state| state.name.clone()).collect();
        state_names1.sort();
        state_names2.sort();
        let sorted_states_equal = state_names1
            .iter()
            .zip(state_names2.iter())
            .all(|(state1, state2)| state1 == state2);
        return if sorted_states_equal {
            Err(DmiCompareError::DifferentIconStateOrder(
                state_names1,
                state_names2,
            ))
        } else {
            Err(DmiCompareError::DifferentIconStates(
                state_names1,
                state_names2,
            ))
        };
    }

    let mut disparate_hash_map = HashMap::new();
    for (state1, state2) in dmi1.states.iter().zip(dmi2.states.iter()) {
        let state1_iter = state1.images.iter();
        let state2_iter = state2.images.iter();
        let all_frames_match = state1_iter
            .clone()
            .zip(state2_iter.clone())
            .all(|(frame1, frame2)| frame1 == frame2);
        if !all_frames_match {
            let mut frame_pairs = vec![];
            for (frame1, frame2) in state1_iter.zip(state2_iter) {
                if frame1 != frame2 {
                    frame_pairs.push((frame1.clone(), frame2.clone()));
                }
            }
            disparate_hash_map.insert(state1.name.clone(), frame_pairs);
        }
    }
    if disparate_hash_map.is_empty() {
        Ok(())
    } else {
        Err(DmiCompareError::DifferentIconStatePixelData(
            disparate_hash_map,
        ))
    }
}

#[derive(Debug, Error)]
pub enum CompareFailureReasonError {
    #[error("Error comparing DMIs: {0}")]
    DmiCompareError(#[from] DmiCompareError),
    #[error("Error walking directory: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct CompareFailureError {
    pub a: PathBuf,
    pub b: PathBuf,
    pub source: CompareFailureReasonError,
}

impl CompareFailureError {
    pub fn new(a: PathBuf, b: PathBuf, source: CompareFailureReasonError) -> Self {
        Self { a, b, source }
    }
}

pub fn deep_compare_path(path1: &Path, path2: &Path) -> Result<(), Vec<CompareFailureError>> {
    let path1_iter = WalkDir::new(path1).into_iter();
    let path2_iter = WalkDir::new(path2).into_iter();

    let res: Vec<_> = path1_iter
        .zip(path2_iter)
        .filter_map(|(entry1, entry2)| {
            if let (Ok(entry1), Ok(entry2)) = (entry1, entry2) {
                if entry1.file_type().is_file() && entry2.file_type().is_file() {
                    let file1 = std::fs::File::open(entry1.path()).unwrap();
                    let dmi1 = Icon::load(file1).unwrap();

                    let file2 = std::fs::File::open(entry2.path()).unwrap();
                    let dmi2 = Icon::load(file2).unwrap();
                    let res = compare_dmi(&dmi1, &dmi2);
                    res.err().map(|inner_err| {
                        CompareFailureError::new(
                            entry1.into_path(),
                            entry2.into_path(),
                            CompareFailureReasonError::DmiCompareError(inner_err),
                        )
                    })
                } else {
                    None
                }
            } else {
                panic!("Unable to walk directory (check ownership and permissions)");
            }
        })
        .collect();

    if res.is_empty() {
        Ok(())
    } else {
        Err(res)
    }
}
