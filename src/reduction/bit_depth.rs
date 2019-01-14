use headers::IhdrData;
use bit_vec::BitVec;
use colors::{BitDepth, ColorType};
use png::PngImage;

const ONE_BIT_PERMUTATIONS: [u8; 2] = [0b0000_0000, 0b1111_1111];
const TWO_BIT_PERMUTATIONS: [u8; 5] = [
    0b0000_0000,
    0b0000_1111,
    0b0011_1100,
    0b1111_0000,
    0b1111_1111,
];
const FOUR_BIT_PERMUTATIONS: [u8; 11] = [
    0b0000_0000,
    0b0000_0011,
    0b0000_1100,
    0b0011_0000,
    0b1100_0000,
    0b0000_1111,
    0b0011_1100,
    0b1111_0000,
    0b0011_1111,
    0b1111_1100,
    0b1111_1111,
];

/// Attempt to reduce the bit depth of the image
/// Returns true if the bit depth was reduced, false otherwise
#[must_use]
pub fn reduce_bit_depth(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Sixteen {
        if png.ihdr.color_type == ColorType::Indexed
            || png.ihdr.color_type == ColorType::Grayscale
        {
            return reduce_bit_depth_8_or_less(png);
        }
        return None;
    }

    // Reduce from 16 to 8 bits per channel per pixel
    let mut reduced = Vec::with_capacity(
        (png.ihdr.width * png.ihdr.height * u32::from(png.channels_per_pixel())
            + png.ihdr.height) as usize,
    );
    let mut high_byte = 0;

    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, &byte) in line.data.iter().enumerate() {
            if i % 2 == 0 {
                // High byte
                high_byte = byte;
            } else {
                // Low byte
                if high_byte != byte {
                    // Can't reduce, exit early
                    return None;
                }
                reduced.push(byte);
            }
        }
    }

    Some(PngImage {
        data: reduced,
        ihdr: IhdrData {
            bit_depth: BitDepth::Eight,
            ..png.ihdr
        },
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
        aux_headers: png.aux_headers.clone(),
    })
}

#[must_use]
pub fn reduce_bit_depth_8_or_less(png: &PngImage) -> Option<PngImage> {
    let mut reduced = BitVec::with_capacity(png.data.len() * 8);
    let bit_depth: usize = png.ihdr.bit_depth.as_u8() as usize;
    let mut allowed_bits = 1;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        if png.ihdr.color_type == ColorType::Indexed {
            for (i, bit) in bit_vec.iter().enumerate() {
                let bit_index = bit_depth - (i % bit_depth);
                if bit && bit_index > allowed_bits {
                    allowed_bits = bit_index.next_power_of_two();
                    if allowed_bits == bit_depth {
                        // Not reducable
                        return None;
                    }
                }
            }
        } else {
            for byte in bit_vec.to_bytes() {
                while allowed_bits < bit_depth {
                    let permutations: &[u8] = if allowed_bits == 1 {
                        &ONE_BIT_PERMUTATIONS
                    } else if allowed_bits == 2 {
                        &TWO_BIT_PERMUTATIONS
                    } else if allowed_bits == 4 {
                        &FOUR_BIT_PERMUTATIONS
                    } else {
                        return None;
                    };
                    if permutations.iter().any(|perm| *perm == byte) {
                        break;
                    } else {
                        allowed_bits <<= 1;
                    }
                }
            }
        }
    }

    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            let bit_index = bit_depth - (i % bit_depth);
            if bit_index <= allowed_bits {
                reduced.push(bit);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
        }
    }

    Some(PngImage {
        data: reduced.to_bytes(),
        ihdr: IhdrData {
            bit_depth: BitDepth::from_u8(allowed_bits as u8),
            ..png.ihdr
        },
        aux_headers: png.aux_headers.clone(),
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
    })
}
