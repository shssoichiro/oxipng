mod deflater;
use crate::AtomicMin;
use crate::{PngError, PngResult};
pub use deflater::crc32;
pub use deflater::deflate;
pub use deflater::inflate;
use std::{fmt, fmt::Display, io};
use std::io::{BufWriter, Cursor, Write};

#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;
#[cfg(feature = "zopfli")]
use zopfli::{DeflateEncoder, Options};

#[cfg(feature = "zopfli")]
mod zopfli_oxipng;
#[cfg(feature = "zopfli")]
pub use zopfli_oxipng::deflate as zopfli_deflate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub trait Deflater: Sync + Send {
    fn deflate(&self, data: &[u8], max_size: &AtomicMin) -> PngResult<Vec<u8>>;
}

impl Deflater for Deflaters {
    fn deflate(&self, data: &[u8], max_size: &AtomicMin) -> PngResult<Vec<u8>> {
        let compressed = match self {
            Self::Libdeflater { compression } => deflate(data, *compression, max_size)?,
            #[cfg(feature = "zopfli")]
            Self::Zopfli { iterations } => zopfli_deflate(data, *iterations)?,
        };
        if let Some(max) = max_size.get() {
            if compressed.len() > max {
                return Err(PngError::DeflatedDataTooLong(max));
            }
        }
        Ok(compressed)
    }
}

#[cfg(feature = "zopfli")]
#[derive(Copy, Clone, Debug)]
pub struct BufferedZopfliDeflater {
    iterations: NonZeroU8,
    input_buffer_size: usize,
    output_buffer_size: usize
}

impl BufferedZopfliDeflater {
    pub const fn new(iterations: NonZeroU8,
                 input_buffer_size: usize,
                 output_buffer_size: usize) -> Self {
        BufferedZopfliDeflater {iterations, input_buffer_size, output_buffer_size }
    }
}

#[cfg(feature = "zopfli")]
impl Deflater for BufferedZopfliDeflater {
    fn deflate(&self, data: &[u8], max_size: &AtomicMin) -> PngResult<Vec<u8>> {
        let options = Options {
            iteration_count: self.iterations,
            ..Default::default()
        };
        let mut buffer = BufWriter::with_capacity(self.input_buffer_size,
                                                  DeflateEncoder::new(
            options, Default::default(), Cursor::new(Vec::new())));
        let result = (|| -> io::Result<Vec<u8>> {
            buffer.write_all(data)?;
            Ok(buffer.into_inner()?.finish()?.into_inner())
        })();
        let result = result.map_err(|e| PngError::new(&e.to_string()))?;
        if max_size.get().is_some_and(|max| max < result.len()) {
            Err(PngError::DeflatedDataTooLong(result.len()))
        } else {
            Ok(result)
        }
    }
}

impl Display for Deflaters {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Libdeflater { compression } => Display::fmt(compression, f),
            #[cfg(feature = "zopfli")]
            Self::Zopfli { .. } => Display::fmt("zopfli", f),
        }
    }
}
