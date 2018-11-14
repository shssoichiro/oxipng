use atomicmin::AtomicMin;
use PngResult;
use PngError;
use cloudflare_zlib::*;
pub use cloudflare_zlib::is_supported;

impl From<ZError> for PngError {
    fn from(err: ZError) -> Self {
        match err {
            ZError::DeflatedDataTooLarge(n) => PngError::DeflatedDataTooLong(n),
            other => PngError::Other(other.to_string().into()),
        }
    }
}

pub fn cfzlib_deflate(
    data: &[u8],
    level: u8,
    strategy: u8,
    window_bits: u8,
    max_size: &AtomicMin,
) -> PngResult<Vec<u8>> {
    let mut stream = Deflate::new(level.into(), strategy.into(), window_bits.into())?;
    stream.reserve(max_size.get().unwrap_or(data.len()/2));
    let max_size = max_size.as_atomic_usize();
    // max size is generally checked after each split,
    // so splitting the buffer into pieces gices more checks
    // = better chance of hitting it sooner.
    let (first, rest) = data.split_at(data.len()/2);
    stream.compress_with_limit(first, max_size)?;
    let (rest1, rest2) = rest.split_at(rest.len()/2);
    stream.compress_with_limit(rest1, max_size)?;
    stream.compress_with_limit(rest2, max_size)?;
    Ok(stream.finish()?)
}

#[test]
fn compress_test() {
    let vec = cfzlib_deflate(b"azxcvbnm", Z_BEST_COMPRESSION as u8, Z_DEFAULT_STRATEGY as u8, 15, &AtomicMin::new(None)).unwrap();
    let res = ::deflate::inflate(&vec).unwrap();
    assert_eq!(&res, b"azxcvbnm");
}
