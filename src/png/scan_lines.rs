use crate::interlace::Interlacing;
use crate::png::PngImage;

/// An iterator over the scan lines of a PNG image
#[derive(Debug, Clone)]
pub struct ScanLines<'a> {
    iter: ScanLineRanges,
    /// A reference to the PNG image being iterated upon
    raw_data: &'a [u8],
}

impl<'a> ScanLines<'a> {
    pub fn new(png: &'a PngImage) -> Self {
        Self {
            iter: ScanLineRanges::new(png),
            raw_data: &png.data,
        }
    }
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(len, pass)| {
            let (data, rest) = self.raw_data.split_at(len);
            self.raw_data = rest;
            let (&filter, data) = data.split_first().unwrap();
            ScanLine { filter, data, pass }
        })
    }
}

#[derive(Debug)]
/// An iterator over the scan lines of a PNG image
pub struct ScanLinesMut<'a> {
    iter: ScanLineRanges,
    /// A reference to the PNG image being iterated upon
    raw_data: Option<&'a mut [u8]>,
}

impl<'a> ScanLinesMut<'a> {
    pub fn new(png: &'a mut PngImage) -> Self {
        Self {
            iter: ScanLineRanges::new(png),
            raw_data: Some(&mut png.data),
        }
    }
}

impl<'a> Iterator for ScanLinesMut<'a> {
    type Item = ScanLineMut<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(len, pass)| {
            let tmp = self.raw_data.take().unwrap();
            let (data, rest) = tmp.split_at_mut(len);
            self.raw_data = Some(rest);
            let (&mut filter, data) = data.split_first_mut().unwrap();
            ScanLineMut { filter, data, pass }
        })
    }
}

#[derive(Debug, Clone)]
/// An iterator over the scan line locations of a PNG image
struct ScanLineRanges {
    /// Current pass number, and 0-indexed row within the pass
    pass: Option<(u8, u32)>,
    bits_per_pixel: u8,
    width: u32,
    height: u32,
    left: usize,
}

impl ScanLineRanges {
    pub fn new(png: &PngImage) -> Self {
        Self {
            bits_per_pixel: png.ihdr.bit_depth.as_u8() * png.channels_per_pixel(),
            width: png.ihdr.width,
            height: png.ihdr.height,
            left: png.data.len(),
            pass: if png.ihdr.interlaced == Interlacing::Adam7 {
                Some((1, 0))
            } else {
                None
            },
        }
    }
}

impl Iterator for ScanLineRanges {
    type Item = (usize, Option<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.left == 0 {
            return None;
        }
        let (pixels_per_line, current_pass) = if let Some(ref mut pass) = self.pass {
            // Scanlines for interlaced PNG files
            // Handle edge cases for images smaller than 5 pixels in either direction
            // No extra case needed for skipping pass 7 as this is already handled by the
            // self.left == 0 check above
            if self.width < 5 && pass.0 == 2 {
                pass.0 = 3;
                pass.1 = 4;
            }
            // Intentionally keep these separate so that they can be applied one after another
            if self.height < 5 && pass.0 == 3 {
                pass.0 = 4;
                pass.1 = 0;
            }
            if self.width < 3 && pass.0 == 4 {
                pass.0 = 5;
                pass.1 = 2;
            }
            if self.height < 3 && pass.0 == 5 {
                pass.0 = 6;
                pass.1 = 0;
            }
            if self.width == 1 && pass.0 == 6 {
                pass.0 = 7;
                pass.1 = 1;
            }
            let (pixels_factor, y_steps) = match pass.0 {
                1 | 2 => (8, 8),
                3 => (4, 8),
                4 => (4, 4),
                5 => (2, 4),
                6 => (2, 2),
                7 => (1, 2),
                _ => unreachable!(),
            };
            let mut pixels_per_line = self.width / pixels_factor;
            // Determine whether to add pixels if there is a final, incomplete 8x8 block
            let gap = self.width % pixels_factor;
            match pass.0 {
                1 | 3 | 5 if gap > 0 => {
                    pixels_per_line += 1;
                }
                2 if gap >= 5 => {
                    pixels_per_line += 1;
                }
                4 if gap >= 3 => {
                    pixels_per_line += 1;
                }
                6 if gap >= 2 => {
                    pixels_per_line += 1;
                }
                _ => (),
            };
            let current_pass = Some(pass.0);
            if pass.1 + y_steps >= self.height {
                pass.0 += 1;
                pass.1 = match pass.0 {
                    3 => 4,
                    5 => 2,
                    7 => 1,
                    _ => 0,
                };
            } else {
                pass.1 += y_steps;
            }
            (pixels_per_line, current_pass)
        } else {
            // Standard, non-interlaced PNG scanlines
            (self.width, None)
        };
        let bits_per_line = pixels_per_line * u32::from(self.bits_per_pixel);
        let bytes_per_line = ((bits_per_line + 7) / 8) as usize;
        let len = bytes_per_line + 1;
        self.left = self.left.checked_sub(len)?;
        Some((len, current_pass))
    }
}

#[derive(Debug, Clone)]
/// A scan line in a PNG image
pub struct ScanLine<'a> {
    /// The filter type used to encode the current scan line (0-4)
    pub filter: u8,
    /// The byte data for the current scan line, encoded with the filter specified in the `filter` field
    pub data: &'a [u8],
    /// The current pass if the image is interlaced
    pub pass: Option<u8>,
}

#[derive(Debug)]
/// A scan line in a PNG image
pub struct ScanLineMut<'a> {
    /// The filter type used to encode the current scan line (0-4)
    pub filter: u8,
    /// The byte data for the current scan line, encoded with the filter specified in the `filter` field
    pub data: &'a mut [u8],
    /// The current pass if the image is interlaced
    pub pass: Option<u8>,
}
