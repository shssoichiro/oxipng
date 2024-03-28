use indexmap::IndexSet;
use rgb::RGBA8;

use crate::{
    colors::{BitDepth, ColorType},
    headers::IhdrData,
    png::{scan_lines::ScanLine, PngImage},
    Interlacing,
};

/// Attempt to reduce the number of colors in the palette, returning the reduced image if successful
#[must_use]
pub fn reduced_palette(png: &PngImage, optimize_alpha: bool) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
        return None;
    }
    let ColorType::Indexed { palette } = &png.ihdr.color_type else {
        return None;
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
    } else if condensed.len() != palette.len() {
        // Data is unchanged but palette is different size
        // Note the new palette could potentially be larger if the original had a missing entry
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

/// Attempt to sort the colors in the palette by luma, returning the sorted image if successful
#[must_use]
pub fn sorted_palette(png: &PngImage) -> Option<PngImage> {
    if png.ihdr.bit_depth != BitDepth::Eight {
        return None;
    }
    let palette = match &png.ihdr.color_type {
        ColorType::Indexed { palette } if palette.len() > 1 => palette,
        _ => return None,
    };

    let mut enumerated: Vec<_> = palette.iter().enumerate().collect();
    // Put the most popular edge color first, which can help slightly if the filter bytes are 0
    let keep_first = most_popular_edge_color(palette.len(), png);
    let first = enumerated.remove(keep_first);

    // Sort the palette
    enumerated.sort_by(|a, b| {
        // Sort by ascending alpha and descending luma
        let color_val = |color: &RGBA8| {
            let a = i32::from(color.a);
            // Put 7 high bits of alpha first, then luma, then low bit of alpha
            // This provides notable improvement in images with a lot of alpha
            ((a & 0xFE) << 18) + (a & 0x01)
            // These are coefficients for standard sRGB to luma conversion
            - i32::from(color.r) * 299
            - i32::from(color.g) * 587
            - i32::from(color.b) * 114
        };
        color_val(a.1).cmp(&color_val(b.1))
    });
    enumerated.insert(0, first);

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

/// Sort the colors in the palette by minimizing entropy, returning the sorted image if successful
#[must_use]
pub fn sorted_palette_battiato(png: &PngImage) -> Option<PngImage> {
    // Interlacing not currently supported
    if png.ihdr.bit_depth != BitDepth::Eight || png.ihdr.interlaced != Interlacing::None {
        return None;
    }
    let palette = match &png.ihdr.color_type {
        // Images with only two colors will remain unchanged from previous luma sort
        ColorType::Indexed { palette } if palette.len() > 2 => palette,
        _ => return None,
    };

    let matrix = co_occurrence_matrix(palette.len(), png);
    let edges = weighted_edges(&matrix);
    let mut old_map = battiato_tsp(palette.len(), edges);

    // Put the most popular edge color first, which can help slightly if the filter bytes are 0
    let keep_first = most_popular_edge_color(palette.len(), png);
    let first_idx = old_map.iter().position(|&i| i == keep_first).unwrap();
    // If the index is past halfway, reverse the order so as to minimize the change
    if first_idx >= old_map.len() / 2 {
        old_map.reverse();
        old_map.rotate_right(first_idx + 1);
    } else {
        old_map.rotate_left(first_idx);
    }

    // Check if anything changed
    if old_map.iter().enumerate().all(|(a, b)| a == *b) {
        return None;
    }

    // Construct the palette and byte maps and convert the data
    let mut new_palette = Vec::new();
    let mut byte_map = [0; 256];
    for (i, &v) in old_map.iter().enumerate() {
        new_palette.push(palette[v]);
        byte_map[v] = i as u8;
    }
    let data = png.data.iter().map(|&b| byte_map[b as usize]).collect();

    Some(PngImage {
        ihdr: IhdrData {
            color_type: ColorType::Indexed {
                palette: new_palette,
            },
            ..png.ihdr
        },
        data,
    })
}

// Find the most popular color on the image edges (the pixels neighboring the filter bytes)
fn most_popular_edge_color(num_colors: usize, png: &PngImage) -> usize {
    let mut counts = [0u32; 256];
    for line in png.scan_lines(false) {
        if let &[first, .., last] = line.data {
            counts[first as usize] += 1;
            counts[last as usize] += 1;
        }
    }
    counts
        .iter()
        .copied()
        .take(num_colors)
        .enumerate()
        .max_by_key(|&(_, v)| v)
        .unwrap_or_default()
        .0
}

// Calculate co-occurences matrix
fn co_occurrence_matrix(num_colors: usize, png: &PngImage) -> Vec<Vec<u32>> {
    let mut matrix = vec![vec![0u32; num_colors]; num_colors];
    let mut prev: Option<ScanLine> = None;
    let mut prev_val = None;
    for line in png.scan_lines(false) {
        for i in 0..line.data.len() {
            let val = line.data[i] as usize;
            if val > num_colors {
                continue;
            }
            if let Some(prev_val) = prev_val.replace(val) {
                matrix[prev_val][val] += 1;
            }
            if let Some(prev) = &prev {
                matrix[prev.data[i] as usize][val] += 1;
            }
        }
        prev = Some(line)
    }
    matrix
}

// Calculate edge list sorted by weight
fn weighted_edges(matrix: &[Vec<u32>]) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for i in 0..matrix.len() {
        for j in 0..i {
            edges.push(((j, i), matrix[i][j] + matrix[j][i]));
        }
    }
    edges.sort_by(|(_, w1), (_, w2)| w2.cmp(w1));
    edges.into_iter().map(|(e, _)| e).collect()
}

// Calculate an approximate solution of the Traveling Salesman Problem using the algorithm
// from "An efficient Re-indexing algorithm for color-mapped images" by Battiato et al
// https://ieeexplore.ieee.org/document/1344033
fn battiato_tsp(num_colors: usize, edges: Vec<(usize, usize)>) -> Vec<usize> {
    let mut chains = Vec::new();
    // Keep track of the state of each vertex (.0) and it's chain number (.1)
    // 0 = an unvisited vertex (White)
    // 1 = an endpoint of a chain (Red)
    // 2 = part of the middle of a chain (Black)
    let mut vx = vec![(0, 0); num_colors];

    // Iterate the edges and assemble them into a chain
    for (i, j) in edges {
        let vi = vx[i];
        let vj = vx[j];
        if vi.0 == 0 && vj.0 == 0 {
            // Two unvisited vertices - create a new chain
            vx[i].0 = 1;
            vx[i].1 = chains.len();
            vx[j].0 = 1;
            vx[j].1 = chains.len();
            chains.push(vec![i, j]);
        } else if vi.0 == 0 && vj.0 == 1 {
            // An unvisited vertex connects with an endpoint of an existing chain
            vx[i].0 = 1;
            vx[i].1 = vj.1;
            vx[j].0 = 2;
            let chain = &mut chains[vj.1];
            if chain[0] == j {
                chain.insert(0, i);
            } else {
                chain.push(i);
            }
        } else if vi.0 == 1 && vj.0 == 0 {
            // An unvisited vertex connects with an endpoint of an existing chain
            vx[j].0 = 1;
            vx[j].1 = vi.1;
            vx[i].0 = 2;
            let chain = &mut chains[vi.1];
            if chain[0] == i {
                chain.insert(0, j);
            } else {
                chain.push(j);
            }
        } else if vi.0 == 1 && vj.0 == 1 && vi.1 != vj.1 {
            // Two endpoints of different chains are connected together
            vx[i].0 = 2;
            vx[j].0 = 2;
            let (a, b) = if vi.1 < vj.1 { (i, j) } else { (j, i) };
            let ca = vx[a].1;
            let cb = vx[b].1;
            let chainb = std::mem::take(&mut chains[cb]);
            for &v in &chainb {
                vx[v].1 = ca;
            }
            let chaina = &mut chains[ca];
            if chaina[0] == a && chainb[0] == b {
                for v in chainb {
                    chaina.insert(0, v);
                }
            } else if chaina[0] == a {
                chaina.splice(0..0, chainb);
            } else if chainb[0] == b {
                chaina.extend(chainb);
            } else {
                let pos = chaina.len();
                for v in chainb {
                    chaina.insert(pos, v);
                }
            }
        }

        if chains[0].len() == num_colors {
            break;
        }
    }

    // Return the completed chain
    chains.swap_remove(0)
}
