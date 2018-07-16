use png::PngData;

pub fn reduce_alpha_channel(png: &mut PngData, channels: usize) -> Option<Vec<u8>> {
    let byte_depth: u8 = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = channels * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        for (i, &byte) in line.data.iter().enumerate() {
            if i % bpp >= colored_bytes {
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
            if i % bpp >= colored_bytes {
                continue;
            } else {
                reduced.push(byte);
            }
        }
    }

    // sBIT contains information about alpha channel's original depth,
    // and alpha has just been removed
    if let Some(sbit_header) = png.aux_headers.get_mut(&"sBIT".to_string()) {
        assert_eq!(sbit_header.len(), channels);
        sbit_header.pop();
    }

    Some(reduced)
}
