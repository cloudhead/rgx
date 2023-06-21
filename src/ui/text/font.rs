use std::array::TryFromSliceError;
use std::fmt;
use thiserror::Error;

use crate::gfx::pixels::PixelsMut;
use crate::gfx::*;
use crate::math::*;
use crate::ui::TextureId;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid font")]
    TryFromSlice(#[from] TryFromSliceError),
    #[error("Invalid tile count `{0}`, expected `{1}`")]
    TileCount(usize, usize),
    #[error("Invalid font byte length '{0}'")]
    ByteLength(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum FontFormat {
    UF1,
    UF2,
}

impl FontFormat {
    pub fn size(&self) -> Size<f32> {
        match self {
            Self::UF1 => Size::from(8.),
            Self::UF2 => Size::from(16.),
        }
    }
}

/// Bitmap font.
#[derive(Debug, Clone, Copy)]
pub struct Font {
    /// Glyph widths.
    pub widths: [u8; 256],
    /// Font texture.
    pub texture_id: TextureId,
    /// Font glyph size.
    pub tile: Size<f32>,
}

impl Font {
    /// Decode a font from a byte slice.
    pub fn decode(bytes: &[u8], format: FontFormat) -> Result<(Image, [u8; 256]), Error> {
        // Tile width and height. For UF2 fonts, each glyph is represented by four tiles.
        const T: usize = 8;
        // Glyph count. Represents the ASCII range.
        const N: usize = 256;
        // Number of tiles per glyph.
        const G: usize = 2 * 2;

        assert!(matches!(format, FontFormat::UF2));

        let (widths, glyphs) = bytes.split_at(N);
        let (head, tiles, tail) = unsafe { glyphs.align_to::<[u8; T]>() };

        if !head.is_empty() || !tail.is_empty() {
            return Err(Error::ByteLength(glyphs.len()));
        }
        if tiles.len() != N * G {
            return Err(Error::TileCount(tiles.len(), N * G));
        }

        // Rasterize the font into a 256x256 texture.
        let size = Size::new(N, N);
        let widths: [u8; N] = widths.try_into()?;
        let mut texels = vec![Rgba8::ZERO; size.area()];
        let mut pixels = PixelsMut::new(&mut texels, size.w, size.h);

        // Each glyph is a 2x2 grid of tiles encoded in the following order:
        //
        //   0 2
        //   1 3
        //
        // We loop through the tiles in chunks of 2, where each iteration renders half a glyph
        // into the texture.
        //
        let v = Rgba8::WHITE;
        let (mut x, mut y) = (0, 0);

        for window in tiles.chunks(G / 2) {
            if let &[a, b] = window {
                pixels.icn(a, x, y, v);
                pixels.icn(b, x, y + T, v);

                x += T;

                if x == size.w {
                    x = 0;
                    y += T + T;
                }
            }
        }

        Ok((Image::new(texels, size), widths))
    }

    pub fn glyph_width(&self, c: u8) -> f32 {
        self.widths[c as usize] as f32
    }

    pub fn text_width(&self, text: &str) -> f32 {
        text.bytes().map(|c| self.glyph_width(c)).sum()
    }

    pub fn text_height(&self) -> f32 {
        FontFormat::UF2.size().h
    }
}

/// Identifies a font.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FontId(pub String);

impl fmt::Display for FontId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for FontId {
    fn default() -> Self {
        Self(String::from("default"))
    }
}

impl From<&str> for FontId {
    fn from(other: &str) -> Self {
        Self(other.to_owned())
    }
}

impl From<String> for FontId {
    fn from(other: String) -> Self {
        Self(other)
    }
}
