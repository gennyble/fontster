use fontdue::Font;
use png::{BitDepth, ColorType, Encoder};
use std::fs;

#[derive(Debug, PartialEq)]
enum Colors {
    RGB,
    Grey
}

struct Image {
    width: usize,
    height: usize,
    data: Vec<u8>
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        let data = vec![0; width * height * 3];

        Self {
            width,
            height,
            data
        }
    }

    fn from_buffer(width: usize, height: usize, mut data: Vec<u8>, colors: Colors) -> Self {
        let expected_len = match colors {
            Colors::Grey => width * height,
            Colors::RGB => width * height * 3
        };

        if data.len() != expected_len {
            panic!("Expected length to be {} but it's {}", expected_len, data.len());
        }

        if colors == Colors::Grey {
            // Not the fastest, but it'll do.
            let mut colordata = Vec::with_capacity(width * height * 3);
            for byte in data.into_iter() {
                colordata.extend_from_slice(&[byte, byte, byte]);
            }
            data = colordata;
        }

        Self {
            width,
            height,
            data
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn xy_to_index(&self, x: usize, y: usize) -> usize {
        (y as usize * self.width + x) * 3
    }

    fn draw_img(&mut self, img: Image, off_x: isize, off_y: isize, ignore_black: bool) {
        let img_data = img.data();
        for img_y in 0..(img.height() as isize) {
            // current pixel y value
            let y = off_y + img_y;

            if y < 0 {
                // Less than 0? Could still come into bounds
                continue;
            } else if y >= self.height as isize {
                // If the pixel Y is greater than the height, it's over
                return;
            }

            for img_x in 0..(img.width() as isize) {
                // Current pixel x value
                let x = off_x + img_x;

                if x < 0 {
                    continue;
                } if x >= self.width as isize{
                    break;
                } else {
                    let img_index = img.xy_to_index(img_x as usize, img_y as usize);
                    let our_index = self.xy_to_index(x as usize, y as usize);

                    if ignore_black && img_data[img_index] == 0 && img_data[img_index+1] == 0 && img_data[img_index] == 0 {
                        continue;
                    }

                    self.data[our_index] = img_data[img_index];
                    self.data[our_index+1] = img_data[img_index+1];
                    self.data[our_index+2] = img_data[img_index+2];
                }
            }
        }
    }

    fn horizontal_line(&mut self, x: usize, y: usize, len: usize, color: (u8, u8, u8)) {
        for i in 0..len {
            // TODO: Check x and y are valid coordiantes
            let index = self.xy_to_index(x + i, y);

            self.data[index] = color.0;
            self.data[index+1] = color.1;
            self.data[index+2] = color.2;
        }
    }

    fn vertical_line(&mut self, x: usize, y: usize, len: usize, color: (u8, u8, u8)) {
        for i in 0..len {
            // TODO: Check x and y are valid coordiantes
            let index = self.xy_to_index(x, y + i);

            self.data[index] = color.0;
            self.data[index+1] = color.1;
            self.data[index+2] = color.2;
        }
    }

    fn rect(&mut self, x1: usize, y1: usize, width: usize, height: usize, color: (u8, u8, u8)) {
        self.vertical_line(x1, y1, height, color); //Right
        self.horizontal_line(x1, y1, width, color); //Top
        self.vertical_line(x1 + width, y1, height, color); //Left
        self.horizontal_line(x1, y1 + height, width, color); //Bottom
    }
}

fn main() {
    let data = fs::read("Cabin-Regular.ttf").expect("Failed to load font from file");
    //let data = fs::read("FiraCode-Regular.ttf").unwrap();
    let font = Font::from_bytes(data, Default::default()).expect("Failed to parse font");

    let px = 64.0;

    // An 'em' referes to the width of M historically, as it was usually the
    // widest character (and took up all the available horizontal space)
    let em = font.metrics('M', px).bounds.width;

    let line_metrics = font.horizontal_line_metrics(px).expect("Is this not a vertical font?");
    // This should the largest height a glpyh can have. ascent is positive (above baseline)
    // and descent is negative (below baseline).
    let max_height = line_metrics.ascent - line_metrics.descent;

    // Width/height, in characters, of the image
    let char_width = 16;
    let char_height = 8;

    // Looks like we can't assume 'em' and maybe not even 'max_height'
    // is correct. We'll figure it out outselves
    let mut max_glyph_width = 0;
    let mut max_glyph_height = 0;
    
    // ASCII
    for index in 0..128u8 {
        let metrics = font.metrics(index as char, px);
        if max_glyph_width < metrics.width {
            max_glyph_width = metrics.width;
        }
        if max_glyph_height < metrics.height {
            max_glyph_height = metrics.height;
        }
    }

    println!("px is set to {}", px);
    println!("em was calculated to {}", em);
    println!("max_height was calculated to {}", max_height);
    println!("Max glyph dimensions:\n\twidth: {}\n\theight: {}", max_glyph_width, max_glyph_height);

    let mut img = Image::new((char_width as f32 * max_glyph_width as f32) as usize, (char_height as f32 * max_glyph_height as f32) as usize);

    // Add every character to the raster imge
    for index in 0..128u8 {
        let char_x = index % char_width;
        let char_y = index / char_width;

        let x = char_x as f32 * max_glyph_width as f32;
        let y = char_y as f32 * max_glyph_height as f32;

        let (metrics, bitmap) = font.rasterize(index as char, px);

        img.draw_img(
            Image::from_buffer(metrics.width, metrics.height, bitmap, Colors::Grey),
            x as isize,
            y as isize,
            true
        );
    }


    // Write out the raster image
    let png_file = fs::File::create("raster.png").expect("Failed to create raster image file");
    let width = img.width() as u32;
    let height = img.height() as u32;

    let mut png = Encoder::new(png_file, width, height);
    png.set_color(ColorType::RGB);
    png.set_depth(BitDepth::Eight);

    let mut writer = png.write_header().expect("Failed to write PNG header");
    writer.write_image_data(img.data()).expect("Failed to write PNG data");

    println!();
    do_sentence(&font, "EHLO, q256!", "ehloq256.png");
    do_sentence(&font, "Hello, World!", "hello_world.png");
    do_sentence(&font, "@Genuinebyte", "genuinebyte.png");
    do_sentence(&font, "Ligatures !=", "ligatures.png");
}

struct Layout {
    pub glyphs: Vec<(isize, isize, usize, usize, Vec<u8>)>,
    pub width: usize,
    pub height: usize,
    pub baseline_offset: usize
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

        glyphs.push(
            (
            metrics.xmin as isize + x_offset as isize,
            (height as isize - metrics.height as isize) + (metrics.ymin as isize * -1) - baseline_bottom_offset as isize,
            metrics.width,
            metrics.height,
            raster
            )
        );

        x_offset += metrics.advance_width;
    }

    Layout {
        glyphs,
        width: width as usize, // Cast to usize now to avod compounding truncationd
        height,
        baseline_offset: baseline_bottom_offset
    }
}

