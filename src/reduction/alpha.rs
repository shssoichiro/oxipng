use reduction::ReducedPng;
use png::PngData;
use colors::ColorType;
use std::collections::HashMap;

#[must_use]
pub fn reduced_alpha_channel(png: &PngData) -> Option<ReducedPng> {
    let target_color_type = match png.ihdr_data.color_type {
        ColorType::GrayscaleAlpha => ColorType::Grayscale,
        ColorType::RGBA => ColorType::RGB,
        _ => return None,
    };
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
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

    let mut raw_data = Vec::with_capacity(png.raw_data.len());
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

    let mut aux_headers = HashMap::new();
    // sBIT contains information about alpha channel's original depth,
    // and alpha has just been removed
    if let Some(sbit_header) = png.aux_headers.get(b"sBIT") {
        // Some programs save the sBIT header as RGB even if the image is RGBA.
        aux_headers.insert(*b"sBIT", Some(sbit_header.iter().cloned().take(3).collect()));
    }

    Some(ReducedPng {
        raw_data,
        bit_depth: png.ihdr_data.bit_depth,
        color_type: target_color_type,
        aux_headers,
        transparency_pixel: None,
        palette: None,
    })
}
