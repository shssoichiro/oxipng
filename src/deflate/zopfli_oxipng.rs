use crate::{PngError, PngResult};
use std::num::NonZeroU8;

pub fn deflate(data: &[u8], iterations: NonZeroU8) -> PngResult<Vec<u8>> {
    use std::cmp::max;

    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    let options = zopfli::Options {
        iteration_count: iterations.into(),
        ..Default::default()
    };
    match zopfli::compress(options, zopfli::Format::Zlib, data, &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}
