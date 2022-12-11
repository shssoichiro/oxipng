use crate::colors::ColorType;
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
