use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
/// The color type used to represent this image
pub enum ColorType {
    /// Grayscale, with one color channel
    Grayscale,
    /// RGB, with three color channels
    RGB,
    /// Indexed, with one byte per pixel representing one of up to 256 colors in the image
    Indexed,
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
                ColorType::Grayscale => "Grayscale",
                ColorType::RGB => "RGB",
                ColorType::Indexed => "Indexed",
                ColorType::GrayscaleAlpha => "Grayscale + Alpha",
                ColorType::RGBA => "RGB + Alpha",
            }
        )
    }
}

impl ColorType {
    /// Get the code used by the PNG specification to denote this color type
    #[inline]
    pub fn png_header_code(self) -> u8 {
        match self {
            ColorType::Grayscale => 0,
            ColorType::RGB => 2,
            ColorType::Indexed => 3,
            ColorType::GrayscaleAlpha => 4,
            ColorType::RGBA => 6,
        }
    }

    #[inline]
    pub fn channels_per_pixel(self) -> u8 {
        match self {
            ColorType::Grayscale | ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGB => 3,
            ColorType::RGBA => 4,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
/// The number of bits to be used per channel per pixel
pub enum BitDepth {
    /// One bit per channel per pixel
    One,
    /// Two bits per channel per pixel
    Two,
    /// Four bits per channel per pixel
    Four,
    /// Eight bits per channel per pixel
    Eight,
    /// Sixteen bits per channel per pixel
    Sixteen,
}

impl fmt::Display for BitDepth {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                BitDepth::One => "1",
                BitDepth::Two => "2",
                BitDepth::Four => "4",
                BitDepth::Eight => "8",
                BitDepth::Sixteen => "16",
            }
        )
    }
}

impl BitDepth {
    /// Retrieve the number of bits per channel per pixel as a `u8`
    #[inline]
    pub fn as_u8(self) -> u8 {
        match self {
            BitDepth::One => 1,
            BitDepth::Two => 2,
            BitDepth::Four => 4,
            BitDepth::Eight => 8,
            BitDepth::Sixteen => 16,
        }
    }
    /// Parse a number of bits per channel per pixel into a `BitDepth`
    #[inline]
    pub fn from_u8(depth: u8) -> BitDepth {
        match depth {
            1 => BitDepth::One,
            2 => BitDepth::Two,
            4 => BitDepth::Four,
            8 => BitDepth::Eight,
            16 => BitDepth::Sixteen,
            _ => panic!("Unsupported bit depth"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
/// Potential optimization methods for alpha channel
pub enum AlphaOptim {
    NoOp,
    Black,
    White,
    Up,
    Right,
    Down,
    Left,
}

impl fmt::Display for AlphaOptim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                AlphaOptim::NoOp => "_",
                AlphaOptim::Black => "B",
                AlphaOptim::White => "W",
                AlphaOptim::Up => "U",
                AlphaOptim::Right => "R",
                AlphaOptim::Down => "D",
                AlphaOptim::Left => "L",
            }
        )
    }
}
