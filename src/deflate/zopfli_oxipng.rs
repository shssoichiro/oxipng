use std::io::{Error, ErrorKind, Read};
use crate::{PngError, PngResult};
use simd_adler32::Adler32;

pub fn deflate(data: &[u8], options: &zopfli::Options) -> PngResult<Vec<u8>> {
    use std::cmp::max;

    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    match zopfli::compress(options, &zopfli::Format::Zlib, data, &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}

/// Forked from zopfli crate
pub trait Hasher {
    fn update(&mut self, data: &[u8]);
}

impl Hasher for &mut Adler32 {
    fn update(&mut self, data: &[u8]) {
        Adler32::write(self, data)
    }
}

/// A reader that wraps another reader, a hasher and an optional counter,
/// updating the hasher state and incrementing a counter of bytes read so
/// far for each block of data read.
pub struct HashingAndCountingRead<'counter, R: Read, H: Hasher> {
    inner: R,
    hasher: H,
    bytes_read: Option<&'counter mut u32>,
}

impl<'counter, R: Read, H: Hasher> HashingAndCountingRead<'counter, R, H> {
    pub fn new(inner: R, hasher: H, bytes_read: Option<&'counter mut u32>) -> Self {
        Self {
            inner,
            hasher,
            bytes_read,
        }
    }
}

impl<R: Read, H: Hasher> Read for HashingAndCountingRead<'_, R, H> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.inner.read(buf) {
            Ok(bytes_read) => {
                self.hasher.update(&buf[..bytes_read]);

                if let Some(total_bytes_read) = &mut self.bytes_read {
                    **total_bytes_read = total_bytes_read
                        .checked_add(bytes_read.try_into().map_err(|_| ErrorKind::Other)?)
                        .ok_or(ErrorKind::Other)?;
                }

                Ok(bytes_read)
            }
            Err(err) => Err(err),
        }
    }
}
