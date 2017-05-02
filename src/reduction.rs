use bit_vec::BitVec;
use colors::{BitDepth, ColorType};
use itertools::Itertools;
use png::PngData;

pub fn reduce_bit_depth_8_or_less(png: &mut PngData) -> bool {
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

pub fn reduce_rgba_to_rgb(png: &mut PngData) -> bool {
    if let Some(reduced) = reduce_alpha_channel(png, 4) {
        png.raw_data = reduced;
        png.ihdr_data.color_type = ColorType::RGB;
        true
    } else {
        false
    }
}

pub fn reduce_rgba_to_grayscale_alpha(png: &mut PngData) -> bool {
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
                    return false;
                }
                if byte_depth == 2 {
                    if high_bytes.iter().unique().count() > 1 {
                        return false;
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

    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), 4);
        sbit_header.remove(1);
        sbit_header.remove(1);
    }
    if let Some(bkgd_header) = png.aux_headers.get_mut(&"bKGD".to_string()) {
        assert_eq!(bkgd_header.len(), 6);
        bkgd_header.truncate(2);
    }

    png.raw_data = reduced;
    png.ihdr_data.color_type = ColorType::GrayscaleAlpha;
    true
}

pub fn reduce_rgba_to_palette(png: &mut PngData) -> bool {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return false;
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
                        return false;
                    }
                    palette.push(cur_pixel);
                    reduced.push(len as u8);
                }
                cur_pixel = Vec::with_capacity(bpp);
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3 +
                                               if png.aux_headers
                                                      .contains_key(&"bKGD".to_string()) {
                                                   6
                                               } else {
                                                   0
                                               });
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

    let headers_size = color_palette.len() + trans_palette.len() + 8;
    if reduced.len() + headers_size > png.raw_data.len() * 4 {
        // Reduction would result in a larger image
        return false;
    }

    if let Some(bkgd_header) = png.aux_headers.get_mut(&"bKGD".to_string()) {
        assert_eq!(bkgd_header.len(), 6);
        let header_pixels = bkgd_header
            .iter()
            .skip(1)
            .step(2)
            .cloned()
            .collect::<Vec<u8>>();
        if let Some(entry) = color_palette
               .chunks(3)
               .position(|x| x == header_pixels.as_slice()) {
            *bkgd_header = vec![entry as u8];
        } else if color_palette.len() / 3 == 256 {
            return false;
        } else {
            let entry = color_palette.len() / 3;
            color_palette.extend_from_slice(&header_pixels);
            *bkgd_header = vec![entry as u8];
        }
    }
    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), 4);
        sbit_header.pop();
    }

    png.raw_data = reduced;
    png.palette = Some(color_palette);
    if trans_palette.iter().any(|x| *x != 255) {
        while let Some(255) = trans_palette.last().cloned() {
            trans_palette.pop();
        }
        png.transparency_palette = Some(trans_palette);
    } else {
        png.transparency_palette = None;
    }
    png.ihdr_data.color_type = ColorType::Indexed;
    true
}

pub fn reduce_rgb_to_palette(png: &mut PngData) -> bool {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return false;
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
                        return false;
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

    let headers_size = color_palette.len() + 4;
    if reduced.len() + headers_size > png.raw_data.len() * 3 {
        // Reduction would result in a larger image
        return false;
    }

    if let Some(bkgd_header) = png.aux_headers.get_mut(&"bKGD".to_string()) {
        assert_eq!(bkgd_header.len(), 6);
        let header_pixels = bkgd_header
            .iter()
            .skip(1)
            .step(2)
            .cloned()
            .collect::<Vec<u8>>();
        if let Some(entry) = color_palette
               .chunks(3)
               .position(|x| x == header_pixels.as_slice()) {
            *bkgd_header = vec![entry as u8];
        } else if color_palette.len() == 255 {
            return false;
        } else {
            let entry = color_palette.len() / 3;
            color_palette.extend_from_slice(&header_pixels);
            *bkgd_header = vec![entry as u8];
        }
    }

    png.raw_data = reduced;
    png.palette = Some(color_palette);
    png.ihdr_data.color_type = ColorType::Indexed;
    true
}

pub fn reduce_rgb_to_grayscale(png: &mut PngData) -> bool {
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
                        return false;
                    }
                    reduced.push(cur_pixel[0]);
                } else {
                    let pixel_bytes = cur_pixel
                        .iter()
                        .step(2)
                        .cloned()
                        .zip(cur_pixel.iter().skip(1).step(2).cloned())
                        .unique()
                        .collect::<Vec<(u8, u8)>>();
                    if pixel_bytes.len() > 1 {
                        return false;
                    }
                    reduced.push(pixel_bytes[0].0);
                    reduced.push(pixel_bytes[0].1);
                }
                cur_pixel.clear();
            }
        }
    }
    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), 3);
        sbit_header.truncate(1);
    }
    if let Some(bkgd_header) = png.aux_headers.get_mut(&"bKGD".to_string()) {
        assert_eq!(bkgd_header.len(), 6);
        bkgd_header.truncate(2);
    }

    png.raw_data = reduced;
    png.ihdr_data.color_type = ColorType::Grayscale;
    true
}

pub fn reduce_grayscale_alpha_to_grayscale(png: &mut PngData) -> bool {
    if let Some(reduced) = reduce_alpha_channel(png, 2) {
        png.raw_data = reduced;
        png.ihdr_data.color_type = ColorType::Grayscale;
        true
    } else {
        false
    }
}

fn reduce_alpha_channel(png: &mut PngData, bpp_factor: usize) -> Option<Vec<u8>> {
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
    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), bpp_factor);
        sbit_header.pop();
    }

    Some(reduced)
}
