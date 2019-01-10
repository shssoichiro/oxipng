use std::collections::HashMap;
use colors::{BitDepth, ColorType};
use std::collections::hash_map::Entry::*;
use png::PngData;
use rgb::RGBA8;

pub mod alpha;
pub mod bit_depth;
pub mod color;

/// Fields to replace in PngData to apply the reduction
pub struct ReducedPng {
    pub color_type: ColorType,
    pub raw_data: Vec<u8>,
    pub bit_depth: BitDepth,
    /// replace if Some
    pub palette: Option<Vec<RGBA8>>,
    /// replace if Some
    pub transparency_pixel: Option<Vec<u8>>,
    /// replace if Some, delete if None
    pub aux_headers: HashMap<[u8; 4], Option<Vec<u8>>>,
    pub interlaced: u8,
}

/// Attempt to reduce the number of colors in the palette
/// Returns `None` if palette hasn't changed
#[must_use]
pub fn reduced_palette(png: &PngData) -> Option<ReducedPng> {
    if png.ihdr_data.color_type != ColorType::Indexed {
        // Can't reduce if there is no palette
        return None;
    }
    if png.ihdr_data.bit_depth == BitDepth::One {
        // Gains from 1-bit images will be at most 1 byte
        // Not worth the CPU time
        return None;
    }

    let mut palette_map = [None; 256];
    let mut used = [false; 256];
    {
        let palette = png.palette.as_ref()?;

        // Find palette entries that are never used
        for line in png.scan_lines() {
            match png.ihdr_data.bit_depth {
                BitDepth::Eight => for &byte in line.data {
                    used[byte as usize] = true;
                },
                BitDepth::Four => for &byte in line.data {
                    used[(byte & 0x0F) as usize] = true;
                    used[(byte >> 4) as usize] = true;
                },
                BitDepth::Two => for &byte in line.data {
                    used[(byte & 0x03) as usize] = true;
                    used[((byte >> 2) & 0x03) as usize] = true;
                    used[((byte >> 4) & 0x03) as usize] = true;
                    used[(byte >> 6) as usize] = true;
                },
                _ => unreachable!(),
            }
        }

        let mut next_index = 0u16;
        let mut seen = HashMap::with_capacity(palette.len());
        for (i, (used, palette_map)) in
            used.iter().cloned().zip(palette_map.iter_mut()).enumerate()
        {
            if !used {
                continue;
            }
            // There are invalid files that use pixel indices beyond palette size
            let color = palette.get(i).cloned().unwrap_or(RGBA8::new(0, 0, 0, 255));
            match seen.entry(color) {
                Vacant(new) => {
                    *palette_map = Some(next_index as u8);
                    new.insert(next_index as u8);
                    next_index += 1;
                }
                Occupied(remap_to) => {
                    *palette_map = Some(*remap_to.get());
                }
            }
        }
    }

    do_palette_reduction(png, &palette_map)
}

#[must_use]
fn do_palette_reduction(png: &PngData, palette_map: &[Option<u8>; 256]) -> Option<ReducedPng> {
    let byte_map = palette_map_to_byte_map(png, palette_map)?;
    let mut raw_data = Vec::with_capacity(png.raw_data.len());

    // Reassign data bytes to new indices
    for line in png.scan_lines() {
        raw_data.push(line.filter);
        for byte in line.data {
            raw_data.push(byte_map[*byte as usize]);
        }
    }

    let mut aux_headers = HashMap::new();
    if let Some(bkgd_header) = png.aux_headers.get(b"bKGD") {
        if let Some(Some(map_to)) = bkgd_header.get(0).and_then(|&idx| palette_map.get(idx as usize)) {
            aux_headers.insert(*b"bKGD", Some(vec![*map_to]));
        }
    }

    Some(ReducedPng {
        color_type: ColorType::Indexed,
        bit_depth: png.ihdr_data.bit_depth,
        interlaced: png.ihdr_data.interlaced,
        raw_data,
        transparency_pixel: None,
        palette: Some(reordered_palette(png.palette.as_ref()?, palette_map)),
        aux_headers,
    })
}

fn palette_map_to_byte_map(png: &PngData, palette_map: &[Option<u8>; 256]) -> Option<[u8; 256]> {
    let len = png.palette.as_ref().map(|p| p.len()).unwrap_or(0);
    if (0..len).all(|i| palette_map[i].map_or(true, |to| to == i as u8)) {
        // No reduction necessary
        return None;
    }

    let mut byte_map = [0u8; 256];

    // low bit-depths can be pre-computed for every byte value
    match png.ihdr_data.bit_depth {
        BitDepth::Eight => {
            for byte in 0..=255 {
                byte_map[byte as usize] = palette_map[byte as usize].unwrap_or(0)
            }
        }
        BitDepth::Four => {
            for byte in 0..=255 {
                byte_map[byte as usize] = palette_map[(byte & 0x0F) as usize].unwrap_or(0)
                    | (palette_map[(byte >> 4) as usize].unwrap_or(0) << 4);
            }
        }
        BitDepth::Two => {
            for byte in 0..=255 {
                byte_map[byte as usize] = palette_map[(byte & 0x03) as usize].unwrap_or(0)
                    | (palette_map[((byte >> 2) & 0x03) as usize].unwrap_or(0) << 2)
                    | (palette_map[((byte >> 4) & 0x03) as usize].unwrap_or(0) << 4)
                    | (palette_map[(byte >> 6) as usize].unwrap_or(0) << 6);
            }
        }
        _ => {}
    }

    return Some(byte_map)
}

fn reordered_palette(palette: &[RGBA8], palette_map: &[Option<u8>; 256]) -> Vec<RGBA8> {
    let max_index = palette_map.iter().cloned()
        .filter_map(|x| x)
        .max()
        .unwrap_or(0) as usize;
    let mut new_palette = vec![RGBA8::new(0, 0, 0, 255); max_index + 1];
    for (&color, &map_to) in palette.iter().zip(palette_map.iter()) {
        if let Some(map_to) = map_to {
            new_palette[map_to as usize] = color;
        }
    }
    new_palette
}
