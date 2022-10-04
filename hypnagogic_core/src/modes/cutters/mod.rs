pub mod bitmask_dir_visibility;
pub mod bitmask_slice;
pub mod bitmask_windows;

#[must_use]
pub fn delay_repeat(to_repeat: &Option<Vec<f32>>, amount: usize) -> Option<Vec<f32>> {
    to_repeat
        .clone()
        .map(|inner| inner.iter().cycle().take(amount).copied().collect())
}
