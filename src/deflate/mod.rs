use atomicmin::AtomicMin;
use error::PngError;
use miniz_oxide;
use std::cmp::max;
use zopfli;
use PngResult;

#[doc(hidden)]
pub mod miniz_stream;

/// Decompress a data stream using the DEFLATE algorithm
pub fn inflate(data: &[u8]) -> PngResult<Vec<u8>> {
    miniz_oxide::inflate::decompress_to_vec_zlib(data)
        .map_err(|e| PngError::new(&format!("Error on decompress: {:?}", e)))
}

/// Compress a data stream using the DEFLATE algorithm
pub fn deflate(data: &[u8], zc: u8, zs: u8, zw: u8, max_size: &AtomicMin) -> PngResult<Vec<u8>> {
    #[cfg(feature = "cfzlib")]
    {
        if is_cfzlib_supported() {
            return cfzlib_deflate(data, zc, zs, zw, max_size);
        }
    }

    miniz_stream::compress_to_vec_oxipng(data, zc, zw.into(), zs.into(), max_size)
}

#[cfg(feature = "cfzlib")]
fn is_cfzlib_supported() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse4.2") && is_x86_feature_detected!("pclmulqdq") {
            return true;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if is_arm_feature_detected!("neon") && is_arm_feature_detected!("crc") {
            return true;
        }
    }
    false
}

#[cfg(feature = "cfzlib")]
pub fn cfzlib_deflate(
    data: &[u8],
    level: u8,
    strategy: u8,
    window_bits: u8,
    max_size: &AtomicMin,
) -> PngResult<Vec<u8>> {
    use cloudflare_zlib_sys::*;
    use std::mem;

    assert!(data.len() < u32::max_value() as usize);
    unsafe {
        let mut stream = mem::zeroed();
        if Z_OK != deflateInit2(
            &mut stream,
            level.into(),
            Z_DEFLATED,
            window_bits.into(),
            MAX_MEM_LEVEL,
            strategy.into(),
        ) {
            return Err(PngError::new("deflateInit2"));
        }

        let upper_bound = deflateBound(&mut stream, data.len() as uLong) as usize;
        let max_size = max_size.get().unwrap_or(upper_bound).min(upper_bound);
        // it's important to have the capacity pre-allocated,
        // as unsafe set_len is called later
        let mut out = Vec::with_capacity(max_size);

        stream.next_in = data.as_ptr() as *mut _;
        stream.total_in = data.len() as uLong;
        stream.avail_in = data.len() as uInt;
        stream.next_out = out.as_mut_ptr();
        stream.avail_out = out.capacity() as uInt;
        match deflate(&mut stream, Z_FINISH) {
            Z_STREAM_END => {}
            Z_OK => {
                deflateEnd(&mut stream);
                return Err(PngError::DeflatedDataTooLong(stream.total_out as usize));
            },
            _ => {
                deflateEnd(&mut stream);
                return Err(PngError::new("deflate"));
            },
        }
        if Z_OK != deflateEnd(&mut stream) {
            return Err(PngError::new("deflateEnd"));
        }
        debug_assert!(stream.total_out as usize <= out.capacity());
        out.set_len(stream.total_out as usize);
        Ok(out)
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
/// DEFLATE algorithms supported by oxipng
pub enum Deflaters {
    /// Use the Zlib/Miniz DEFLATE implementation
    Zlib,
    /// Use the better but slower Zopfli implementation
    Zopfli,
}
