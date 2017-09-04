use error::PngError;
use miniz_sys;
use std::cmp::max;
use zopfli;

#[doc(hidden)]
pub mod miniz_stream;

/// Decompress a data stream using the DEFLATE algorithm
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, PngError> {
    let mut input = data.to_owned();
    let mut stream = miniz_stream::Stream::new_decompress();
    let mut output = Vec::with_capacity(data.len());
    loop {
        match stream.decompress_vec(input.as_mut(), output.as_mut()) {
            miniz_sys::MZ_OK => output.reserve(data.len()),
            miniz_sys::MZ_STREAM_END => break,
            c => return Err(PngError::new(&format!("Error code on decompress: {}", c))),
        }
    }
    output.shrink_to_fit();

    Ok(output)
}

/// Compress a data stream using the zlib implementation of the DEFLATE algorithm
pub fn deflate(data: &[u8], zc: u8, zm: u8, zs: u8, zw: u8) -> Result<Vec<u8>, PngError> {
    let mut input = data.to_owned();
    // Compressed input should be smaller than decompressed, so allocate less than data.len()
    // However, it needs a minimum capacity in order to handle very small images
    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    let mut stream = miniz_stream::Stream::new_compress(
        i32::from(zc),
        i32::from(zw),
        i32::from(zm),
        i32::from(zs),
    );
    loop {
        match stream.compress_vec(input.as_mut(), output.as_mut()) {
            miniz_sys::MZ_OK => output.reserve(max(1024, data.len() / 20)),
            miniz_sys::MZ_STREAM_END => break,
            c => return Err(PngError::new(&format!("Error code on compress: {}", c))),
        }
    }

    output.shrink_to_fit();

    Ok(output)
}

pub fn zopfli_deflate(data: &[u8]) -> Result<Vec<u8>, PngError> {
    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    let options = zopfli::Options::default();
    match zopfli::compress(&options, &zopfli::Format::Zlib, data, &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// DEFLATE algorithms supported by oxipng
pub enum Deflaters {
    /// Use the Zlib DEFLATE implementation
    Zlib,
    /// Use the better but slower Zopfli implementation
    Zopfli,
}
