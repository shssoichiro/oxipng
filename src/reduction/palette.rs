use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;
use indexmap::IndexSet;
use rgb::RGBA8;

/// Attempt to reduce the number of colors in the palette, returning the reduced image if successful
#[must_use]
pub fn reduced_palette(png: &PngImage, optimize_alpha: bool) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
        return None;
    }
    let palette = match &png.ihdr.color_type {
        ColorType::Indexed { palette } if palette.len() > 1 => palette,
        _ => return None,
    };

    let mut used = [false; 256];
    for &byte in &png.data {
        used[byte as usize] = true;
    }

    let black = RGBA8::new(0, 0, 0, 255);
    let mut condensed = IndexSet::with_capacity(palette.len());
    let mut byte_map = [0; 256];
    let mut did_change = false;
    for (i, used) in used.iter().enumerate() {
        if !used {
            continue;
        }
        // There are invalid files that use pixel indices beyond palette size
        let color = *palette.get(i).unwrap_or(&black);
        byte_map[i] = add_color_to_set(color, &mut condensed, optimize_alpha);
        if byte_map[i] as usize != i {
            did_change = true;
        }
    }

    let data = if did_change {
        // Reassign data bytes to new indices
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

/// Attempt to sort the colors in the palette, returning the sorted image if successful
#[must_use]
pub fn sorted_palette(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
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

    // Construct the new mapping and convert the data
    let mut byte_map = [0; 256];
    for (i, &v) in old_map.iter().enumerate() {
        byte_map[v] = i as u8;
    }
    let data = png.data.iter().map(|&b| byte_map[b as usize]).collect();

    Some(PngImage {
        ihdr: IhdrData {
            color_type: ColorType::Indexed { palette },
            ..png.ihdr
        },
        data,
    })
}
