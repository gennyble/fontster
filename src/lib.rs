mod color;
mod image;

pub use fontdue::Font;
use std::fs;

pub use color::Color;
use image::{Colors, Image};

#[derive(Debug, Clone, Copy, Default)]
pub struct Settings {
    pub font_size: f32,
    pub padding: usize,
    pub text_color: Color,
    pub background_color: Color,
    pub draw_baseline: bool,
    pub draw_glyph_outline: bool,
    pub draw_sentence_outline: bool,
}

pub fn get_font() -> Font {
    let font_data = include_bytes!("../Cabin-Regular.ttf");
    Font::from_bytes(font_data.as_ref(), Default::default()).expect("Failed to parse font")
}

pub fn get_font_italic() -> Font {
    let font_data = include_bytes!("../Cabin-Italic-VariableFont_wdth,wght.ttf");
    Font::from_bytes(font_data.as_ref(), Default::default()).expect("Failed to parse font")
}

struct Layout {
    pub glyphs: Vec<(isize, isize, usize, usize, Vec<u8>)>,
    pub width: usize,
    pub height: usize,
    pub baseline_offset: usize,
}

fn get_layout(font: &Font, size_px: f32, sentence: &str) -> Layout {
    let mut width = 0.0;
    let mut height = 0;
    let mut baseline_bottom_offset = 0;

    for ch in sentence.chars() {
        let metrics = font.metrics(ch, size_px);
        width += metrics.advance_width;

        if metrics.ymin >= 0 {
            let needed_height = metrics.height + metrics.ymin as usize;

            if height < needed_height {
                height = needed_height;
            }
        } else {
            let above_baseline = metrics.height - metrics.ymin.abs() as usize;
            let below_baseline = metrics.ymin.abs() as usize;

            if baseline_bottom_offset < below_baseline {
                // Add the difference in baselines
                height += below_baseline - baseline_bottom_offset;
                // Set the new baseline
                baseline_bottom_offset = below_baseline
            }

            if (height - baseline_bottom_offset) < above_baseline {
                height = above_baseline + baseline_bottom_offset;
            }
        }
    }

    let mut glyphs = Vec::with_capacity(sentence.len());
    let mut x_offset = 0.0;
    for ch in sentence.chars() {
        let (metrics, raster) = font.rasterize(ch, size_px);

        glyphs.push((
            metrics.xmin as isize + x_offset as isize,
            (height as isize - metrics.height as isize) + (metrics.ymin as isize * -1)
                - baseline_bottom_offset as isize,
            metrics.width,
            metrics.height,
            raster,
        ));

        x_offset += metrics.advance_width;
    }

    Layout {
        glyphs,
        width: width as usize, // Cast to usize now to avod compounding truncationd
        height,
        baseline_offset: baseline_bottom_offset,
    }
}

pub fn do_sentence(font: &Font, sentence: &str, settings: Settings) -> Image {
    let border_width = settings.padding;
    let layout = get_layout(font, settings.font_size, sentence);

    let img_width = layout.width + (border_width * 2);
    let img_height = layout.height + (border_width * 2);

    let mut img = Image::with_color(img_width, img_height, settings.background_color);

    if settings.draw_baseline {
        img.horizontal_line(
            border_width,
            border_width + (layout.height - layout.baseline_offset),
            layout.width,
            (255, 0, 0).into(),
        );
    }

    if settings.draw_sentence_outline {
        img.rect(
            border_width - 1,
            border_width - 1,
            layout.width + 2,
            layout.height + 2,
            (0, 0, 255).into(),
        );
    }

    for (mut x, mut y, width, height, raster) in layout.glyphs {
        x += border_width as isize;
        y += border_width as isize;

        img.draw_img(
            Image::from_buffer(
                width,
                height,
                raster,
                Colors::GreyAsAlpha(settings.text_color),
            ),
            x,
            y,
        );

        if settings.draw_glyph_outline {
            img.rect(x as usize, y as usize, width, height, (0, 255, 0).into());
        }
    }

    img
}
