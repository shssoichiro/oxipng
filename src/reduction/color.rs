use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;
use indexmap::IndexMap;
use itertools::Itertools;
use rgb::{FromSlice, RGB8, RGBA8};
use std::hash::Hash;

#[must_use]
pub fn reduce_rgba_to_grayscale_alpha(png: &PngImage) -> Option<PngImage> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let byte_depth = png.ihdr.bit_depth.as_u8() >> 3;
    let bpp = 4 * byte_depth;
    let bpp_mask = bpp - 1;
    if 0 != bpp & bpp_mask {
        return None;
    }
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

    let mut aux_headers = png.aux_headers.clone();
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        if let Some(&s) = sbit_header.get(0) {
            aux_headers.insert(*b"sBIT", vec![s]);
        }
    }

    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        if let Some(b) = bkgd_header.get(0..2) {
            aux_headers.insert(*b"bKGD", b.to_owned());
        }
    }

    Some(PngImage {
        data: reduced,
        ihdr: IhdrData {
            color_type: ColorType::GrayscaleAlpha,
            ..png.ihdr
        },
        palette: None,
        transparency_pixel: None,
        aux_headers,
    })
}

fn reduce_scanline_to_palette<T>(
    iter: impl IntoIterator<Item = T>,
    palette: &mut IndexMap<T, u8>,
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
pub fn reduced_color_to_palette(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut raw_data = Vec::with_capacity(png.data.len());
    let mut palette = IndexMap::with_capacity(257);
    let transparency_pixel = png
        .transparency_pixel
        .as_ref()
        .filter(|t| png.ihdr.color_type == ColorType::RGB && t.len() >= 6)
        .map(|t| RGB8::new(t[1], t[3], t[5]));
    for line in png.scan_lines() {
        raw_data.push(line.filter);
        let ok = if png.ihdr.color_type == ColorType::RGB {
            reduce_scanline_to_palette(
                line.data.as_rgb().iter().copied().map(|px| {
                    px.alpha(if Some(px) == transparency_pixel {
                        0
                    } else {
                        255
                    })
                }),
                &mut palette,
                &mut raw_data,
            )
        } else {
            debug_assert_eq!(png.ihdr.color_type, ColorType::RGBA);
            reduce_scanline_to_palette(
                line.data.as_rgba().iter().copied(),
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
            if px.a == 255 {
                None
            } else {
                Some(idx as usize + 1)
            }
        })
        .max();
    let trns_size = num_transparent.map_or(0, |n| n + 8);

    let headers_size = palette.len() * 3 + 8 + trns_size;
    if raw_data.len() + headers_size > png.data.len() {
        // Reduction would result in a larger image
        return None;
    }

    let mut aux_headers = png.aux_headers.clone();
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        if bkgd_header.len() != 6 {
            // malformed chunk?
            return None;
        }
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
        aux_headers.insert(*b"bKGD", vec![entry]);
    }

    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        // Some programs save the sBIT header as RGB even if the image is RGBA.
        aux_headers.insert(*b"sBIT", sbit_header.iter().copied().take(3).collect());
    }

    let mut palette_vec = vec![RGBA8::new(0, 0, 0, 0); palette.len()];
    for (color, idx) in palette {
        palette_vec[idx as usize] = color;
    }

    Some(PngImage {
        data: raw_data,
        ihdr: IhdrData {
            color_type: ColorType::Indexed,
            ..png.ihdr
        },
        aux_headers,
        transparency_pixel: None,
        palette: Some(palette_vec),
    })
}

#[must_use]
pub fn reduce_rgb_to_grayscale(png: &PngImage) -> Option<PngImage> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let byte_depth: u8 = png.ihdr.bit_depth.as_u8() >> 3;
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
                        .step_by(2)
                        .copied()
                        .zip(cur_pixel.iter().skip(1).step_by(2).copied())
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

    let transparency_pixel = png.transparency_pixel.as_ref().map_or_else(
        || png.transparency_pixel.clone(),
        |trns| {
            if trns.len() != 6 || trns[0..2] != trns[2..4] || trns[2..4] != trns[4..6] {
                None
            } else {
                Some(trns[0..2].to_owned())
            }
        },
    );

    let mut aux_headers = png.aux_headers.clone();
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        if let Some(&byte) = sbit_header.get(0) {
            aux_headers.insert(*b"sBIT", vec![byte]);
        }
    }
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        if let Some(b) = bkgd_header.get(0..2) {
            aux_headers.insert(*b"bKGD", b.to_owned());
        }
    }

    Some(PngImage {
        data: reduced,
        ihdr: IhdrData {
            color_type: ColorType::Grayscale,
            ..png.ihdr
        },
        aux_headers,
        palette: None,
        transparency_pixel,
    })
}
