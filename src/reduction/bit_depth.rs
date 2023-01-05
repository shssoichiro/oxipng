use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;

/// Attempt to reduce the bit depth of the image
/// Returns true if the bit depth was reduced, false otherwise
#[must_use]
pub fn reduce_bit_depth(png: &PngImage, minimum_bits: usize) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Sixteen {
        if png.ihdr.color_type == ColorType::Indexed || png.ihdr.color_type == ColorType::Grayscale
        {
            return reduce_bit_depth_8_or_less(png, minimum_bits);
        }
        return None;
    }

    // Reduce from 16 to 8 bits per channel per pixel
    if png.data.chunks(2).any(|pair| pair[0] != pair[1]) {
        // Can't reduce
        return None;
    }

    Some(PngImage {
        data: png.data.iter().step_by(2).cloned().collect(),
        ihdr: IhdrData {
            bit_depth: BitDepth::Eight,
            ..png.ihdr
        },
        palette: None,
        transparency_pixel: png.transparency_pixel.clone(),
        aux_headers: png.aux_headers.clone(),
    })
}

#[must_use]
pub fn reduce_bit_depth_8_or_less(png: &PngImage, mut minimum_bits: usize) -> Option<PngImage> {
    assert!((1..8).contains(&minimum_bits));
    let bit_depth: usize = png.ihdr.bit_depth.as_u8() as usize;
    if minimum_bits >= bit_depth || bit_depth > 8 {
        return None;
    }
    // Calculate the current number of pixels per byte
    let ppb = 8 / bit_depth;

    if png.ihdr.color_type == ColorType::Indexed {
        for line in png.scan_lines(false) {
            let line_max = line
                .data
                .iter()
                .map(|&byte| match png.ihdr.bit_depth {
                    BitDepth::Two => (byte & 0x3)
                        .max((byte >> 2) & 0x3)
                        .max((byte >> 4) & 0x3)
                        .max(byte >> 6),
                    BitDepth::Four => (byte & 0xF).max(byte >> 4),
                    _ => byte,
                })
                .max()
                .unwrap_or(0);
            let required_bits = match line_max {
                x if x > 0x0F => 8,
                x if x > 0x03 => 4,
                x if x > 0x01 => 2,
                _ => 1,
            };
            if required_bits > minimum_bits {
                minimum_bits = required_bits;
                if minimum_bits >= bit_depth {
                    // Not reducable
                    return None;
                }
            }
        }
    } else {
        // Checking for grayscale depth reduction is quite different than for indexed
        let mut mask = (1 << minimum_bits) - 1;
        let mut divisions = 1..(bit_depth / minimum_bits);
        for &b in &png.data {
            if b == 0 || b == 255 {
                continue;
            }
            'try_depth: loop {
                let mut byte = b;
                // Loop over each pixel in the byte
                for _ in 0..ppb {
                    // Align the first pixel division with the mask
                    byte = byte.rotate_left(minimum_bits as u32);
                    // Each potential division of this pixel must be identical to successfully reduce
                    let compare = byte & mask;
                    for _ in divisions.clone() {
                        // Align the next division with the mask
                        byte = byte.rotate_left(minimum_bits as u32);
                        if byte & mask != compare {
                            // This depth is not possible, try the next one up
                            minimum_bits <<= 1;
                            if minimum_bits == bit_depth {
                                return None;
                            }
                            mask = (1 << minimum_bits) - 1;
                            divisions = 1..(bit_depth / minimum_bits);
                            continue 'try_depth;
                        }
                    }
                }
                break;
            }
        }
    }

    let mut reduced = Vec::with_capacity(png.data.len());
    let mask = (1 << minimum_bits) - 1;
    for line in png.scan_lines(false) {
        // Loop over the data in chunks that will produce 1 byte of output
        for chunk in line.data.chunks(bit_depth / minimum_bits) {
            let mut new_byte = 0;
            let mut shift = 8;
            for &(mut byte) in chunk {
                // Loop over each pixel in the byte
                for _ in 0..ppb {
                    // Align the current pixel with the mask
                    byte = byte.rotate_left(bit_depth as u32);
                    shift -= minimum_bits;
                    // Take the low bits of the pixel and shift them into the output byte
                    new_byte |= (byte & mask) << shift;
                }
            }
            reduced.push(new_byte);
        }
    }

    // If the image is grayscale we also need to reduce the transparency pixel
    let mut transparency_pixel = png
        .transparency_pixel
        .clone()
        .filter(|t| png.ihdr.color_type == ColorType::Grayscale && t.len() >= 2);
    if let Some(trans) = transparency_pixel {
        let reduced_trans = trans[1] >> (bit_depth - minimum_bits);
        // Verify the reduction is valid by restoring back to original bit depth
        let mut check = reduced_trans;
        let mut bits = minimum_bits;
        while bits < bit_depth {
            check = check << bits | check;
            bits <<= 1;
        }
        if trans[0] == 0 && trans[1] == check {
            transparency_pixel = Some(vec![0, reduced_trans]);
        } else {
            // The transparency doesn't fit the new bit depth and is therefore unused - set it to None
            transparency_pixel = None;
        }
    }

    Some(PngImage {
        data: reduced,
        ihdr: IhdrData {
            bit_depth: BitDepth::from_u8(minimum_bits as u8),
            ..png.ihdr
        },
        aux_headers: png.aux_headers.clone(),
        palette: png.palette.clone(),
        transparency_pixel,
    })
}
