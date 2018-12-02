use colors::{BitDepth, ColorType};
use itertools::Itertools;
use png::PngData;
use rgb::{FromSlice, RGB8, RGBA8};
use std::collections::HashMap;
use std::hash::Hash;

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
        assert!(sbit_header.len() >= 3);
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

fn reduce_scanline_to_palette<T>(
    iter: impl IntoIterator<Item = T>,
    palette: &mut HashMap<T, u8>,
    reduced: &mut Vec<u8>,
) -> bool
where
    T: Eq + Hash,
{
    for pixel in iter {
        let idx = if let Some(&idx) = palette.get(&pixel) {
            idx
        } else {
            let len = palette.len();
            if len == 256 {
                return false;
            }
            let idx = len as u8;
            palette.insert(pixel, idx);
            idx
        };
        reduced.push(idx);
    }
    true
}

pub fn reduce_color_to_palette(png: &mut PngData) -> bool {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return false;
    }
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = HashMap::with_capacity(257);
    let transparency_pixel = png
        .transparency_pixel
        .as_ref()
        .map(|t| RGB8::new(t[1], t[3], t[5]));
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let ok = if png.ihdr_data.color_type == ColorType::RGB {
            reduce_scanline_to_palette(
                line.data.as_rgb().iter().cloned().map(|px| {
                    px.alpha(if Some(px) != transparency_pixel {
                        255
                    } else {
                        0
                    })
                }),
                &mut palette,
                &mut reduced,
            )
        } else {
            debug_assert_eq!(png.ihdr_data.color_type, ColorType::RGBA);
            reduce_scanline_to_palette(
                line.data.as_rgba().iter().cloned(),
                &mut palette,
                &mut reduced,
            )
        };
        if !ok {
            return false;
        }
    }

    let num_transparent = palette
        .iter()
        .filter_map(|(px, &idx)| {
            if px.a != 255 {
                Some(idx as usize + 1)
            } else {
                None
            }
        }).max();
    let trns_size = num_transparent.map(|n| n + 8).unwrap_or(0);

    let headers_size = palette.len() * 3 + 8 + trns_size;
    if reduced.len() + headers_size > png.raw_data.len() {
        // Reduction would result in a larger image
        return false;
    }

    if let Some(bkgd_header) = png.aux_headers.get_mut(b"bKGD") {
        assert_eq!(bkgd_header.len(), 6);
        // In bKGD 16-bit values are used even for 8-bit images
        let bg = RGBA8::new(bkgd_header[1], bkgd_header[3], bkgd_header[5], 255);
        let entry = if let Some(&entry) = palette.get(&bg) {
            entry
        } else if palette.len() < 256 {
            let entry = palette.len() as u8;
            palette.insert(bg, entry);
            entry
        } else {
            return false;
        };
        *bkgd_header = vec![entry];
    }

    if let Some(sbit_header) = png.aux_headers.get_mut(b"sBIT") {
        // Some programs save the sBIT header as RGB even if the image is RGBA.
        // Only remove the alpha channel if it's actually there.
        if sbit_header.len() == 4 {
            sbit_header.pop();
        }
    }

    let mut palette_vec = vec![RGBA8::new(0, 0, 0, 0); palette.len()];
    for (color, idx) in palette {
        palette_vec[idx as usize] = color;
    }

    png.raw_data = reduced;
    png.transparency_pixel = None;
    png.palette = Some(palette_vec);
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
