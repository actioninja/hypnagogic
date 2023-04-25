use image::{DynamicImage, GenericImage};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// all printable ascii characters
const VALID_CHARS: [char; 95] = [
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2',
    '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
];

const CHARACTER_RAW_BYTES: &[u8; 371] = include_bytes!("characters.png");
static CHARACTER_IMAGE: Lazy<DynamicImage> =
    Lazy::new(|| image::load_from_memory(CHARACTER_RAW_BYTES).unwrap());

const CHARACTER_WIDTH: u32 = 3;
const CHARACTER_HEIGHT: u32 = 5;

const MAX_LENGTH: usize = 100;

const fn is_char_narrow(char: char) -> Option<u32> {
    match char {
        ';' | ',' | 'l' | 'j' | '(' | ')' | '[' | ']' | '`' | '\'' => Some(2),
        '!' | '.' | 'i' => Some(1),
        _ => None,
    }
}

#[must_use]
pub fn generate_text_line(text_to_gen: &str) -> DynamicImage {
    let num_chars = text_to_gen.chars().count() as u32;
    // -1 because we don't want to count the last space
    let num_spaces = num_chars - 1;
    let width = CHARACTER_WIDTH * num_chars + num_spaces;
    let height = CHARACTER_HEIGHT;
    let mut image = DynamicImage::new_rgba8(width, height);
    let mut pos = 0;
    for (i, char) in text_to_gen.chars().enumerate() {
        if char == ' ' {
            pos += 2;
            continue;
        }
        let x = pos;
        pos += is_char_narrow(char).unwrap_or(CHARACTER_WIDTH) + 1;
        let y = 0;
        let char_image = get_char_crop(char).unwrap();
        image
            .copy_from(&char_image, x, y)
            .expect("Failed to copy (bad image?)");
    }
    // crop off the last space
    image.crop_imm(0, 0, pos - 1, CHARACTER_HEIGHT)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// generates a block of text
/// splits the text into lines by spaces and generates each line
/// then combines the lines into a single image
pub fn generate_text_block(text_to_gen: &str, alignment: Alignment) -> DynamicImage {
    let split: Vec<&str> = text_to_gen.split(' ').collect();
    let images: Vec<DynamicImage> = split.iter().map(|&s| generate_text_line(s)).collect();
    let longest_line = images.iter().max_by_key(|i| i.width()).unwrap().width();
    let height = split.len() * CHARACTER_HEIGHT as usize + (split.len() - 1);
    let mut image = DynamicImage::new_rgba8(longest_line, height as u32);
    for (i, line) in images.iter().enumerate() {
        let x = match alignment {
            Alignment::Left => 0,
            Alignment::Center => (longest_line - line.width()) / 2,
            Alignment::Right => longest_line - line.width(),
        };
        let y = i * (CHARACTER_HEIGHT as usize + 1);
        image
            .copy_from(line, x, y as u32)
            .expect("Failed to copy (bad image?)");
    }
    image
}

#[must_use]
pub fn get_char_crop(char: char) -> Option<DynamicImage> {
    let (x, y) = lookup_coords(char)?;
    let x = x * CHARACTER_WIDTH;
    let y = y * CHARACTER_HEIGHT;
    let crop = CHARACTER_IMAGE.crop_imm(x, y, CHARACTER_WIDTH, CHARACTER_HEIGHT);
    Some(crop)
}

/// bootleg contains that is const
#[must_use]
const fn const_contains(slice: &[char], char: char) -> bool {
    let mut i = 0;
    while i < slice.len() {
        if slice[i] == char {
            return true;
        }
        i += 1;
    }
    false
}

/// returns the x and y coordinates of the character in the character grid
#[must_use]
pub const fn lookup_coords(char: char) -> Option<(u32, u32)> {
    if !const_contains(&VALID_CHARS, char) || char == ' ' {
        return None;
    }
    let grid_width: u32 = 16;
    let starting_char = '!' as u32;

    let char_index = char as u32 - starting_char;

    let x = char_index % grid_width;
    let y = char_index / grid_width;
    Some((x, y))
}

#[derive(Debug, Error)]
pub enum TextError {
    #[error("Text is too long ({0}), max length is {MAX_LENGTH}")]
    TooLong(u32),
    #[error("Text contains invalid characters: {0:?}")]
    InvalidCharacters(Vec<char>),
}

#[cfg(test)]
mod test {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn char_lookup() {
        let char = '!';
        let (x, y) = lookup_coords(char).unwrap();
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        let char = ' ';
        let failed = lookup_coords(char);
        assert_eq!(failed, None);

        let char = 'a';
        let (x, y) = lookup_coords(char).unwrap();
        assert_eq!(x, 0);
        assert_eq!(y, 4);
    }

    #[test]
    fn char_crop() {
        let char = '!';
        let image = get_char_crop(char).unwrap();
        assert_eq!(image.dimensions(), (CHARACTER_WIDTH, CHARACTER_HEIGHT));
    }

    #[test]
    fn write_out_test_line() {
        let text = "Hello, world!";
        let image = generate_text_line(text);
        image.save("./junk/test_line.png").unwrap();
    }

    #[test]
    fn write_out_test_block() {
        let text = "Hello, world! This is a test of the text generator. I hope it works!";
        let image = generate_text_block(text, Alignment::Center);
        image.save("./junk/test_block.png").unwrap();
    }
}
