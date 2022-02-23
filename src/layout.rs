use std::borrow::Borrow;

use fontdue::Font;

#[derive(Debug)]
pub struct GlyphPosition<U: Clone> {
    pub c: char,
    pub x: f32,
    pub y: f32,
    pub width: usize,
    pub height: usize,
    pub font_index: usize,
    pub font_size: f32,
    pub user: U,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutSettings {
    pub horizontal_align: HorizontalAlign,
    pub line_height: LineHeight,
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

/// The gap between lines
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineHeight {
    /// The ascent, descent, and gap between lines are taken form the font file.
    /// These are the recommended values from the font author.
    Font,
    /// Take the ascent and descent from the font file, but multiply them by the
    /// given ratio.
    Ratio(f32),
    /// Determine the ascent and descent by the maxima of the glyphs. The gap acts
    /// like ['Ratio'](LineHeight::Ratio) and will use the provided `f32`
    Smallest(f32),
}

impl Default for LineHeight {
    fn default() -> Self {
        LineHeight::Font
    }
}

/// A line of text. All glyph positions are relative to the line until
/// Layout::glyphs
#[derive(Debug)]
struct Line<U: Clone> {
    width: f32,

    // A recommended space between the lines.
    // A font-recommended gap between the last lines descent and the next lines ascent.
    gap: f32,
    // The highest a glyph extends above the baseline, typically positive
    ascent: f32,
    // The lowest a glyph descends below the baseline, typically negative
    descent: f32,

    glyphs: Vec<GlyphPosition<U>>,
}

impl<U: Clone> Line<U> {
    /// The height of this line including any gap
    pub fn height(&self) -> f32 {
        self.ascent - self.descent + self.gap
    }
}

impl<U: Clone> Default for Line<U> {
    fn default() -> Self {
        Self {
            width: 0.0,
            gap: 0.0,
            ascent: 0.0,
            descent: 0.0,
            glyphs: vec![],
        }
    }
}

pub struct StyledText<'a, U> {
    pub text: &'a str,
    pub font_size: f32,
    pub font_index: usize,
    pub user: U,
}

#[derive(Debug, Default)]
pub struct Layout<U: Clone> {
    settings: LayoutSettings,
    lines: Vec<Line<U>>,
}

impl<U: Clone> Layout<U> {
    pub fn new(settings: LayoutSettings) -> Self {
        Self {
            settings,
            lines: vec![Line::default()],
        }
    }

    pub fn append<F: Borrow<Font>>(&mut self, fonts: &[F], styled: StyledText<U>) {
        let font: &Font = fonts[styled.font_index].borrow();
        let line_metrics = font.horizontal_line_metrics(styled.font_size).unwrap();

        for ch in styled.text.chars() {
            // Our new method assures us we always have at least one line.
            let line = self.lines.last_mut().unwrap();

            if ch == '\n' {
                self.lines.push(Line::default());
                continue;
            } else if ch.is_control() {
                // Ignore control characters
                continue;
            }

            let metrics = font.metrics(ch, styled.font_size);

            if let LineHeight::Smallest(_) = self.settings.line_height {
                line.ascent = line.ascent.max(metrics.height as f32 + metrics.ymin as f32);
                line.descent = line.descent.min(metrics.ymin as f32);
            } else {
                // Set our line metrics to the max of any font used on that line
                line.ascent = line.ascent.max(line_metrics.ascent);
                line.descent = line.descent.min(line_metrics.descent);
            }

            line.gap = match self.settings.line_height {
                LineHeight::Font => line.gap.max(line_metrics.line_gap),
                LineHeight::Ratio(ratio) | LineHeight::Smallest(ratio) => {
                    let min = line.ascent + line.descent;
                    line.gap.max((min * ratio) - min)
                }
            };

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
                font_index: styled.font_index,
                font_size: styled.font_size,
                user: styled.user.clone(),
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
        let mut lastheight = 0.0;

        for line in &self.lines {
            if line.glyphs.len() == 0 {
                height += lastheight;
                continue;
            }

            height += line.height();
            lastheight = line.height();
        }

        height
    }

    pub fn glyphs(self) -> Vec<GlyphPosition<U>> {
        let mut ret = vec![];
        let settings = self.settings;
        let width = self.width();
        let mut lastheight = 0.0;

        let mut baseline = 0.0;
        for line in self.lines {
            let x_offset = match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => (width - line.width) / 2.0,
                HorizontalAlign::Right => width - line.width,
            };

            if line.glyphs.len() == 0 {
                baseline += lastheight;
                continue;
            }
            lastheight = line.height();

            baseline += line.ascent;

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
