use crate::atomicmin::AtomicMin;
use crate::Deadline;
use crate::PngError;
use crate::PngResult;
pub use cloudflare_zlib::is_supported;
use cloudflare_zlib::*;

impl From<ZError> for PngError {
    fn from(err: ZError) -> Self {
        match err {
            ZError::DeflatedDataTooLarge(n) => PngError::DeflatedDataTooLong(n),
            other => PngError::Other(other.to_string().into()),
        }
    }
}

pub(crate) fn cfzlib_deflate(
    data: &[u8],
    level: u8,
    strategy: u8,
    window_bits: u8,
    max_size: &AtomicMin,
    deadline: &Deadline,
) -> PngResult<Vec<u8>> {
    let mut stream = Deflate::new(level.into(), strategy.into(), window_bits.into())?;
    stream.reserve(max_size.get().unwrap_or(data.len() / 2));
    let max_size = max_size.as_atomic_usize();
    // max size is generally checked after each split,
    // so splitting the buffer into pieces gives more checks
    // = better chance of hitting it sooner.
    let chunk_size = (data.len() / 4).max(1 << 15).min(1 << 18); // 32-256KB
    for chunk in data.chunks(chunk_size) {
        stream.compress_with_limit(chunk, max_size)?;
        if deadline.passed() {
            return Err(PngError::TimedOut);
        }
    }
    Ok(stream.finish()?)
}

#[test]
fn compress_test() {
    let vec = cfzlib_deflate(
        b"azxcvbnm",
        Z_BEST_COMPRESSION as u8,
        Z_DEFAULT_STRATEGY as u8,
        15,
        &AtomicMin::new(None),
        &Deadline::new(None, false),
    )
    .unwrap();
    let res = crate::deflate::inflate(&vec).unwrap();
    assert_eq!(&res, b"azxcvbnm");
}
