mod deflater;
use crate::AtomicMin;
use crate::{PngError, PngResult};
pub use deflater::crc32;
pub use deflater::deflate;
pub use deflater::inflate;
use std::{fmt, fmt::Display};

#[cfg(feature = "zopfli")]
use std::io::{self, copy, BufWriter, Cursor, Write};

#[cfg(feature = "zopfli")]
use zopfli::{DeflateEncoder, Options};
#[cfg(feature = "zopfli")]
mod zopfli_oxipng;
#[cfg(feature = "zopfli")]
use simd_adler32::Adler32;
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
        /// Zopfli compression options
        options: Options,
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
            Self::Zopfli { options } => zopfli_deflate(data, options)?,
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
    input_buffer_size: usize,
    output_buffer_size: usize,
    options: Options,
}

#[cfg(feature = "zopfli")]
impl BufferedZopfliDeflater {
    pub const fn new(
        input_buffer_size: usize,
        output_buffer_size: usize,
        options: Options,
    ) -> Self {
        BufferedZopfliDeflater {
            input_buffer_size,
            output_buffer_size,
            options,
        }
    }
}

#[cfg(feature = "zopfli")]
impl Default for BufferedZopfliDeflater {
    fn default() -> Self {
        BufferedZopfliDeflater {
            input_buffer_size: 1024 * 1024,
            output_buffer_size: 64 * 1024,
            options: Options::default(),
        }
    }
}

#[cfg(feature = "zopfli")]
impl Deflater for BufferedZopfliDeflater {
    /// Fork of the zlib_compress function in Zopfli.
    fn deflate(&self, data: &[u8], max_size: &AtomicMin) -> PngResult<Vec<u8>> {
        let mut out = Cursor::new(Vec::with_capacity(self.output_buffer_size));
        let cmf = 120; /* CM 8, CINFO 7. See zlib spec.*/
        let flevel = 3;
        let fdict = 0;
        let mut cmfflg: u16 = 256 * cmf + fdict * 32 + flevel * 64;
        let fcheck = 31 - cmfflg % 31;
        cmfflg += fcheck;

        let out = (|| -> io::Result<Vec<u8>> {
            let mut rolling_adler = Adler32::new();
            let mut in_data =
                zopfli_oxipng::HashingAndCountingRead::new(data, &mut rolling_adler, None);
            out.write_all(&cmfflg.to_be_bytes())?;
            let mut buffer = BufWriter::with_capacity(
                self.input_buffer_size,
                DeflateEncoder::new(self.options, Default::default(), &mut out),
            );
            copy(&mut in_data, &mut buffer)?;
            buffer.into_inner()?.finish()?;
            out.write_all(&rolling_adler.finish().to_be_bytes())?;
            Ok(out.into_inner())
        })();
        let out = out.map_err(|e| PngError::new(&e.to_string()))?;
        if max_size.get().map(|max| max < out.len()).unwrap_or(false) {
            Err(PngError::DeflatedDataTooLong(out.len()))
        } else {
            Ok(out)
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
