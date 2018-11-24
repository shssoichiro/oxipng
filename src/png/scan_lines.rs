use super::PngData;

#[derive(Debug, Clone)]
/// An iterator over the scan lines of a PNG image
pub struct ScanLines<'a> {
    /// A reference to the PNG image being iterated upon
    start: usize,
    /// Current pass number, and 0-indexed row within the pass
    pass: Option<(u8, u32)>,
    bits_per_pixel: u8,
    width: u32,
    height: u32,
    raw_data: &'a [u8],
}

impl<'a> ScanLines<'a> {
    pub fn new(png: &'a PngData) -> Self {
        Self {
            bits_per_pixel: png.ihdr_data.bit_depth.as_u8() * png.channels_per_pixel(),
            width: png.ihdr_data.width,
            height: png.ihdr_data.height,
            raw_data: &png.raw_data,
            start: 0,
            pass: if png.ihdr_data.interlaced == 1 {Some((1, 0))} else {None},
       }
    }
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.raw_data.len() {
            None
        } else if let Some(ref mut pass) = self.pass {
            // Scanlines for interlaced PNG files
            // Handle edge cases for images smaller than 5 pixels in either direction
            if self.width < 5 && pass.0 == 2 {
                pass.0 = 3;
                pass.1 = 4;
            }
            // Intentionally keep these separate so that they can be applied one after another
            if self.height < 5 && pass.0 == 3 {
                pass.0 = 4;
                pass.1 = 0;
            }
            let (pixels_factor, y_steps) = match pass {
                (1, _) | (2, _) => (8, 8),
                (3, _) => (4, 8),
                (4, _) => (4, 4),
                (5, _) => (2, 4),
                (6, _) => (2, 2),
                (7, _) => (1, 2),
                _ => unreachable!(),
            };
            let mut pixels_per_line = self.width / pixels_factor as u32;
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
            let bytes_per_line = ((pixels_per_line * self.bits_per_pixel as u32 + 7) / 8) as usize;
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
            let start = self.start;
            let len = bytes_per_line + 1;
            self.start += len;
            Some(ScanLine {
                filter: self.raw_data[start],
                data: &self.raw_data[(start + 1)..(start + len)],
                pass: current_pass,
            })
        } else {
            // Standard, non-interlaced PNG scanlines
            let bits_per_line = self.width * self.bits_per_pixel as u32;
            let bytes_per_line = ((bits_per_line + 7) / 8) as usize;
            let start = self.start;
            let len = bytes_per_line + 1;
            self.start += len;
            Some(ScanLine {
                filter: self.raw_data[start],
                data: &self.raw_data[(start + 1)..(start + len)],
                pass: None,
            })
        }
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
