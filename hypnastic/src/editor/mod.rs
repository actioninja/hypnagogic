pub mod basic_slice_editor;

use egui::Ui;
use hypnagogic_core::operations::cutters::bitmask_slice::BitmaskSlice;
use hypnagogic_core::operations::IconOperation;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveFileData {
    pub path: PathBuf,
    pub image_path: PathBuf,
    #[serde(skip)]
    pub image: DynamicImage,
    pub operation: Option<IconOperation>,
    pub template: String,
    pub template_operation: Option<IconOperation>,
}

pub trait Editor {
    fn draw(&mut self, ui: &mut egui::Ui, active_file: &mut ActiveFileData);
}

impl Editor for IconOperation {
    fn draw(&mut self, ui: &mut Ui, active_file: &mut ActiveFileData) {
        match self {
            IconOperation::BitmaskSlice(slice) => {
                slice.draw(ui, active_file);
            }
            _ => {
                ui.label("This operation is not yet supported");
            }
        }
    }
}
