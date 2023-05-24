use crate::util::color::Color;
use image::{DynamicImage, GenericImage};
use serde::{Deserialize, Serialize};

pub fn draw_rect(image: &mut DynamicImage, x: u32, y: u32, width: u32, height: u32, color: Color) {
    for x in x..x + width {
        for y in y..y + height {
            image.put_pixel(x, y, image::Rgba(color.into()));
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderStyle {
    Solid,
    Dotted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub style: BorderStyle,
    pub color: Color,
}

pub fn draw_border(
    image: &mut DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    border: Border,
) {
    match border.style {
        BorderStyle::Solid => {
            draw_rect(image, x, y, width, 1, border.color);
            draw_rect(image, x, y + height - 1, width, 1, border.color);
            draw_rect(image, x, y, 1, height, border.color);
            draw_rect(image, x + width - 1, y, 1, height, border.color);
        }
        BorderStyle::Dotted => {
            for x in x..x + width {
                if x % 2 == 0 {
                    image.put_pixel(x, y, image::Rgba(border.color.into()));
                }
                if x % 2 == 1 {
                    image.put_pixel(x, y + height - 1, image::Rgba(border.color.into()));
                }
            }
            for y in y..y + height {
                if y % 2 == 0 {
                    image.put_pixel(x, y, image::Rgba(border.color.into()));
                }
                if y % 2 == 1 {
                    image.put_pixel(x + width - 1, y, image::Rgba(border.color.into()));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {}
