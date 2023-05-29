use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;
use indexmap::IndexSet;
use rgb::RGBA8;

/// Attempt to reduce the number of colors in the palette, returning the reduced image if successful
#[must_use]
pub fn reduced_palette(png: &PngImage, optimize_alpha: bool) -> Option<PngImage> {
    let palette = match &png.ihdr.color_type {
        ColorType::Indexed { palette } if palette.len() > 1 => palette,
        _ => return None,
    };

    let used = get_used_entries(png);

    let black = RGBA8::new(0, 0, 0, 255);
    let mut condensed = IndexSet::with_capacity(palette.len());
    let mut palette_map = [0; 256];
    let mut did_change = false;
    for (i, used) in used.iter().enumerate() {
        if !used {
            continue;
        }
        // There are invalid files that use pixel indices beyond palette size
        let color = *palette.get(i).unwrap_or(&black);
        palette_map[i] = add_color_to_set(color, &mut condensed, optimize_alpha);
        if palette_map[i] as usize != i {
            did_change = true;
        }
    }

    let data = if did_change {
        // Reassign data bytes to new indices
        let byte_map = palette_map_to_byte_map(png.ihdr.bit_depth, &palette_map);
        png.data.iter().map(|b| byte_map[*b as usize]).collect()
    } else if condensed.len() < palette.len() {
        // Data is unchanged but palette will be truncated
        png.data.clone()
    } else {
        // Nothing has changed
        return None;
    };

    let palette: Vec<_> = condensed.into_iter().collect();

    Some(PngImage {
        ihdr: IhdrData {
            color_type: ColorType::Indexed { palette },
            ..png.ihdr
        },
        data,
    })
}

fn add_color_to_set(mut color: RGBA8, set: &mut IndexSet<RGBA8>, optimize_alpha: bool) -> u8 {
    // If there are multiple fully transparent entries, reduce them into one
    if optimize_alpha && color.a == 0 {
        color.r = 0;
        color.g = 0;
        color.b = 0;
    }
    let (idx, _) = set.insert_full(color);
    idx as u8
}

fn get_used_entries(png: &PngImage) -> [bool; 256] {
    let mut used = [false; 256];
    match png.ihdr.bit_depth {
        BitDepth::Eight => {
            for &byte in &png.data {
                used[byte as usize] = true;
            }
        }
        BitDepth::Four => {
            for &byte in &png.data {
                used[(byte & 0x0F) as usize] = true;
                used[(byte >> 4) as usize] = true;
            }
        }
        BitDepth::Two => {
            for &byte in &png.data {
                used[(byte & 0x03) as usize] = true;
                used[((byte >> 2) & 0x03) as usize] = true;
                used[((byte >> 4) & 0x03) as usize] = true;
                used[(byte >> 6) as usize] = true;
            }
        }
        BitDepth::One => {
            // Only two options, don't bother checking which are actually used
            used[0] = true;
            used[1] = true;
        }
        _ => unreachable!(),
    };
    used
}

fn palette_map_to_byte_map(bit_depth: BitDepth, palette_map: &[u8; 256]) -> [u8; 256] {
    // Low bit-depths can be pre-computed for every byte value
    match bit_depth {
        BitDepth::Eight => *palette_map,
        BitDepth::Four => {
            let mut byte_map = [0_u8; 256];
            for byte in 0..256 {
                byte_map[byte] = palette_map[byte & 0x0F] | (palette_map[byte >> 4] << 4);
            }
            byte_map
        }
        BitDepth::Two => {
            let mut byte_map = [0_u8; 256];
            for byte in 0..256 {
                byte_map[byte] = palette_map[byte & 0x03]
                    | (palette_map[(byte >> 2) & 0x03] << 2)
                    | (palette_map[(byte >> 4) & 0x03] << 4)
                    | (palette_map[byte >> 6] << 6);
            }
            byte_map
        }
        _ => unreachable!(),
    }
}

/// Attempt to sort the colors in the palette, returning the sorted image if successful
#[must_use]
pub fn sorted_palette(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth == BitDepth::One {
        // Don't bother trying to sort a 1-bit image
        return None;
    }
    let palette = match &png.ihdr.color_type {
        ColorType::Indexed { palette } => palette,
        _ => return None,
    };

    let mut enumerated: Vec<_> = palette.iter().enumerate().collect();

    // Sort the palette
    enumerated.sort_by(|a, b| {
        // Sort by ascending alpha and descending luma
        let color_val = |color: &RGBA8| {
            ((color.a as i32) << 18)
            // These are coefficients for standard sRGB to luma conversion
            - i32::from(color.r) * 299
            - i32::from(color.g) * 587
            - i32::from(color.b) * 114
        };
        color_val(a.1).cmp(&color_val(b.1))
    });

    // Extract the new palette and determine if anything changed
    let (old_map, palette): (Vec<_>, Vec<RGBA8>) = enumerated.into_iter().unzip();
    if old_map.iter().enumerate().all(|(a, b)| a == *b) {
        return None;
    }

    // Construct the palette and byte maps and convert the data
    let mut new_map = [0; 256];
    for (i, &v) in old_map.iter().enumerate() {
        new_map[v] = i as u8;
    }
    let byte_map = palette_map_to_byte_map(png.ihdr.bit_depth, &new_map);
    let data = png.data.iter().map(|&b| byte_map[b as usize]).collect();

    Some(PngImage {
        ihdr: IhdrData {
            color_type: ColorType::Indexed { palette },
            ..png.ihdr
        },
        data,
    })
}
