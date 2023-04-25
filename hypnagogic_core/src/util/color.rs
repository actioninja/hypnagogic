use image::{DynamicImage, GenericImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    pub fn from_hex_str(hex_str: &str) -> Result<Self, HexConversionError> {
        let Some(hex_str) = hex_str.strip_prefix('#') else { return Err(HexConversionError::MissingHash(hex_str.to_string())) };

        let hex_str = match hex_str.len() {
            3 | 4 => {
                let mut new_hex = String::new();
                for c in hex_str.chars() {
                    new_hex.push(c);
                    new_hex.push(c);
                }
                new_hex
            }
            6 | 8 => hex_str.to_string(),
            _ => {
                return Err(HexConversionError::InvalidLength(
                    hex_str.to_string(),
                    hex_str.len(),
                ))
            }
        };

        let mut hex_chars = hex_str.chars();
        let red = u8::from_str_radix(
            &format!("{}{}", hex_chars.next().unwrap(), hex_chars.next().unwrap()),
            16,
        )?;
        let green = u8::from_str_radix(
            &format!("{}{}", hex_chars.next().unwrap(), hex_chars.next().unwrap()),
            16,
        )?;
        let blue = u8::from_str_radix(
            &format!("{}{}", hex_chars.next().unwrap(), hex_chars.next().unwrap()),
            16,
        )?;
        let alpha = match hex_chars.next() {
            Some(h) => u8::from_str_radix(&format!("{}{}", h, hex_chars.next().unwrap()), 16)?,
            None => 255,
        };

        Ok(Self {
            red,
            green,
            blue,
            alpha,
        })
    }

    pub fn to_hex_str(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex_str())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex_str = String::deserialize(deserializer)?;
        Self::from_hex_str(&hex_str).map_err(serde::de::Error::custom)
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        [color.red, color.green, color.blue, color.alpha]
    }
}

impl From<[u8; 4]> for Color {
    fn from(color: [u8; 4]) -> Self {
        Self {
            red: color[0],
            green: color[1],
            blue: color[2],
            alpha: color[3],
        }
    }
}

impl TryFrom<Color> for [u8; 3] {
    type Error = ColorError;

    fn try_from(color: Color) -> Result<Self, Self::Error> {
        if color.alpha != 255 {
            return Err(ColorError::HasAlpha);
        }
        Ok([color.red, color.green, color.blue])
    }
}

impl From<[u8; 3]> for Color {
    fn from(color: [u8; 3]) -> Self {
        Self {
            red: color[0],
            green: color[1],
            blue: color[2],
            alpha: 255,
        }
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    fn from(color: Color) -> Self {
        (color.red, color.green, color.blue, color.alpha)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8, u8)) -> Self {
        Self {
            red: color.0,
            green: color.1,
            blue: color.2,
            alpha: color.3,
        }
    }
}

impl TryFrom<Color> for (u8, u8, u8) {
    type Error = ColorError;

    fn try_from(color: Color) -> Result<Self, Self::Error> {
        if color.alpha != 255 {
            return Err(ColorError::HasAlpha);
        }
        Ok((color.red, color.green, color.blue))
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8)) -> Self {
        Self {
            red: color.0,
            green: color.1,
            blue: color.2,
            alpha: 255,
        }
    }
}

#[derive(Debug, Error)]
pub enum ColorError {
    #[error("Attempted to convert a color with alpha to a 3 byte type")]
    HasAlpha,
    #[error("Error converting to hex: {0}")]
    HexConversionError(#[from] HexConversionError),
}

#[derive(Debug, Error)]
pub enum HexConversionError {
    #[error("Invalid hex string (missing #): {0}")]
    MissingHash(String),
    #[error("Invalid hex string (invalid length): {0} (length: {1})")]
    InvalidLength(String, usize),
    #[error("Invalid hex string (invalid characters): {0}")]
    BadHex(#[from] ParseIntError),
}

pub fn fill_image_color(image: &mut DynamicImage, color: Color) {
    let mut buffer = image.clone().into_rgba8();
    for image::Rgba([r, g, b, a]) in buffer.pixels_mut() {
        if *a != 0 {
            *r = color.red;
            *g = color.green;
            *b = color.blue;
            *a = color.alpha;
        }
    }
    *image = DynamicImage::ImageRgba8(buffer);
}

#[cfg(test)]
mod tests {
    use crate::util::color::hex_to_tuple;

    #[test]
    fn hex_to_tuple_test() {
        let hex = "#ff0000";
        let (r, g, b, a) = hex_to_tuple(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
        assert_eq!(a, 255);

        let hex = "#ff0000ff";
        let (r, g, b, a) = hex_to_tuple(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
        assert_eq!(a, 255);

        let hex = "#ffff00ff";
        let (r, g, b, a) = hex_to_tuple(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
        assert_eq!(a, 255);

        let hex = "#f00";
        let (r, g, b, a) = hex_to_tuple(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
        assert_eq!(a, 255);

        let hex = "#f00f";
        let (r, g, b, a) = hex_to_tuple(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
        assert_eq!(a, 255);

        let hex = "#f00ff";
        let result = hex_to_tuple(hex);
        assert!(result.is_err());

        let hex = "f00ff";
        let result = hex_to_tuple(hex);
        assert!(result.is_err());

        let hex = "#f00ffg";
        let result = hex_to_tuple(hex);
        assert!(result.is_err());
    }
}
