use super::PngData;

#[derive(Debug, Clone)]
/// An iterator over the scan lines of a PNG image
pub struct ScanLines<'a> {
    /// A reference to the PNG image being iterated upon
    pub png: &'a PngData,
    pub start: usize,
    pub end: usize,
    /// Current pass number, and 0-indexed row within the pass
    pub pass: Option<(u8, u32)>,
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end == self.png.raw_data.len() {
            None
        } else if self.png.ihdr_data.interlaced == 1 {
            // Scanlines for interlaced PNG files
            if self.pass.is_none() {
                self.pass = Some((1, 0));
            }
            // Handle edge cases for images smaller than 5 pixels in either direction
            if self.png.ihdr_data.width < 5 && self.pass.unwrap().0 == 2 {
                if let Some(pass) = self.pass.as_mut() {
                    pass.0 = 3;
                    pass.1 = 4;
                }
            }
            // Intentionally keep these separate so that they can be applied one after another
            if self.png.ihdr_data.height < 5 && self.pass.unwrap().0 == 3 {
                if let Some(pass) = self.pass.as_mut() {
                    pass.0 = 4;
                    pass.1 = 0;
                }
            }
            let bits_per_pixel = u32::from(self.png.ihdr_data.bit_depth.as_u8())
                * u32::from(self.png.channels_per_pixel());
            let y_steps;
            let pixels_factor;
            match self.pass {
                Some((1, _)) | Some((2, _)) => {
                    pixels_factor = 8;
                    y_steps = 8;
                }
                Some((3, _)) => {
                    pixels_factor = 4;
                    y_steps = 8;
                }
                Some((4, _)) => {
                    pixels_factor = 4;
                    y_steps = 4;
                }
                Some((5, _)) => {
                    pixels_factor = 2;
                    y_steps = 4;
                }
                Some((6, _)) => {
                    pixels_factor = 2;
                    y_steps = 2;
                }
                Some((7, _)) => {
                    pixels_factor = 1;
                    y_steps = 2;
                }
                _ => unreachable!(),
            }
            let mut pixels_per_line = self.png.ihdr_data.width / pixels_factor as u32;
            // Determine whether to add pixels if there is a final, incomplete 8x8 block
            let gap = self.png.ihdr_data.width % pixels_factor;
            if gap > 0 {
                match self.pass.unwrap().0 {
                    1 | 3 | 5 => {
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
            }
            let current_pass = if let Some(pass) = self.pass {
                Some(pass.0)
            } else {
                None
            };
            let bytes_per_line = ((pixels_per_line * bits_per_pixel + 7) / 8) as usize;
            self.start = self.end;
            self.end = self.start + bytes_per_line + 1;
            if let Some(pass) = self.pass.as_mut() {
                if pass.1 + y_steps >= self.png.ihdr_data.height {
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
            }
            Some(ScanLine {
                filter: self.png.raw_data[self.start],
                data: self.png.raw_data[(self.start + 1)..self.end].to_owned(),
                pass: current_pass,
            })
        } else {
            // Standard, non-interlaced PNG scanlines
            let bits_per_line = self.png.ihdr_data.width as usize
                * self.png.ihdr_data.bit_depth.as_u8() as usize
                * self.png.channels_per_pixel() as usize;
            let bytes_per_line = (bits_per_line + 7) / 8 as usize;
            self.start = self.end;
            self.end = self.start + bytes_per_line + 1;
            Some(ScanLine {
                filter: self.png.raw_data[self.start],
                data: self.png.raw_data[(self.start + 1)..self.end].to_owned(),
                pass: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
/// A scan line in a PNG image
pub struct ScanLine {
    /// The filter type used to encode the current scan line (0-4)
    pub filter: u8,
    /// The byte data for the current scan line, encoded with the filter specified in the `filter` field
    pub data: Vec<u8>,
    /// The current pass if the image is interlaced
    pub pass: Option<u8>,
}
