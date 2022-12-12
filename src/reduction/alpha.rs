use crate::colors::{BitDepth, ColorType};
use crate::headers::IhdrData;
use crate::png::PngImage;

/// Clean the alpha channel by setting the color of all fully transparent pixels to black
pub fn cleaned_alpha_channel(png: &PngImage) -> Option<PngImage> {
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

    let mut reduced = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for pixel in line.data.chunks(bpp) {
            if pixel.iter().skip(bpp - bpc).all(|b| *b == 0) {
                reduced.resize(reduced.len() + bpp, 0);
            } else {
                reduced.extend_from_slice(pixel);
            }
        }
    }

    Some(PngImage {
        data: reduced,
        ihdr: png.ihdr,
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
        aux_headers: png.aux_headers.clone(),
    })
}

#[must_use]
pub fn reduced_alpha_channel(png: &PngImage, optimize_alpha: bool) -> Option<PngImage> {
    let target_color_type = match png.ihdr.color_type {
        ColorType::GrayscaleAlpha => ColorType::Grayscale,
        ColorType::RGBA => ColorType::RGB,
        _ => return None,
    };
    let byte_depth = (png.ihdr.bit_depth.as_u8() >> 3) as usize;
    let channels = png.channels_per_pixel() as usize;
    let bpp = channels * byte_depth;
    let colored_bytes = bpp - byte_depth;

    // If alpha optimisation is enabled, see if the image contains only fully opaque and fully transparent pixels.
    // In case this occurs, we want to try and find an unused color we can use for the tRNS chunk.
    // Rather than an exhaustive search, we will just keep track of 256 shades of gray, which should cover many cases.
    let mut has_transparency = false;
    let mut used_colors = vec![false; 256];

    for line in png.scan_lines() {
        for pixel in line.data.chunks(bpp) {
            if optimize_alpha && pixel.iter().skip(colored_bytes).all(|b| *b == 0) {
                // Fully transparent, we may be able to reduce with tRNS
                has_transparency = true;
            } else if pixel.iter().skip(colored_bytes).any(|b| *b != 255) {
                // Partially transparent, the image is not reducible
                return None;
            } else if optimize_alpha && pixel.iter().take(colored_bytes).all(|b| *b == pixel[0]) {
                // Opaque shade of gray, we can't use this color for tRNS
                used_colors[pixel[0] as usize] = true;
            }
        }
    }

    let transparency_pixel = if has_transparency {
        // If no unused color was found we will have to fail here
        // Otherwise, proceed to construct the tRNS chunk
        let unused_color = used_colors.iter().position(|b| !*b)? as u8;
        Some(match png.ihdr.bit_depth {
            BitDepth::Sixteen => vec![unused_color; colored_bytes],
            // 8-bit is still stored as 16-bit, with the high byte set to 0
            _ => [0, unused_color].repeat(colored_bytes),
        })
    } else {
        None
    };

    let mut raw_data = Vec::with_capacity(png.data.len());
    for line in png.scan_lines() {
        raw_data.push(line.filter);
        for pixel in line.data.chunks(bpp) {
            match transparency_pixel {
                Some(ref trns) if pixel.iter().skip(colored_bytes).all(|b| *b == 0) => {
                    raw_data.resize(raw_data.len() + colored_bytes, trns[1]);
                }
                _ => raw_data.extend_from_slice(&pixel[0..colored_bytes]),
            };
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
        transparency_pixel,
        palette: None,
    })
}
