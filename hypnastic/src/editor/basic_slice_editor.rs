use crate::editor::{ActiveFileData, Editor};
use egui::Ui;
use hypnagogic_core::operations::cutters::bitmask_slice::BitmaskSlice;

impl Editor for BitmaskSlice {
    fn draw(&mut self, ui: &mut Ui, active_file: &mut ActiveFileData) {
        todo!()
    }
}
