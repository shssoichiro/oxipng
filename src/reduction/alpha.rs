use crate::colors::AlphaOptim;
use crate::colors::ColorType;
use crate::evaluate::Evaluator;
use crate::headers::IhdrData;
use crate::png::scan_lines::ScanLine;
use crate::png::PngImage;
#[cfg(not(feature = "parallel"))]
use crate::rayon::prelude::*;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) fn try_alpha_reductions(
    png: Arc<PngImage>,
    alphas: &HashSet<AlphaOptim>,
    eval: &Evaluator,
) {
    if alphas.is_empty() {
        return;
    }

    let alphas = alphas.iter().collect::<Vec<_>>();
    let alphas_iter = alphas.par_iter().with_max_len(1);
    alphas_iter
        .filter_map(|&alpha| filtered_alpha_channel(&png, *alpha))
        .for_each(|image| eval.try_image(Arc::new(image), 0.99));
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
                for _ in 0..bpp {
                    reduced.push(0);
                }
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
                for _ in 0..(bpp - bpc) {
                    reduced.push(255);
                }
                for _ in 0..bpc {
                    reduced.push(0);
                }
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
    }
    reduced
}

fn reduced_alpha_to_up(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut scan_lines = png.scan_lines().collect::<Vec<ScanLine<'_>>>();
    scan_lines.reverse();
    let mut lines = Vec::with_capacity(scan_lines.len());
    let mut last_line = Vec::new();
    let mut current_line = Vec::with_capacity(scan_lines[0].data.len() + 1); // filter size + pixels
    for line in scan_lines {
        if line.data.len() != last_line.len() {
            last_line = vec![0; line.data.len()];
        }
        current_line.push(line.filter);
        for (pixel, last_pixel) in line.data.chunks(bpp).zip(last_line.chunks(bpp)) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                current_line.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                for _ in 0..bpc {
                    current_line.push(0);
                }
            } else {
                current_line.extend_from_slice(pixel);
            }
        }
        last_line = current_line.clone();
        lines.push(current_line.clone());
        current_line.clear();
    }
    lines.into_iter().rev().flatten().collect()
}

fn reduced_alpha_to_down(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    let mut last_line = Vec::new();
    for line in png.scan_lines() {
        if line.data.len() != last_line.len() {
            last_line = vec![0; line.data.len()];
        }
        reduced.push(line.filter);
        for (pixel, last_pixel) in line.data.chunks(bpp).zip(last_line.chunks(bpp)) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                for _ in 0..bpc {
                    reduced.push(0);
                }
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
        last_line = reduced.clone();
    }
    reduced
}

fn reduced_alpha_to_left(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        let mut line_bytes = Vec::with_capacity(line.data.len());
        let mut last_pixel = vec![0; bpp];
        for pixel in line.data.chunks(bpp).rev() {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                line_bytes.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                for _ in 0..bpc {
                    line_bytes.push(0);
                }
            } else {
                line_bytes.extend_from_slice(pixel);
            }
            last_pixel = pixel.to_owned();
        }
        reduced.push(line.filter);
        reduced.extend(line_bytes.chunks(bpp).rev().flatten());
    }
    reduced
}

fn reduced_alpha_to_right(png: &PngImage, bpc: usize, bpp: usize) -> Vec<u8> {
    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut last_pixel = vec![0; bpp];
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                reduced.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                for _ in 0..bpc {
                    reduced.push(0);
                }
            } else {
                reduced.extend_from_slice(pixel);
            }
            last_pixel = pixel.to_owned();
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
    assert_eq!(0, bpp & bpp_mask);
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
            } else {
                raw_data.push(byte);
            }
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
