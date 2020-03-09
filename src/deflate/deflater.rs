use crate::{PngError, PngResult};
use crate::atomicmin::AtomicMin;
use libdeflater::{CompressionError, CompressionLvl, Compressor};

pub fn deflate(data: &[u8], max_size: &AtomicMin) -> PngResult<Vec<u8>> {
    let mut compressor = Compressor::new(CompressionLvl::best());
    let capacity = max_size.get().unwrap_or(data.len() / 2);
    let mut dest = Vec::with_capacity(capacity);
    unsafe {
        // This is ok because the Vec contains Copy-able data (u8)
        // and because libdeflater wrapper doesn't try to read
        // the bytes from the target.
        //
        // That said, it should be able to accept MaybeUninit instead,
        // so I raised an upstream issue that should make this safer:
        // https://github.com/adamkewley/libdeflater/issues/1
        dest.set_len(capacity);
    }
    let len = compressor
        .zlib_compress(data, &mut dest)
        .map_err(|err| match err {
            CompressionError::InsufficientSpace => PngError::DeflatedDataTooLong(capacity),
        })?;
    if let Some(max) = max_size.get() {
        if len > max {
            return Err(PngError::DeflatedDataTooLong(max));
        }
    }
    dest.truncate(len);
    Ok(dest)
}
