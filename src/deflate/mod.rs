use error::PngError;
use libz_sys;
use miniz_sys;
use libc::c_int;
use std::cmp::max;
use zopfli;

pub mod libz_stream;
pub mod miniz_stream;

/// Decompress a data stream using the DEFLATE algorithm
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, PngError> {
    let mut input = data.to_owned();
    let mut stream = libz_stream::Stream::new_decompress();
    let mut output = Vec::with_capacity(data.len());
    loop {
        match stream.decompress_vec(input.as_mut(), output.as_mut()) {
            libz_sys::Z_OK => output.reserve(data.len()),
            libz_sys::Z_STREAM_END => break,
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
    if zs == 0 || zs == 1 {
        // Miniz performs 1-2 orders of magnitude better for strategies 0 and 1
        let mut stream =
            miniz_stream::Stream::new_compress(zc as c_int, zw as c_int, zm as c_int, zs as c_int);
        loop {
            match stream.compress_vec(input.as_mut(), output.as_mut()) {
                miniz_sys::MZ_OK => output.reserve(max(1024, data.len() / 20)),
                miniz_sys::MZ_STREAM_END => break,
                c => return Err(PngError::new(&format!("Error code on compress: {}", c))),
            }
        }
    } else {
        // libz performs an order of magnitude better for strategies 2 and 3
        let mut stream =
            libz_stream::Stream::new_compress(zc as c_int, zw as c_int, zm as c_int, zs as c_int);
        loop {
            match stream.compress_vec(input.as_mut(), output.as_mut()) {
                libz_sys::Z_OK => output.reserve(max(1024, data.len() / 20)),
                libz_sys::Z_STREAM_END => break,
                c => return Err(PngError::new(&format!("Error code on compress: {}", c))),
            }
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
