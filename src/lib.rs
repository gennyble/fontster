mod layout;

pub use fontdue::Font;
pub use layout::{GlyphPosition, HorizontalAlign, Layout, LayoutSettings};

use std::{fs::File, io::Read};
use thiserror::Error;

/// Attempt to open the file at the given path and parse it as a font.
pub fn parse_font_file<P: AsRef<str>>(path: P) -> Result<Font, FontParseError> {
    let mut file = File::open(path.as_ref())?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    parse_font(&buffer)
}

/// Attempt to parse the given data as a font
pub fn parse_font(data: &[u8]) -> Result<Font, FontParseError> {
    Font::from_bytes(data, Default::default()).map_err(|s| FontParseError::ParseError(s))
}

#[derive(Debug, Error)]
pub enum FontParseError {
    #[error("failed to open font file")]
    FileError(#[from] std::io::Error),
    #[error("failed to parse font data: {0}")]
    ParseError(&'static str),
}
