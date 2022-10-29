use crate::atomicmin::AtomicMin;
use crate::error::PngError;
use crate::Deadline;
use crate::PngResult;
use indexmap::IndexSet;

#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;

#[doc(hidden)]
pub mod miniz_stream;

#[cfg(feature = "libdeflater")]
mod deflater;
#[cfg(feature = "libdeflater")]
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
    miniz_oxide::inflate::decompress_to_vec_zlib(data).map_err(|e| {
        PngError::new(&format!(
            "Error on decompress: {:?} (after {:?} decompressed bytes)",
            e.status,
            e.output.len()
        ))
    })
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

#[cfg(feature = "zopfli")]
pub fn zopfli_deflate(data: &[u8], iterations: NonZeroU8) -> PngResult<Vec<u8>> {
    use std::cmp::max;

    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    let options = zopfli::Options {
        iteration_count: iterations,
        ..Default::default()
    };
    match zopfli::compress(&options, &zopfli::Format::Zlib, data, &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    #[cfg(feature = "zopfli")]
    /// Use the better but slower Zopfli implementation
    Zopfli {
        /// The number of compression iterations to do. 15 iterations are fine
        /// for small files, but bigger files will need to be compressed with
        /// less iterations, or else they will be too slow.
        iterations: NonZeroU8,
    },
    #[cfg(feature = "libdeflater")]
    /// Use libdeflater.
    Libdeflater {
        /// Which compression levels to try on the file (1-12)
        compression: IndexSet<u8>,
    },
}
