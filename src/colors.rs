use rgb::{RGB16, RGBA8};
use std::fmt;

use crate::PngError;

#[derive(Debug, PartialEq, Eq, Clone)]
/// The color type used to represent this image
pub enum ColorType {
    /// Grayscale, with one color channel
    Grayscale { transparent: Option<u16> },
    /// RGB, with three color channels
    RGB { transparent: Option<RGB16> },
    /// Indexed, with one byte per pixel representing one of up to 256 colors in the image
    Indexed { palette: Vec<RGBA8> },
    /// Grayscale + Alpha, with two color channels
    GrayscaleAlpha,
    /// RGBA, with four color channels
    RGBA,
}

impl fmt::Display for ColorType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ColorType::Grayscale { .. } => "Grayscale",
                ColorType::RGB { .. } => "RGB",
                ColorType::Indexed { .. } => "Indexed",
                ColorType::GrayscaleAlpha => "Grayscale + Alpha",
                ColorType::RGBA => "RGB + Alpha",
            }
        )
    }
}

impl ColorType {
    /// Get the code used by the PNG specification to denote this color type
    #[inline]
    pub fn png_header_code(&self) -> u8 {
        match self {
            ColorType::Grayscale { .. } => 0,
            ColorType::RGB { .. } => 2,
            ColorType::Indexed { .. } => 3,
            ColorType::GrayscaleAlpha => 4,
            ColorType::RGBA => 6,
        }
    }

    #[inline]
    pub fn channels_per_pixel(&self) -> u8 {
        match self {
            ColorType::Grayscale { .. } | ColorType::Indexed { .. } => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGB { .. } => 3,
            ColorType::RGBA => 4,
        }
    }

    #[inline]
    pub fn is_rgb(&self) -> bool {
        matches!(self, ColorType::RGB { .. } | ColorType::RGBA)
    }

    #[inline]
    pub fn has_alpha(&self) -> bool {
        matches!(self, ColorType::GrayscaleAlpha | ColorType::RGBA)
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
/// The number of bits to be used per channel per pixel
pub enum BitDepth {
    /// One bit per channel per pixel
    One = 1,
    /// Two bits per channel per pixel
    Two = 2,
    /// Four bits per channel per pixel
    Four = 4,
    /// Eight bits per channel per pixel
    Eight = 8,
    /// Sixteen bits per channel per pixel
    Sixteen = 16,
}

impl TryFrom<u8> for BitDepth {
    type Error = PngError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            4 => Ok(Self::Four),
            8 => Ok(Self::Eight),
            16 => Ok(Self::Sixteen),
            _ => Err(PngError::new("Unexpected bit depth")),
        }
    }
}

impl fmt::Display for BitDepth {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
