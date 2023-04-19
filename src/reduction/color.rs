use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;
use indexmap::IndexMap;
use rgb::{ComponentMap, FromSlice, RGBA, RGBA8};
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

fn reduce_scanline_to_palette<T>(
    iter: impl IntoIterator<Item = T>,
    palette: &mut FxIndexMap<T, u8>,
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
pub fn reduce_to_palette(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut raw_data = Vec::with_capacity(png.data.len());
    let mut palette = FxIndexMap::default();
    palette.reserve(257);
    let ok = if let ColorType::RGB { transparent } = png.ihdr.color_type {
        // Convert the RGB16 transparency to RGB8
        let transparency_pixel = transparent.map(|t| t.map(|c| c as u8));
        reduce_scanline_to_palette(
            png.data.as_rgb().iter().cloned().map(|px| {
                px.alpha(if Some(px) != transparency_pixel {
                    255
                } else {
                    0
                })
            }),
            &mut palette,
            &mut raw_data,
        )
    } else if png.ihdr.color_type == ColorType::GrayscaleAlpha {
        reduce_scanline_to_palette(
            png.data.as_gray_alpha().iter().cloned().map(|px| RGBA {
                r: px.0,
                g: px.0,
                b: px.0,
                a: px.1,
            }),
            &mut palette,
            &mut raw_data,
        )
    } else {
        debug_assert_eq!(png.ihdr.color_type, ColorType::RGBA);
        reduce_scanline_to_palette(
            png.data.as_rgba().iter().cloned(),
            &mut palette,
            &mut raw_data,
        )
    };
    if !ok {
        return None;
    }

    let num_transparent = palette
        .iter()
        .filter_map(|(px, &idx)| {
            if px.a != 255 {
                Some(idx as usize + 1)
            } else {
                None
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
        aux_headers.insert(*b"sBIT", sbit_header.iter().cloned().take(3).collect());
    }

    let mut palette_vec = vec![RGBA8::new(0, 0, 0, 0); palette.len()];
    for (color, idx) in palette {
        palette_vec[idx as usize] = color;
    }

    Some(PngImage {
        data: raw_data,
        ihdr: IhdrData {
            color_type: ColorType::Indexed {
                palette: palette_vec,
            },
            ..png.ihdr
        },
        aux_headers,
    })
}

#[must_use]
pub fn reduce_rgb_to_grayscale(png: &PngImage) -> Option<PngImage> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let byte_depth = png.ihdr.bit_depth.as_u8() as usize >> 3;
    let bpp = png.channels_per_pixel() as usize * byte_depth;
    let last_color = 2 * byte_depth;
    for pixel in png.data.chunks(bpp) {
        if byte_depth == 1 {
            if pixel[0] != pixel[1] || pixel[1] != pixel[2] {
                return None;
            }
        } else if pixel[0..2] != pixel[2..4] || pixel[2..4] != pixel[4..6] {
            return None;
        }
        reduced.extend_from_slice(&pixel[last_color..]);
    }

    let mut aux_headers = png.aux_headers.clone();
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        if let Some(&byte) = sbit_header.first() {
            aux_headers.insert(*b"sBIT", vec![byte]);
        }
    }
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        if let Some(b) = bkgd_header.get(0..2) {
            aux_headers.insert(*b"bKGD", b.to_owned());
        }
    }

    let color_type = match png.ihdr.color_type {
        ColorType::RGB { transparent } => ColorType::Grayscale {
            // Copy the transparent component if it is also gray
            transparent: transparent
                .filter(|t| t.r == t.g && t.g == t.b)
                .map(|t| t.r),
        },
        _ => ColorType::GrayscaleAlpha,
    };

    Some(PngImage {
        data: reduced,
        ihdr: IhdrData {
            color_type,
            ..png.ihdr
        },
        aux_headers,
    })
}
