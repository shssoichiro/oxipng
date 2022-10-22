use crate::error::PngError;
use crate::PngResult;
use indexmap::IndexSet;

#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;

mod deflater;
pub use deflater::crc32;
pub use deflater::deflate;
pub use deflater::inflate;

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
    /// Use libdeflater.
    Libdeflater {
        /// Which compression levels to try on the file (1-12)
        compression: IndexSet<u8>,
    },
    #[cfg(feature = "zopfli")]
    /// Use the better but slower Zopfli implementation
    Zopfli {
        /// The number of compression iterations to do. 15 iterations are fine
        /// for small files, but bigger files will need to be compressed with
        /// less iterations, or else they will be too slow.
        iterations: NonZeroU8,
    },
}