fn do_sentence(font: &Font, sentence: &str, fname: &str) {
    let px = 128.0;
    let border_width = px as usize/4;
    let layout = get_layout(font, px, sentence);

    let img_width = layout.width + (border_width * 2);
    let img_height = layout.height + (border_width * 2);

    let mut img = Image::new(img_width, img_height);

    // Draw the baseline
    img.horizontal_line(border_width, border_width + (layout.height - layout.baseline_offset), layout.width, (255, 0, 0));
    // Draw bounding box
    img.rect(border_width-1, border_width-1, layout.width + 2, layout.height + 2, (0, 0, 255));

    for (mut x, mut y, width, height, raster) in layout.glyphs {
        x += border_width as isize;
        y += border_width as isize;

        img.draw_img(
            Image::from_buffer(width, height, raster, Colors::Grey),
            x,
            y,
            true
        );
        img.rect(x as usize, y as usize, width, height, (0, 255, 0));
    }

    let png_file = fs::File::create(fname).expect("Failed to create sentence image file");

    let mut png = Encoder::new(png_file, img.width() as u32, img.height() as u32);
    png.set_color(ColorType::RGB);
    png.set_depth(BitDepth::Eight);

    let mut writer = png.write_header().expect("Failed to write PNG header");
    writer.write_image_data(img.data()).expect("Failed to write PNG data");
}