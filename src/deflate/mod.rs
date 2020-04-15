use crate::atomicmin::AtomicMin;
use crate::error::PngError;
use crate::Deadline;
use crate::PngResult;
use indexmap::IndexSet;
use miniz_oxide;
use std::cmp::max;
use zopfli;

#[doc(hidden)]
pub mod miniz_stream;

mod deflater;
pub use deflater::deflate as libdeflater_deflate;

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
#[doc(hidden)]
pub fn deflate(
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

#[derive(Clone, Debug, PartialEq)]
/// DEFLATE algorithms supported by oxipng
pub enum Deflaters {
    /// Use the Zlib/Miniz DEFLATE implementation
    Zlib {
        /// Which zlib compression levels to try on the file (1-9)
        ///
        /// Default: `9`
        compression: IndexSet<u8>,
        /// Which zlib compression strategies to try on the file (0-3)
        ///
        /// Default: `0-3`
        strategies: IndexSet<u8>,
        /// Window size to use when compressing the file, as `2^window` bytes.
        ///
        /// Doesn't affect compression but may affect speed and memory usage.
        /// 8-15 are valid values.
        ///
        /// Default: `15`
        window: u8,
    },
    /// Use the better but slower Zopfli implementation
    Zopfli,
    /// Use libdeflater.
    Libdeflater,
}
