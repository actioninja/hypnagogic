use crate::config::blocks::generators::{MapIcon, Position};
use crate::generation::error::GenerationError;
use crate::generation::rect::{draw_border, draw_rect};
use crate::generation::text::generate_text_block;
use crate::util::color::fill_image_color;
use image::DynamicImage;

pub fn generate_map_icon(
    height: u32,
    width: u32,
    args: &MapIcon,
) -> Result<DynamicImage, GenerationError> {
    let MapIcon {
        base_color,
        text,
        text_color,
        text_position,
        text_alignment,
        inner_border,
        outer_border,
        ..
    } = args;
    let mut image = DynamicImage::new_rgba8(width, height);
    draw_rect(&mut image, 0, 0, width, height, *base_color);
    // draw the text block

    if let Some(text) = text {
        let mut text_image = generate_text_block(text, *text_alignment);
        if text_image.width() > (width - 4) {
            return Err(GenerationError::TextTooLong(text.clone(), (width - 4) / 4));
        }
        if text_image.height() > (height - 4) {
            return Err(GenerationError::TooManyLines(
                text_image.height(),
                (height - 4) / 6,
            ));
        }
        fill_image_color(&mut text_image, *text_color);
        let text_width = text_image.width();
        let text_height = text_image.height();
        let (text_x, text_y) = match text_position {
            Position::TopLeft => (3, 3),
            Position::TopRight => (width - text_width - 3, 3),
            Position::BottomLeft => (3, height - text_height - 3),
            Position::BottomRight => (width - text_width - 3, height - text_height - 3),
            Position::Center => ((width - text_width) / 2, (height - text_height) / 2),
        };
        image::imageops::overlay(&mut image, &text_image, text_x as i64, text_y as i64);
    }

    // outer border
    if let Some(border) = outer_border {
        draw_border(&mut image, 0, 0, width, height, *border);
    }
    // inner border
    if let Some(border) = inner_border {
        draw_border(&mut image, 1, 1, width - 2, height - 2, *border);
    }
    Ok(image)
}

#[cfg(test)]
mod test {}
