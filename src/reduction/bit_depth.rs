use bit_vec::BitVec;
use colors::{BitDepth, ColorType};
use png::PngData;

pub fn reduce_bit_depth_8_or_less(png: &mut PngData) -> bool {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let mut allowed_bits = 1;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            let bit_index = if png.ihdr_data.color_type == ColorType::Indexed {
                bit_depth - (i % bit_depth)
            } else {
                i % bit_depth
            };
            if bit && bit_index > allowed_bits {
                allowed_bits = bit_index.next_power_of_two();
                if allowed_bits == bit_depth {
                    // Not reducable
                    return false;
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

    png.raw_data = reduced.to_bytes();
    png.ihdr_data.bit_depth = BitDepth::from_u8(allowed_bits as u8);
    true
}
