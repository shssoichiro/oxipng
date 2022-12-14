use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;
use bitvec::prelude::*;

const ONE_BIT_PERMUTATIONS: [u8; 2] = [0b0000_0000, 0b1111_1111];
const TWO_BIT_PERMUTATIONS: [u8; 4] = [0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111];
const FOUR_BIT_PERMUTATIONS: [u8; 16] = [
    0b0000_0000,
    0b0001_0001,
    0b0010_0010,
    0b0011_0011,
    0b0100_0100,
    0b0101_0101,
    0b0110_0110,
    0b0111_0111,
    0b1000_1000,
    0b1001_1001,
    0b1010_1010,
    0b1011_1011,
    0b1100_1100,
    0b1101_1101,
    0b1110_1110,
    0b1111_1111,
];

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
    if minimum_bits >= bit_depth {
        return None;
    }
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
        for &byte in &png.data {
            while minimum_bits < bit_depth {
                let permutations: &[u8] = if minimum_bits == 1 {
                    &ONE_BIT_PERMUTATIONS
                } else if minimum_bits == 2 {
                    &TWO_BIT_PERMUTATIONS
                } else if minimum_bits == 4 {
                    &FOUR_BIT_PERMUTATIONS
                } else {
                    return None;
                };
                if permutations.iter().any(|perm| *perm == byte) {
                    break;
                }
                minimum_bits <<= 1;
            }
        }
    }

    let mut reduced = BitVec::<u8, Msb0>::with_capacity(png.data.len() * 8);
    for line in png.scan_lines(false) {
        let bit_vec = line.data.view_bits::<Msb0>();
        for (i, bit) in bit_vec.iter().by_vals().enumerate() {
            let bit_index = bit_depth - (i % bit_depth);
            if bit_index <= minimum_bits {
                reduced.push(bit);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
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
        data: reduced.as_raw_slice().to_vec(),
        ihdr: IhdrData {
            bit_depth: BitDepth::from_u8(minimum_bits as u8),
            ..png.ihdr
        },
        aux_headers: png.aux_headers.clone(),
        palette: png.palette.clone(),
        transparency_pixel,
    })
}
