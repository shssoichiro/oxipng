use crate::atomicmin::AtomicMin;
use crate::error::PngError;
use crate::Deadline;
use crate::PngResult;
use miniz_oxide;
use std::cmp::max;
use zopfli;

#[doc(hidden)]
pub mod miniz_stream;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod cfzlib;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub mod cfzlib {
    pub fn is_supported() -> bool {
        return false;
    }
}

/// Decompress a data stream using the DEFLATE algorithm
pub fn inflate(data: &[u8]) -> PngResult<Vec<u8>> {
    miniz_oxide::inflate::decompress_to_vec_zlib(data)
        .map_err(|e| PngError::new(&format!("Error on decompress: {:?}", e)))
}

/// Compress a data stream using the DEFLATE algorithm
pub(crate) fn deflate(
    data: &[u8],
    zc: u8,
    zs: u8,
    zw: u8,
    max_size: &AtomicMin,
    deadline: &Deadline,
) -> PngResult<Vec<u8>> {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        if cfzlib::is_supported() {
            return cfzlib::cfzlib_deflate(data, zc, zs, zw, max_size, deadline);
        }
    }

    miniz_stream::compress_to_vec_oxipng(data, zc, zw.into(), zs.into(), max_size, deadline)
}

pub fn zopfli_deflate(data: &[u8]) -> PngResult<Vec<u8>> {
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
    /// Use the Zlib/Miniz DEFLATE implementation
    Zlib,
    /// Use the better but slower Zopfli implementation
    Zopfli,
}
