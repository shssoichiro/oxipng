use png::PngData;

pub fn reduce_alpha_channel(png: &mut PngData, channels: u8) -> Option<Vec<u8>> {
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp = channels * byte_depth;
    let bpp_mask = bpp - 1;
    assert_eq!(0, bpp & bpp_mask);
    let colored_bytes = bpp - byte_depth;
    for line in png.scan_lines() {
        for (i, &byte) in line.data.iter().enumerate() {
            if i as u8 & bpp_mask >= colored_bytes {
                if byte != 255 {
                    return None;
                }
            }
        }
    }

    let mut reduced = Vec::with_capacity(png.raw_data.len());
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, &byte) in line.data.iter().enumerate() {
            if i as u8 & bpp_mask >= colored_bytes {
                continue;
            } else {
                reduced.push(byte);
            }
        }
    }

    // sBIT contains information about alpha channel's original depth,
    // and alpha has just been removed
    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), channels as usize);
        sbit_header.pop();
    }

    Some(reduced)
}
