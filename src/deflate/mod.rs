mod deflater;
pub use deflater::crc32;
pub use deflater::deflate;
pub use deflater::inflate;

#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;
#[cfg(feature = "zopfli")]
mod zopfli_oxipng;
#[cfg(feature = "zopfli")]
pub use zopfli_oxipng::deflate as zopfli_deflate;

#[derive(Clone, Debug, PartialEq, Eq)]
/// DEFLATE algorithms supported by oxipng
pub enum Deflaters {
    /// Use libdeflater.
    Libdeflater {
        /// Which compression level to use on the file (1-12)
        compression: u8,
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

impl Copy for Deflaters {}
