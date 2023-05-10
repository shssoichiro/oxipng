use rgb::{RGB16, RGBA8};
use std::{fmt, fmt::Display};

use crate::PngError;

#[derive(Debug, PartialEq, Eq, Clone)]
/// The color type used to represent this image
pub enum ColorType {
    /// Grayscale, with one color channel
    Grayscale {
        /// Optional shade of gray that should be rendered as transparent
        transparent_shade: Option<u16>,
    },
    /// RGB, with three color channels
    RGB {
        /// Optional color value that should be rendered as transparent
        transparent_color: Option<RGB16>,
    },
    /// Indexed, with one byte per pixel representing a color from the palette
    Indexed {
        /// The palette containing the colors used, up to 256 entries
        palette: Vec<RGBA8>,
    },
    /// Grayscale + Alpha, with two color channels
    GrayscaleAlpha,
    /// RGBA, with four color channels
    RGBA,
}

impl Display for ColorType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorType::Grayscale { .. } => Display::fmt("Grayscale", f),
            ColorType::RGB { .. } => Display::fmt("RGB", f),
            ColorType::Indexed { palette } => {
                Display::fmt(&format!("Indexed ({} colors)", palette.len()), f)
            }
            ColorType::GrayscaleAlpha => Display::fmt("Grayscale + Alpha", f),
            ColorType::RGBA => Display::fmt("RGB + Alpha", f),
        }
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
    pub(crate) fn channels_per_pixel(&self) -> u8 {
        match self {
            ColorType::Grayscale { .. } | ColorType::Indexed { .. } => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGB { .. } => 3,
            ColorType::RGBA => 4,
        }
    }

    #[inline]
    pub(crate) fn is_rgb(&self) -> bool {
        matches!(self, ColorType::RGB { .. } | ColorType::RGBA)
    }

    #[inline]
    pub(crate) fn is_grayscale(&self) -> bool {
        matches!(
            self,
            ColorType::Grayscale { .. } | ColorType::GrayscaleAlpha
        )
    }

    #[inline]
    pub(crate) fn has_alpha(&self) -> bool {
        matches!(self, ColorType::GrayscaleAlpha | ColorType::RGBA)
    }

    #[inline]
    pub(crate) fn has_trns(&self) -> bool {
        match self {
            ColorType::Grayscale { transparent_shade } => transparent_shade.is_some(),
            ColorType::RGB { transparent_color } => transparent_color.is_some(),
            _ => false,
        }
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

impl Display for BitDepth {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&(*self as u8).to_string(), f)
    }
}
