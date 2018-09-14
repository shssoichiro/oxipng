use colors::{BitDepth, ColorType};
use itertools::Itertools;
use png::PngData;

use super::alpha::reduce_alpha_channel;

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
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp = 4 * byte_depth;
    let bpp_mask = bpp - 1;
    assert_eq!(0, bpp & bpp_mask);
    let colored_bytes = bpp - byte_depth;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut low_bytes = Vec::with_capacity(4);
        let mut high_bytes = Vec::with_capacity(4);
        let mut trans_bytes = Vec::with_capacity(byte_depth as usize);
        for (i, byte) in line.data.iter().enumerate() {
            if i as u8 & bpp_mask < colored_bytes {
                if byte_depth == 1 || i % 2 == 1 {
                    low_bytes.push(*byte);
                } else {
                    high_bytes.push(*byte);
                }
            } else {
                trans_bytes.push(*byte);
            }

            if (i as u8 & bpp_mask) == bpp - 1 {
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

    if let Some(sbit_header) = png.aux_headers.get_mut(b"sBIT") {
        assert_eq!(sbit_header.len(), 4);
        sbit_header.remove(1);
        sbit_header.remove(1);
    }
    if let Some(bkgd_header) = png.aux_headers.get_mut(b"bKGD") {
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

    let mut color_palette = Vec::with_capacity(
        palette.len() * 3 + if png.aux_headers.contains_key(b"bKGD") {
            6
        } else {
            0
        },
    );
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

    if let Some(bkgd_header) = png.aux_headers.get_mut(b"bKGD") {
        assert_eq!(bkgd_header.len(), 6);
        let header_pixels = bkgd_header
            .iter()
            .skip(1)
            .step(2)
            .cloned()
            .collect::<Vec<u8>>();
        if let Some(entry) = color_palette
            .chunks(3)
            .position(|x| x == header_pixels.as_slice())
        {
            *bkgd_header = vec![entry as u8];
        } else if color_palette.len() / 3 == 256 {
            return false;
        } else {
            let entry = color_palette.len() / 3;
            color_palette.extend_from_slice(&header_pixels);
            *bkgd_header = vec![entry as u8];
        }
    }
    if let Some(sbit_header) = png.aux_headers.get_mut(b"sBIT") {
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
    if let Some(ref trns) = png.transparency_pixel {
        assert_eq!(trns.len(), 6);
        if trns[0] != trns[1] || trns[2] != trns[3] || trns[4] != trns[5] {
            return false;
        }
        palette.push(vec![trns[0], trns[2], trns[4]]);
    }
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

    if let Some(bkgd_header) = png.aux_headers.get_mut(b"bKGD") {
        assert_eq!(bkgd_header.len(), 6);
        let header_pixels = bkgd_header
            .iter()
            .skip(1)
            .step(2)
            .cloned()
            .collect::<Vec<u8>>();
        if let Some(entry) = color_palette
            .chunks(3)
            .position(|x| x == header_pixels.as_slice())
        {
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
    if png.transparency_pixel.is_some() {
        png.transparency_pixel = None;
        png.transparency_palette = Some(vec![0]);
    };
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
    if let Some(ref mut trns) = png.transparency_pixel {
        assert_eq!(trns.len(), 6);
        if trns[0..2] != trns[2..4] || trns[2..4] != trns[4..6] {
            return false;
        }
        *trns = trns[0..2].to_owned();
    }
    if let Some(sbit_header) = png.aux_headers.get_mut(b"sBIT") {
        assert_eq!(sbit_header.len(), 3);
        sbit_header.truncate(1);
    }
    if let Some(bkgd_header) = png.aux_headers.get_mut(b"bKGD") {
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
