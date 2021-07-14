use fontdue::Font;

#[derive(Debug)]
pub struct GlyphPosition {
    pub c: char,
    pub x: f32,
    pub y: f32,
    pub width: usize,
    pub height: usize,
}

/// A line of text. All glyph positions are relative to the line until
/// Layout::glyphs
#[derive(Debug, Default)]
struct Line {
    width: f32,

    // A recommended space between the lines.
    // A font-recommended gap between the last lines descent and the next lines ascent.
    gap: f32,
    // The highest a glyph extends above the baseline
    ascent: f32,
    // The lowest a glyph descends below the baseline
    descent: f32,

    glyphs: Vec<GlyphPosition>,
}

#[derive(Debug, Default)]
pub struct Layout {
    lines: Vec<Line>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            lines: vec![Line::default()],
            ..Default::default()
        }
    }

    pub fn append(&mut self, font: &Font, font_size: f32, text: &str) {
        for ch in text.chars() {
            println!("{}", ch);
            if ch == '\n' {
                println!("starting new line");
                // Start a new line if we're told to
                self.lines.push(Line::default());
                continue;
            } else if ch.is_control() {
                // Ignore control characyers
                continue;
            }

            let metrics = font.metrics(ch, font_size);
            let line_metrics = font.horizontal_line_metrics(font_size).unwrap();

            // Our new method assues us we always have at least one line.
            let line = self.lines.last_mut().unwrap();

            // Set our line metrics to the max of any font used on that line
            line.gap = line.gap.max(line_metrics.line_gap);
            line.ascent = line.ascent.max(line_metrics.ascent);
            line.descent = line.descent.max(line_metrics.descent);

            // NOTE:
            // See how we're setting the y value to metrics.ymin? That's the
            // position of the BOTTOM of the bitmap relative to the baseline.
            // We don't set the proper y position of the glyph here because if
            // we mix fonts within the line the ascent and descent can change,
            // which would mess everything up.
            line.glyphs.push(GlyphPosition {
                c: ch,
                x: line.width,
                y: metrics.ymin as f32,
                width: metrics.width,
                height: metrics.height,
            });

            line.width += metrics.advance_width;
        }
    }

    pub fn width(&self) -> f32 {
        let mut width = 0.0;
        for line in &self.lines {
            width = line.width.max(width);
        }

        width
    }

    pub fn height(&self) -> f32 {
        let mut height = 0.0;
        for line in &self.lines {
            height += line.ascent + line.descent + line.gap;
        }

        height
    }

    pub fn glyphs(self) -> Vec<GlyphPosition> {
        let mut ret = vec![];

        let mut baseline = 0.0;
        for line in self.lines {
            baseline += line.ascent;

            for mut glyph in line.glyphs {
                // calculate the top-left corner y value of the glyph and then
                // move it to the baseline
                glyph.y = (glyph.y * -1.0 - glyph.height as f32) + baseline;
                ret.push(glyph);
            }

            baseline += line.descent + line.gap;
        }

        ret
    }
}
