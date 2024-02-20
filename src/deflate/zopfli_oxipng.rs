use std::num::NonZeroU8;

use crate::{PngError, PngResult};

pub fn deflate(data: &[u8], iterations: NonZeroU8) -> PngResult<Vec<u8>> {
    let mut output = Vec::with_capacity(data.len());
    let options = zopfli::Options {
        iteration_count: iterations.into(),
        ..Default::default()
    };
    // Since Rust v1.74, passing &[u8] directly into zopfli causes a regression in compressed size
    // for some files. Wrapping the slice in another Read implementer such as Box fixes it for now.
    match zopfli::compress(options, zopfli::Format::Zlib, Box::new(data), &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}
