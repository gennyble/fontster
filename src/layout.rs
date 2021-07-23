use fontdue::Font;

#[derive(Debug)]
pub struct GlyphPosition {
    pub c: char,
    pub x: f32,
    pub y: f32,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutSettings {
    pub horizontal_align: HorizontalAlign,
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

impl Default for HorizontalAlign {
    fn default() -> Self {
        HorizontalAlign::Left
    }
}

/// A line of text. All glyph positions are relative to the line until
/// Layout::glyphs
#[derive(Debug, Default)]
struct Line {
    width: f32,

    // A recommended space between the lines.
    // A font-recommended gap between the last lines descent and the next lines ascent.
    gap: f32,
    // The highest a glyph extends above the baseline, typically positive
    ascent: f32,
    // The lowest a glyph descends below the baseline, typically negative
    descent: f32,

    glyphs: Vec<GlyphPosition>,
}

impl Line {
    /// The height of this line including any gap
    pub fn height(&self) -> f32 {
        self.ascent - self.descent + self.gap
    }
}

pub struct StyledText<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub font_index: usize,
}

#[derive(Debug, Default)]
pub struct Layout {
    settings: LayoutSettings,
    lines: Vec<Line>,
}

impl Layout {
    pub fn new(settings: LayoutSettings) -> Self {
        Self {
            settings,
            lines: vec![Line::default()],
        }
    }

    pub fn append(&mut self, fonts: &[Font], styled: StyledText) {
        for ch in styled.text.chars() {
            if ch == '\n' {
                // Start a new line if we're told to
                self.lines.push(Line::default());
                continue;
            } else if ch.is_control() {
                // Ignore control characyers
                continue;
            }

            let font = &fonts[styled.font_index];
            let metrics = font.metrics(ch, styled.font_size);
            let line_metrics = font.horizontal_line_metrics(styled.font_size).unwrap();

            // Our new method assues us we always have at least one line.
            let line = self.lines.last_mut().unwrap();

            // Set our line metrics to the max of any font used on that line
            line.gap = line.gap.max(line_metrics.line_gap);
            line.ascent = line.ascent.max(line_metrics.ascent);
            line.descent = line.descent.min(line_metrics.descent);

            let kern = match line.glyphs.last() {
                Some(last) => font
                    .horizontal_kern(last.c, ch, styled.font_size)
                    .unwrap_or(0.0),
                None => 0.0,
            };

            // NOTE:
            // See how we're setting the y value to metrics.ymin? That's the
            // position of the BOTTOM of the bitmap relative to the baseline.
            // We don't set the proper y position of the glyph here because if
            // we mix fonts within the line the ascent and descent can change,
            // which would mess everything up.
            line.glyphs.push(GlyphPosition {
                c: ch,
                x: (kern + metrics.xmin as f32 + line.width).max(0.0) as f32,
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
            height += line.height();
        }

        height
    }

    pub fn glyphs(self) -> Vec<GlyphPosition> {
        let mut ret = vec![];
        let settings = self.settings;
        let width = self.width();

        let mut baseline = 0.0;
        for line in self.lines {
            baseline += line.ascent;

            let x_offset = match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => (width - line.width) / 2.0,
                HorizontalAlign::Right => width - line.width,
            };

            for mut glyph in line.glyphs {
                glyph.x += x_offset;
                // calculate the top-left corner y value of the glyph and then
                // move it to the baseline
                glyph.y = glyph.y * -1.0 + baseline - glyph.height as f32;
                ret.push(glyph);
            }

            baseline += -line.descent + line.gap;
        }

        ret
    }
}
