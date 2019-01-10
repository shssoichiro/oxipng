use reduction::ReducedPng;
use bit_vec::BitVec;
use colors::{BitDepth, ColorType};
use png::PngData;

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

#[must_use]
pub fn reduce_bit_depth_8_or_less(png: &PngData) -> Option<ReducedPng> {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let mut allowed_bits = 1;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        if png.ihdr_data.color_type == ColorType::Indexed {
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

    Some(ReducedPng {
        color_type: png.ihdr_data.color_type,
        raw_data: reduced.to_bytes(),
        bit_depth: BitDepth::from_u8(allowed_bits as u8),
        aux_headers: Default::default(),
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
    })
}
