use reduction::ReducedPng;
use colors::{BitDepth, ColorType};
use itertools::Itertools;
use png::PngData;
use rgb::{FromSlice, RGB8, RGBA8};
use std::collections::HashMap;
use std::hash::Hash;

#[must_use]
pub fn reduce_rgba_to_grayscale_alpha(png: &PngData) -> Option<ReducedPng> {
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

    let mut aux_headers = HashMap::new();
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        aux_headers.insert(*b"sBIT", sbit_header.get(0).map(|&s| vec![s]));
    }

    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        aux_headers.insert(*b"bKGD", bkgd_header.get(0..2).map(|b| b.to_owned()));
    }

    Some(ReducedPng {
        raw_data: reduced,
        bit_depth: png.ihdr_data.bit_depth,
        interlaced: png.ihdr_data.interlaced,
        color_type: ColorType::GrayscaleAlpha,
        palette: None,
        transparency_pixel: None,
        aux_headers,
    })
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

#[must_use]
pub fn reduced_color_to_palette(png: &PngData) -> Option<ReducedPng> {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut raw_data = Vec::with_capacity(png.raw_data.len());
    let mut palette = HashMap::with_capacity(257);
    let transparency_pixel = png
        .transparency_pixel
        .as_ref()
        .map(|t| RGB8::new(t[1], t[3], t[5]));
    for line in png.scan_lines() {
        raw_data.push(line.filter);
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
                &mut raw_data,
            )
        } else {
            debug_assert_eq!(png.ihdr_data.color_type, ColorType::RGBA);
            reduce_scanline_to_palette(
                line.data.as_rgba().iter().cloned(),
                &mut palette,
                &mut raw_data,
            )
        };
        if !ok {
            return None;
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
    if raw_data.len() + headers_size > png.raw_data.len() {
        // Reduction would result in a larger image
        return None;
    }

    let mut aux_headers = HashMap::new();
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
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
            return None; // No space in palette to store the bg as an index
        };
        aux_headers.insert(*b"bKGD", Some(vec![entry]));
    }

    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        // Some programs save the sBIT header as RGB even if the image is RGBA.
        aux_headers.insert(*b"sBIT", Some(sbit_header.iter().cloned().take(3).collect()));
    }

    let mut palette_vec = vec![RGBA8::new(0, 0, 0, 0); palette.len()];
    for (color, idx) in palette {
        palette_vec[idx as usize] = color;
    }

    Some(ReducedPng {
        color_type: ColorType::Indexed,
        bit_depth: png.ihdr_data.bit_depth,
        interlaced: png.ihdr_data.interlaced,
        aux_headers,
        raw_data,
        transparency_pixel: None,
        palette: Some(palette_vec),
    })
}

#[must_use]
pub fn reduce_rgb_to_grayscale(png: &PngData) -> Option<ReducedPng> {
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
                    let pixel_bytes = cur_pixel
                        .iter()
                        .step(2)
                        .cloned()
                        .zip(cur_pixel.iter().skip(1).step(2).cloned())
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

    let transparency_pixel = if let Some(ref trns) = png.transparency_pixel {
        if trns.len() != 6 || trns[0..2] != trns[2..4] || trns[2..4] != trns[4..6] {
            None
        } else {
            Some(trns[0..2].to_owned())
        }
    } else {
        png.transparency_pixel.clone()
    };

    let mut aux_headers = HashMap::new();
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        aux_headers.insert(*b"sBIT", sbit_header.get(0).map(|&byte| vec![byte]));
    }
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        aux_headers.insert(*b"bKGD", bkgd_header.get(0..2).map(|b| b.to_owned()));
    }

    Some(ReducedPng {
        raw_data: reduced,
        color_type: ColorType::Grayscale,
        bit_depth: png.ihdr_data.bit_depth,
        interlaced: png.ihdr_data.interlaced,
        palette: None,
        transparency_pixel,
        aux_headers,
    })
}
