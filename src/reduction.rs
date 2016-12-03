use bit_vec::BitVec;
use colors::BitDepth;
use itertools::Itertools;
use png::PngData;

pub fn reduce_bit_depth_8_or_less(png: &PngData) -> Option<(Vec<u8>, u8)> {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let mut allowed_bits = 1;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
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

    Some((reduced.to_bytes(), allowed_bits as u8))
}

pub fn reduce_rgba_to_rgb(png: &PngData) -> Option<Vec<u8>> {
    reduce_alpha_channel(png, 4)
}

pub fn reduce_rgba_to_grayscale_alpha(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth: u8 = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 4 * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut low_bytes = Vec::with_capacity(4);
        let mut high_bytes = Vec::with_capacity(4);
        let mut trans_bytes = Vec::with_capacity(byte_depth as usize);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp < colored_bytes {
                if byte_depth == 1 || i % 2 == 1 {
                    low_bytes.push(*byte);
                } else {
                    high_bytes.push(*byte);
                }
            } else {
                trans_bytes.push(*byte);
            }

            if i % bpp == bpp - 1 {
                if low_bytes.iter().unique().count() > 1 {
                    return None;
                }
                if byte_depth == 2 {
                    if high_bytes.iter().unique().count() > 1 {
                        return None;
                    }
                    reduced.push(high_bytes[0]);
                    high_bytes.clear();
                }
                reduced.push(low_bytes[0]);
                low_bytes.clear();
                reduced.extend_from_slice(&trans_bytes);
                trans_bytes.clear();
            }
        }
    }

    Some(reduced)
}

pub fn reduce_rgba_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = Vec::with_capacity(256);
    let bpp: usize = (4 * png.ihdr_data.bit_depth.as_u8() as usize) >> 3;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if let Some(idx) = palette.iter().position(|x| x == &cur_pixel) {
                    reduced.push(idx as u8);
                } else {
                    let len = palette.len();
                    if len == 256 {
                        return None;
                    }
                    palette.push(cur_pixel);
                    reduced.push(len as u8);
                }
                cur_pixel = Vec::with_capacity(bpp);
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    let mut trans_palette = Vec::with_capacity(palette.len());
    for color in &palette {
        for (i, byte) in color.iter().enumerate() {
            if i < 3 {
                color_palette.push(*byte);
            } else {
                trans_palette.push(*byte);
            }
        }
    }

    Some((reduced, color_palette, trans_palette))
}

pub fn reduce_rgb_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = Vec::with_capacity(256);
    let bpp: usize = (3 * png.ihdr_data.bit_depth.as_u8() as usize) >> 3;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if let Some(idx) = palette.iter().position(|x| x == &cur_pixel) {
                    reduced.push(idx as u8);
                } else {
                    let len = palette.len();
                    if len == 256 {
                        return None;
                    }
                    palette.push(cur_pixel);
                    reduced.push(len as u8);
                }
                cur_pixel = Vec::with_capacity(bpp);
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    for color in &palette {
        color_palette.extend_from_slice(color);
    }

    Some((reduced, color_palette))
}

pub fn reduce_grayscale_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth == BitDepth::Sixteen {
        return None;
    }
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    // Only perform reduction if we can get to 4-bits or less
    let mut palette = Vec::with_capacity(16);
    let bpp: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let bpp_inverse = 8 - bpp;
    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        let mut cur_pixel = BitVec::with_capacity(bpp);
        for (i, bit) in bit_vec.iter().enumerate() {
            cur_pixel.push(bit);
            if i % bpp == bpp - 1 {
                let pix_value = cur_pixel.to_bytes()[0] >> bpp_inverse;
                let pix_slice = vec![pix_value, pix_value, pix_value];
                if palette.contains(&pix_slice) {
                    let index = palette.iter().enumerate().find(|&x| x.1 == &pix_slice).unwrap().0;
                    let idx = BitVec::from_bytes(&[(index as u8) << bpp_inverse]);
                    for b in idx.iter().take(bpp) {
                        reduced.push(b);
                    }
                } else {
                    let len = palette.len();
                    if len == 16 {
                        return None;
                    }
                    palette.push(pix_slice);
                    let idx = BitVec::from_bytes(&[(len as u8) << bpp_inverse]);
                    for b in idx.iter().take(bpp) {
                        reduced.push(b);
                    }
                }
                cur_pixel = BitVec::with_capacity(bpp);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    for color in &palette {
        color_palette.extend_from_slice(color);
    }

    Some((reduced.to_bytes(), color_palette))
}

pub fn reduce_rgb_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth: u8 = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 3 * byte_depth as usize;
    let mut cur_pixel = Vec::with_capacity(bpp);
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if bpp == 3 {
                    if cur_pixel.iter().unique().count() > 1 {
                        return None;
                    }
                    reduced.push(cur_pixel[0]);
                } else {
                    let pixel_bytes = cur_pixel.iter()
                        .step(2)
                        .cloned()
                        .zip(cur_pixel.iter()
                            .skip(1)
                            .step(2)
                            .cloned())
                        .unique()
                        .collect::<Vec<(u8, u8)>>();
                    if pixel_bytes.len() > 1 {
                        return None;
                    }
                    reduced.push(pixel_bytes[0].0);
                    reduced.push(pixel_bytes[0].1);
                }
                cur_pixel.clear();
            }
        }
    }

    Some(reduced)
}

pub fn reduce_grayscale_alpha_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    reduce_alpha_channel(png, 2)
}

fn reduce_alpha_channel(png: &PngData, bpp_factor: usize) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth: u8 = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = bpp_factor * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp >= colored_bytes {
                if *byte != 255 {
                    return None;
                }
            } else {
                reduced.push(*byte);
            }
        }
    }

    Some(reduced)
}
