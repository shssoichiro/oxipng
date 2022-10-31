use crate::colors::AlphaOptim;
use crate::colors::ColorType;
use crate::evaluate::Evaluator;
use crate::headers::IhdrData;
use crate::png::PngImage;
#[cfg(not(feature = "parallel"))]
use crate::rayon::prelude::*;
use indexmap::IndexSet;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::sync::Arc;

pub(crate) fn try_alpha_reductions(
    png: Arc<PngImage>,
    alphas: &IndexSet<AlphaOptim>,
    eval: &Evaluator,
) {
    if alphas.is_empty() {
        return;
    }

    alphas
        .par_iter()
        .with_max_len(1)
        .filter_map(|&alpha| filtered_alpha_channel(&png, alpha))
        .for_each(|image| eval.try_image(Arc::new(image)));
}

pub fn filtered_alpha_channel(png: &PngImage, optim: AlphaOptim) -> Option<PngImage> {
    let (bpc, bpp) = match png.ihdr.color_type {
        ColorType::RGBA | ColorType::GrayscaleAlpha => {
            let cpp = png.channels_per_pixel();
            let bpc = png.ihdr.bit_depth.as_u8() / 8;
            (bpc as usize, (bpc * cpp) as usize)
        }
        _ => {
            return None;
        }
    };

    let raw_data = match optim {
        AlphaOptim::NoOp => return None,
        AlphaOptim::Black => reduced_alpha_to_black(png, bpc, bpp),
        AlphaOptim::White => reduced_alpha_to_white(png, bpc, bpp),
        AlphaOptim::Up => reduced_alpha_to_up(png, bpc, bpp),
        AlphaOptim::Down => reduced_alpha_to_down(png, bpc, bpp),
        AlphaOptim::Left => reduced_alpha_to_left(png, bpc, bpp),
        AlphaOptim::Right => reduced_alpha_to_right(png, bpc, bpp),
    };

    Some(PngImage {
        data: raw_data,
        ihdr: png.ihdr,
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
        aux_headers: png.aux_headers.clone(),
    })
}

fn reduced_alpha_to_black(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.resize(reduced.len() + bpp, 0);
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
    }
    reduced
}

fn reduced_alpha_to_white(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.resize(reduced.len() + bpp - bpc, 255);
                reduced.resize(reduced.len() + bpc, 0);
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
    }
    reduced
}

fn reduced_alpha_to_up(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let mut prev_line = Vec::new();
    let mut transparent = Vec::new();
    for line in png.scan_lines() {
        if line.data.len() != prev_line.len() {
            prev_line = vec![0; line.data.len()];
            transparent = vec![0; line.data.len()];
        }
        reduced.push(line.filter);
        let line_start = reduced.len();
        let mut line_transparent = true;
        for (col, (pixel, prev_pixel)) in
            line.data.chunks(bpp).zip(prev_line.chunks(bpp)).enumerate()
        {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                // Copy the color values from the previous line
                reduced.extend_from_slice(&prev_pixel[0..(bpp - bpc)]);
                reduced.resize(reduced.len() + bpc, 0);
                transparent[col] += 1;
            } else {
                if transparent[col] > 0 {
                    // Copy the current color values upwards in this column
                    let mut offset = line_start + col * bpp;
                    for _ in 0..transparent[col] {
                        offset -= prev_line.len() + 1;
                        reduced[offset..(offset + bpp - bpc)]
                            .copy_from_slice(&pixel[..(bpp - bpc)]);
                    }
                }
                transparent[col] = i32::MIN; // Prevent copying upwards again
                reduced.extend_from_slice(pixel);
                line_transparent = false;
            }
        }
        if line_transparent {
            // Zero out the line if it's fully transparent
            reduced.truncate(line_start);
            reduced.resize(line_start + prev_line.len(), 0);
            transparent = vec![0; prev_line.len()];
        }
        prev_line = reduced[line_start..].to_vec();
    }
    reduced
}

fn reduced_alpha_to_down(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let mut prev_line = Vec::new();
    for line in png.scan_lines() {
        if line.data.len() != prev_line.len() {
            prev_line = vec![0; line.data.len()];
        }
        reduced.push(line.filter);
        let line_start = reduced.len();
        for (pixel, prev_pixel) in line.data.chunks(bpp).zip(prev_line.chunks(bpp)) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.extend_from_slice(&prev_pixel[0..(bpp - bpc)]);
                reduced.resize(reduced.len() + bpc, 0);
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
        prev_line = reduced[line_start..].to_vec();
    }
    reduced
}

fn reduced_alpha_to_left(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut prev_pixel = vec![0; bpp];
        let mut transparent = 0;
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                // Count number of consecutive transparent pixel bytes
                transparent += bpp;
            } else {
                prev_pixel[..(bpp - bpc)].copy_from_slice(&pixel[..(bpp - bpc)]);
                if transparent > 0 {
                    // Copy the current color values to preceding transparent pixels
                    reduced.extend(prev_pixel.iter().cycle().take(transparent));
                    transparent = 0;
                }
                reduced.extend_from_slice(pixel);
            }
        }
        if transparent > 0 {
            reduced.extend(prev_pixel.iter().cycle().take(transparent));
        }
    }
    reduced
}

fn reduced_alpha_to_right(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut prev_pixel = vec![0; bpp];
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.extend_from_slice(&prev_pixel[0..(bpp - bpc)]);
                reduced.resize(reduced.len() + bpc, 0);
            } else {
                prev_pixel[..(bpp - bpc)].copy_from_slice(&pixel[..(bpp - bpc)]);
                reduced.extend_from_slice(pixel);
            }
        }
    }
    reduced
}

#[must_use]
pub fn reduced_alpha_channel(png: &PngImage) -> Option<PngImage> {
    let target_color_type = match png.ihdr.color_type {
        ColorType::GrayscaleAlpha => ColorType::Grayscale,
        ColorType::RGBA => ColorType::RGB,
        _ => return None,
    };
    let byte_depth = png.ihdr.bit_depth.as_u8() >> 3;
    let channels = png.channels_per_pixel();
    let bpp = channels * byte_depth;
    let bpp_mask = bpp - 1;
    if 0 != bpp & bpp_mask {
        return None;
    }
    let colored_bytes = bpp - byte_depth;
    for line in png.scan_lines() {
        for (i, &byte) in line.data.iter().enumerate() {
            if i as u8 & bpp_mask >= colored_bytes && byte != 255 {
                return None;
            }
        }
    }

    let mut raw_data = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        raw_data.push(line.filter);
        for (i, &byte) in line.data.iter().enumerate() {
            if i as u8 & bpp_mask >= colored_bytes {
                continue;
            }

            raw_data.push(byte);
        }
    }

    let mut aux_headers = png.aux_headers.clone();
    // sBIT contains information about alpha channel's original depth,
    // and alpha has just been removed
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        // Some programs save the sBIT header as RGB even if the image is RGBA.
        aux_headers.insert(*b"sBIT", sbit_header.iter().cloned().take(3).collect());
    }

    Some(PngImage {
        data: raw_data,
        ihdr: IhdrData {
            color_type: target_color_type,
            ..png.ihdr
        },
        aux_headers,
        transparency_pixel: None,
        palette: None,
    })
}
